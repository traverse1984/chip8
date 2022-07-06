use super::mem::{Ram, Registers, Stack, State, V_FLAG};
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
    i: u16,
    pc: u16,
    dt: u8,
    st: u8,
    reg: Registers,
    stack: Stack,
    ram: Ram,

    screen: S,
    keypad: K,
    buzzer: B,
    delay: D,
}

macro_rules! ok {
    ($($tail: tt)*) => {{
        $($tail)*;
        Ok(())
    }};
}

impl<S, K, B, D> Chip8<S, K, B, D>
where
    S: Screen,
    K: Keypad,
    B: Buzzer,
    D: Delay,
{
    pub const KEYPAD_POLL_FREQUENCY: u32 = 1000;
    pub const INSTRUCTION_SIZE: u16 = 2;

    fn load<'a, P: Into<Program<'a>>>(&mut self, program: P) -> Result<(), ()> {
        self.ram.load(0x200, program.into().as_bytes()).unwrap();
        Ok(())
    }

    fn step(&mut self) -> Status {
        Ok(())
    }

    fn exec(&mut self, instruction: u16) -> Status {
        let cmd = instruction >> 12;
        let nnn = instruction & 0x0FFF;
        let byte = nnn as u8; // Lossy
        let nibble = byte & 0xF;
        let vx = (nnn >> 8) as u8;
        let vy = byte >> 4;

        match cmd {
            // CLS
            0 if nnn == 0x0E0 => {
                self.screen.clear().map_err(|e| e.into())?;
                Ok(())
            }

            // RET
            0 if nnn == 0x0EE => {
                self.pc = self.stack.pop()?;
                Ok(())
            }

            // JP addr
            1 => {
                self.pc = nnn;
                Ok(())
            }

            // CALL addr
            2 => {
                self.stack.push(self.pc)?;
                self.pc = nnn;
                Ok(())
            }

            // SE Vx, byte
            3 => {
                if self.reg.get(vx)? == byte {
                    self.pc += Self::INSTRUCTION_SIZE;
                }
                Ok(())
            }

            // SNE Vx, byte
            4 => {
                if self.reg.get(vx)? != byte {
                    self.pc += Self::INSTRUCTION_SIZE;
                }
                Ok(())
            }

            // // SE Vx, Vy, 0
            5 if nibble == 0 => {
                if self.reg.get(vx)? == self.reg.get(vy)? {
                    self.pc += Self::INSTRUCTION_SIZE;
                }
                Ok(())
            }

            // // LD Vx, byte
            6 => {
                self.reg.set(vx, byte)?;
                Ok(())
            }

            // // ADD Vx, byte
            7 => {
                let add = self.reg.get(vx)?.wrapping_add(byte);
                self.reg.set(vx, add)?;
                Ok(())
            }

            // // XOR
            8 => {
                let byte = match nibble {
                    // LD Vx, Vy
                    0 => self.reg.get(vy)?,

                    // OR Vx, Vy
                    1 => self.reg.get(vx)? | self.reg.get(vy)?,

                    // AND Vx, Vy
                    2 => self.reg.get(vx)? & self.reg.get(vy)?,

                    // XOR Vx, Vy
                    3 => self.reg.get(vx)? ^ self.reg.get(vy)?,

                    // ADD Vx, Vy
                    4 => {
                        let (x, y) = (self.reg.get(vx)?, self.reg.get(vy)?);
                        match x.checked_add(y) {
                            Some(val) => val,
                            None => {
                                self.reg.set(V_FLAG, 1)?;
                                x.wrapping_add(y)
                            }
                        }
                    }

                    // SUB Vx, Vy
                    5 => {
                        let (x, y) = (self.reg.get(vx)?, self.reg.get(vy)?);
                        self.reg.set(V_FLAG, (x > y) as u8)?;
                        x.wrapping_sub(y)
                    }

                    // SHR Vx (, Vy)
                    6 => {
                        let x = self.reg.get(vx)?;
                        self.reg.set(V_FLAG, x & 1)?;
                        x >> 1
                    }

                    // SUBN Vx, Vy
                    7 => {
                        let (x, y) = (self.reg.get(vx)?, self.reg.get(vy)?);
                        self.reg.set(V_FLAG, (y > x) as u8)?;
                        y.wrapping_sub(x)
                    }

                    // SHL Vx (, Vy)
                    0xE => {
                        let x = self.reg.get(vx)?;
                        self.reg.set(V_FLAG, x >> 7)?;
                        x << 1
                    }

                    _ => return Err(Error::Instruction(instruction)),
                };

                self.reg.set(vx, byte)?;
                Ok(())
            }

            // // SNE
            9 if nibble == 0 => {
                if self.reg.get(vx)? != self.reg.get(vy)? {
                    self.pc += Self::INSTRUCTION_SIZE;
                }
                Ok(())
            }

            // // LD I, addr
            0xA => {
                self.i = nnn;
                Ok(())
            }

            // // JP V0, addr
            0xB => {
                self.pc = nnn + self.reg.get(0)? as u16;
                Ok(())
            }

            // // RND Vx, byte
            0xC => todo!("Rand"),

            // // DRW Vx, Vy, len
            0xD => {
                let x = self.reg.get(vx)?;
                let y = self.reg.get(vy)?;
                let data = self.ram.read_bytes(self.i, nibble)?;

                let erased = self.screen.xor(x, y, data).map_err(|e| e.into())?;
                self.reg.set(V_FLAG, erased as u8)?;
                Ok(())
            }

            // // SKP Vx
            0xE if byte == 0x9E => {
                match self
                    .keypad
                    .read_key(&mut self.delay)
                    .map_err(|e| e.into())?
                {
                    Some(key) if key == self.reg.get(vx)? => self.pc += Self::INSTRUCTION_SIZE,
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
                    Some(key) if key != self.reg.get(vx)? => self.pc += Self::INSTRUCTION_SIZE,
                    _ => (),
                }

                Ok(())
            }

            0xF if byte == 0x07 => {
                self.reg.set(vx, self.dt)?;
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

                self.reg.set(vx, key)?;
                Ok(())
            }

            // LD DT, Vx
            0xF if byte == 0x15 => {
                self.dt = self.reg.get(vx)?;
                Ok(())
            }

            // LD ST, Vx
            0xF if byte == 0x18 => {
                self.st = self.reg.get(vx)?;
                Ok(())
            }

            // ADD I, Vx
            0xF if byte == 0x1E => {
                self.i = self.i.wrapping_add(self.reg.get(vx)? as u16);
                Ok(())
            }

            // LD F, Vx
            0xF if byte == 0x29 => {
                self.i = self.reg.get(vx).and_then(|x| self.ram.get_sprite_addr(x))?;
                Ok(())
            }

            // LD B, Vx
            0xF if byte == 0x33 => {
                let x = self.reg.get(vx)?;
                let hundreds = x / 100;
                let tens = (x / 10) % 10;
                let units = x % 10;

                self.ram.write_byte(self.i, hundreds)?;
                self.ram.write_byte(self.i + 1, tens)?;
                self.ram.write_byte(self.i + 2, units)?;

                Ok(())
            }

            // LD [I], Vx
            0xF if byte == 0x55 => {
                for index in 0..=vx {
                    self.reg
                        .get(index)
                        .and_then(|x| self.ram.write_byte(self.i + index as u16, x))?;
                }
                Ok(())
            }

            // Ld Vx, [I]
            0xF if byte == 0x65 => {
                for (val, index) in self
                    .ram
                    .read_bytes(self.i, vx + 1)?
                    .iter()
                    .copied()
                    .zip(0..=vx)
                {
                    self.reg.set(index, val)?;
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
        let State {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
        } = state;

        Self {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
            screen,
            keypad,
            buzzer,
            delay,
        }
    }

    pub fn state(&self) -> State {
        let Chip8 {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
            ..
        } = *self;

        State {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
        }
    }

    pub fn free(self) -> (S, K, B, D, State) {
        let state = self.state();

        let Chip8 {
            screen,
            keypad,
            buzzer,
            delay,
            ..
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
    fn cls() {
        let (screen, ..) = exec!(0x00E0).free();
        assert_eq!(screen.commands, vec![ScreenCommand::Clear])
    }

    #[test]
    fn jp() {
        let mut chip = chip!();

        chip.exec(0x1123).unwrap();
        assert_eq!(chip.state().pc, 0x0123);

        chip.exec(0x1456).unwrap();
        assert_eq!(chip.state().pc, 0x0456);
    }

    #[test]
    fn call() {
        let mut chip = chip!();
        chip.pc = 0x0123;
        chip.exec(0x2456).unwrap();

        assert_eq!(chip.pc, 0x0456);
        assert_eq!(chip.state().stack.pop().unwrap(), 0x0123);
    }

    #[test]
    fn se_3xkk() {}
}
