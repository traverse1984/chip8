use super::mem::State;
use super::program::Program;
use super::status::{Error, Status};
use crate::pal::{Buzzer, Delay, Keypad, Screen};

pub struct Chip8<S, K, B, D>
where
    S: Screen,
    K: Keypad,
    B: Buzzer,
    D: Delay,
{
    state: State,
    screen: S,
    keypad: K,
    buzzer: B,
    delay: D,
}

impl<S, K, B, D> Chip8<S, K, B, D>
where
    S: Screen,
    K: Keypad,
    B: Buzzer,
    D: Delay,
{
    pub const KEYPAD_POLL_FREQUENCY: u32 = 1000;
    pub const INST_LEN: u8 = 2;
    pub const STEP: u16 = 2;
    pub const FLAG: u8 = 0x0F;

    fn load<'a, P: Into<Program<'a>>>(&mut self, program: P) -> Result<(), ()> {
        self.state
            .ram
            .load(0x200, program.into().as_bytes())
            .map_err(|_| ())
    }

    fn run(&mut self) -> Status {
        self.state.pc = 0x200;
        loop {
            self.step()?;
        }
    }

    fn read_instruction(&mut self, addr: u16) -> Result<u16, Error> {
        let addr = self.state.ram.to_valid_address(addr)?;

        if addr % Self::STEP == 0 {
            let bytes = self.state.ram.read_bytes(addr, Self::INST_LEN)?;
            Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
        } else {
            Err(Error::NotAligned(addr))
        }
    }

    fn step(&mut self) -> Status {
        self.read_instruction(self.state.pc)
            .and_then(|inst| self.exec(inst))?;

        self.state.pc += Self::STEP;
        Ok(())
    }

    fn exec(&mut self, instruction: u16) -> Status {
        let cmd = instruction >> 12;
        let nnn = instruction & 0x0FFF;
        let byte = nnn as u8; // Lossy
        let nibble = byte & 0xF;
        let vx = (nnn >> 8) as u8;
        let vy = byte >> 4;

        let State {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
        } = &mut self.state;

        match cmd {
            // CLS
            0 if nnn == 0x0E0 => {
                self.screen.clear().map_err(|e| e.into())?;
                Ok(())
            }

            // RET
            0 if nnn == 0x0EE => {
                *pc = stack.pop()?;
                Ok(())
            }

            // JP addr
            1 => {
                *pc = nnn;
                Ok(())
            }

            // CALL addr
            2 => {
                stack.push(*pc)?;
                *pc = nnn;
                Ok(())
            }

            // SE Vx, byte
            3 => {
                if reg.get(vx)? == byte {
                    *pc += Self::STEP;
                }
                Ok(())
            }

            // SNE Vx, byte
            4 => {
                if reg.get(vx)? != byte {
                    *pc += Self::STEP;
                }
                Ok(())
            }

            // // SE Vx, Vy, 0
            5 if nibble == 0 => {
                if reg.get(vx)? == reg.get(vy)? {
                    *pc += Self::STEP;
                }
                Ok(())
            }

            // // LD Vx, byte
            6 => {
                reg.set(vx, byte)?;
                Ok(())
            }

            // // ADD Vx, byte
            7 => {
                let add = reg.get(vx)?.wrapping_add(byte);
                reg.set(vx, add)?;
                Ok(())
            }

            // // XOR
            8 => {
                let byte = match nibble {
                    // LD Vx, Vy
                    0 => reg.get(vy)?,

                    // OR Vx, Vy
                    1 => reg.get(vx)? | reg.get(vy)?,

                    // AND Vx, Vy
                    2 => reg.get(vx)? & reg.get(vy)?,

                    // XOR Vx, Vy
                    3 => reg.get(vx)? ^ reg.get(vy)?,

                    // ADD Vx, Vy
                    4 => {
                        let (x, y) = (reg.get(vx)?, reg.get(vy)?);
                        match x.checked_add(y) {
                            Some(val) => val,
                            None => {
                                reg.set(Self::FLAG, 1)?;
                                x.wrapping_add(y)
                            }
                        }
                    }

                    // SUB Vx, Vy
                    5 => {
                        let (x, y) = (reg.get(vx)?, reg.get(vy)?);
                        reg.set(Self::FLAG, (x > y) as u8)?;
                        x.wrapping_sub(y)
                    }

                    // SHR Vx (, Vy)
                    6 => {
                        let x = reg.get(vx)?;
                        reg.set(Self::FLAG, x & 1)?;
                        x >> 1
                    }

                    // SUBN Vx, Vy
                    7 => {
                        let (x, y) = (reg.get(vx)?, reg.get(vy)?);
                        reg.set(Self::FLAG, (y > x) as u8)?;
                        y.wrapping_sub(x)
                    }

                    // SHL Vx (, Vy)
                    0xE => {
                        let x = reg.get(vx)?;
                        reg.set(Self::FLAG, x >> 7)?;
                        x << 1
                    }

                    _ => return Err(Error::Instruction(instruction)),
                };

                reg.set(vx, byte)?;
                Ok(())
            }

            // // SNE
            9 if nibble == 0 => {
                if reg.get(vx)? != reg.get(vy)? {
                    *pc += Self::STEP;
                }
                Ok(())
            }

            // // LD I, addr
            0xA => {
                *i = nnn;
                Ok(())
            }

            // // JP V0, addr
            0xB => {
                *pc = nnn + reg.get(0)? as u16;
                Ok(())
            }

            // // RND Vx, byte
            0xC => todo!("Rand"),

            // // DRW Vx, Vy, len
            0xD => {
                let x = reg.get(vx)?;
                let y = reg.get(vy)?;
                let data = ram.read_bytes(*i, nibble)?;

                let erased = self.screen.xor(x, y, data).map_err(|e| e.into())?;
                reg.set(Self::FLAG, erased as u8)?;
                Ok(())
            }

            // // SKP Vx
            0xE if byte == 0x9E => {
                match self
                    .keypad
                    .read_key(&mut self.delay)
                    .map_err(|e| e.into())?
                {
                    Some(key) if key == reg.get(vx)? => *pc += Self::STEP,
                    _ => (),
                }
                Ok(())
            }

            // // SKNP Vx
            0xE if byte == 0xA1 => {
                match self
                    .keypad
                    .read_key(&mut self.delay)
                    .map_err(|e| e.into())?
                {
                    Some(key) if key != reg.get(vx)? => *pc += Self::STEP,
                    _ => (),
                }

                Ok(())
            }

            0xF if byte == 0x07 => {
                reg.set(vx, *dt)?;
                Ok(())
            }

            // LD Vx, K
            // All execution means what? Also stop secrementing timers?
            0xF if byte == 0x0A => {
                let key = loop {
                    if let Some(key) = self
                        .keypad
                        .read_key(&mut self.delay)
                        .map_err(|e| e.into())?
                    {
                        break key;
                    }

                    self.delay
                        .delay_us(Self::KEYPAD_POLL_FREQUENCY)
                        .map_err(|e| e.into())?;
                };

                reg.set(vx, key)?;
                Ok(())
            }

            // LD DT, Vx
            0xF if byte == 0x15 => {
                *dt = reg.get(vx)?;
                Ok(())
            }

            // LD ST, Vx
            0xF if byte == 0x18 => {
                *st = reg.get(vx)?;
                Ok(())
            }

            // ADD I, Vx
            0xF if byte == 0x1E => {
                *i = i.wrapping_add(reg.get(vx)? as u16);
                Ok(())
            }

            // LD F, Vx
            0xF if byte == 0x29 => {
                *i = reg.get(vx).and_then(|x| ram.get_sprite_addr(x))?;
                Ok(())
            }

            // LD B, Vx
            0xF if byte == 0x33 => {
                let x = reg.get(vx)?;
                let hundreds = x / 100;
                let tens = (x / 10) % 10;
                let units = x % 10;

                ram.write_byte(*i, hundreds)?;
                ram.write_byte(*i + 1, tens)?;
                ram.write_byte(*i + 2, units)?;

                Ok(())
            }

            // LD [I], Vx
            0xF if byte == 0x55 => {
                for index in 0..=vx {
                    reg.get(index)
                        .and_then(|x| ram.write_byte(*i + index as u16, x))?;
                }
                Ok(())
            }

            // Ld Vx, [I]
            0xF if byte == 0x65 => {
                for (val, index) in ram.read_bytes(*i, vx + 1)?.iter().copied().zip(0..=vx) {
                    reg.set(index, val)?;
                }
                Ok(())
            }

            _ => Err(Error::Instruction(instruction)),
        }
    }
}

impl<S, K, B, D> Chip8<S, K, B, D>
where
    S: Screen,
    K: Keypad,
    B: Buzzer,
    D: Delay,
{
    pub fn new(screen: S, keypad: K, buzzer: B, delay: D) -> Self {
        Self::from_state(screen, keypad, buzzer, delay, State::default())
    }

    pub fn from_state(screen: S, keypad: K, buzzer: B, delay: D, state: State) -> Self {
        Self {
            state,
            screen,
            keypad,
            buzzer,
            delay,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn free(self) -> (S, K, B, D, State) {
        let Chip8 {
            screen,
            keypad,
            buzzer,
            delay,
            state,
        } = self;

        (screen, keypad, buzzer, delay, state)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::vec;

    use super::{super::chip, *};
    use crate::vm::mocks::ScreenCommand;

    macro_rules! exec {
        ($($inst: expr),+ $(; $($tail: tt)*)?) => {{
            let mut chip = chip!( $($($tail)*)? );
            $(chip.exec($inst).unwrap();)+
            chip
        }};
    }

    #[test]
    fn load() {
        let mut chip = chip!();
        chip.load(&[1u8, 2, 3, 4][..]).unwrap();

        assert_eq!(chip.state.ram.read_bytes(0x200, 4).unwrap(), &[1, 2, 3, 4]);
    }

    #[test]
    fn read_instruction() {
        let mut chip = chip!();
        chip.load(&mut [0x11u16, 0x22u16, 0x33u16][..]).unwrap();

        assert_eq!(chip.read_instruction(0x200).unwrap(), 0x11);
        assert_eq!(chip.read_instruction(0x202).unwrap(), 0x22);
        assert_eq!(chip.read_instruction(0x204).unwrap(), 0x33);

        assert_eq!(
            chip.read_instruction(0x201).unwrap_err(),
            Error::NotAligned(0x201)
        )
    }

    #[test]
    fn cls() {
        let (screen, ..) = exec!(0x00E0).free();
        assert_eq!(screen.commands, vec![ScreenCommand::Clear])
    }

    #[test]
    fn jp() {
        let mut chip = chip!();

        chip.exec(0x1123).unwrap();
        assert_eq!(chip.state.pc, 0x0123);

        chip.exec(0x1456).unwrap();
        assert_eq!(chip.state.pc, 0x0456);
    }

    #[test]
    fn call() {
        let mut chip = chip!();
        chip.state.pc = 0x0123;
        chip.exec(0x2456).unwrap();

        assert_eq!(chip.state.pc, 0x0456);
        assert_eq!(chip.state.stack.pop().unwrap(), 0x0123);
    }

    #[test]
    fn se_3xkk() {}
}
