use super::mem::Mem;
use super::program::Program;
use super::status::{Error, Status};
use crate::hal::{Buzzer, Delay, Keypad, Rng, Screen};

pub struct Chip8<S, K, B, R, D>
where
    S: Screen,
    K: Keypad,
    B: Buzzer,
    R: Rng,
    D: Delay,
{
    screen: S,
    keypad: K,
    buzzer: B,
    rng: R,
    delay: D,
    mem: Mem,
}

impl<S, K, B, R, D> Chip8<S, K, B, R, D>
where
    S: Screen,
    K: Keypad,
    B: Buzzer,
    R: Rng,
    D: Delay,
{
    pub const POLL_FREQ: u32 = 1000;
    pub const INST_STEP: u16 = 2;
    pub const INST_BYTES: u8 = 2;
    pub const FLAG: u8 = 0x0F;

    fn load<'a, P: Into<Program<'a>>>(&mut self, program: P) -> Result<(), ()> {
        self.mem
            .ram
            .load(0x200, program.into().as_bytes())
            .map_err(|_| ())
    }

    fn run(&mut self) -> Status {
        self.mem.pc = 0x200;
        loop {
            self.step()?;
        }
    }

    fn read_inst(&mut self, addr: u16) -> Result<u16, Error> {
        if self.mem.ram.to_read_addr(addr)? % Self::INST_STEP == 0 {
            Ok(u16::from_be_bytes([
                self.mem.ram.read_byte(addr)?,
                self.mem.ram.read_byte(addr.saturating_add(1))?,
            ]))
        } else {
            Err(Error::NotAligned(addr))
        }
    }

    fn step(&mut self) -> Status {
        self.read_inst(self.mem.pc).and_then(|inst| self.exec(inst))
    }

    fn read_key(keypad: &mut K, delay: &mut D) -> Result<Option<u8>, Error> {
        keypad.read_key(delay).map_err(|e| e.into().into())
    }

    fn exec(&mut self, instruction: u16) -> Status {
        let cmd = instruction >> 12;
        let nnn = instruction & 0x0FFF;
        let byte = nnn as u8; // Lossy
        let nibble = byte & 0xF;
        let vx = (nnn >> 8) as u8;
        let vy = byte >> 4;

        let Mem {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
        } = &mut self.mem;

        macro_rules! pc {
            (++ if $cond: expr) => {
                if $cond {
                    *pc += Self::INST_STEP;
                }
            };

            ($($code: tt)*) => {{
                *pc = $($code)*;
                return Ok(());
            }};
        }

        macro_rules! vx {
            () => {
                reg.get(vx)?
            };

            (vf = $flag: expr) => {{
                reg.set(Self::FLAG, $flag)?;
            }};

            ($val: expr $(, vf = $flag: expr)?) => {{
                reg.set(vx, $val)?;
                $( reg.set(Self::FLAG, $flag)?; )?
            }};
        }

        macro_rules! vy {
            () => {
                reg.get(vy)?
            };
        }

        match cmd {
            // CLS
            0 if nnn == 0x0E0 => self.screen.clear().map_err(|e| e.into())?,

            // RET
            0 if nnn == 0x0EE => pc!(stack.pop()?),

            // JP addr
            1 => pc!(nnn),

            // CALL addr
            2 => {
                stack.push(*pc)?;
                pc!(nnn);
            }

            // SE Vx, byte
            3 => pc!(++ if vx!() == byte),

            // SNE Vx, byte
            4 => pc!(++ if vx!() != byte),

            // // SE Vx, Vy, 0
            5 if nibble == 0 => pc!(++ if vx!() == vy!()),

            // // LD Vx, byte
            6 => vx!(byte),

            // // ADD Vx, byte
            7 => vx!(byte.wrapping_add(vx!())),

            // // XOR
            8 => match nibble {
                // LD Vx, Vy
                0 => vx!(vy!()),

                // OR Vx, Vy
                1 => vx!(vx!() | vy!()),

                // AND Vx, Vy
                2 => vx!(vx!() & vy!()),

                // XOR Vx, Vy
                3 => vx!(vx!() ^ vy!()),

                // ADD Vx, Vy
                4 => {
                    let (x, y) = (vx!(), vy!());
                    match x.checked_add(y) {
                        Some(val) => vx!(val),
                        None => vx!(x.wrapping_add(y), vf = 1),
                    }
                }

                // SUB Vx, Vy
                5 => {
                    let (x, y) = (vx!(), vy!());
                    vx!(x.wrapping_sub(y), vf = (x > y) as u8);
                }

                // SHR Vx (, Vy)
                6 => {
                    let x = vx!();
                    vx!(x >> 1, vf = x & 1);
                }

                // SUBN Vx, Vy
                7 => {
                    let (x, y) = (vx!(), vy!());
                    vx!(y.wrapping_sub(x), vf = (y > x) as u8);
                }

                // SHL Vx (, Vy)
                0xE => {
                    let x = vx!();
                    vx!(x << 1, vf = x >> 7);
                }

                _ => return Err(Error::Instruction(instruction)),
            },

            // // SNE
            9 if nibble == 0 => pc!(++ if vx!() != vy!()),

            // // LD I, addr
            0xA => *i = nnn,

            // // JP V0, addr
            0xB => pc!(nnn + reg.get(0)? as u16),

            // // RND Vx, byte
            0xC => todo!("Rand"),

            // // DRW Vx, Vy, len
            0xD => {
                let data = ram.read_bytes(*i, nibble)?;
                let erased = self.screen.xor(vx!(), vy!(), data).map_err(|e| e.into())?;
                vx!(vf = erased as u8);
            }

            // // SKP Vx
            0xE if byte == 0x9E => match Self::read_key(&mut self.keypad, &mut self.delay)? {
                Some(key) => pc!(++ if key == vx!()),
                _ => (),
            },

            // // SKNP Vx
            0xE if byte == 0xA1 => match Self::read_key(&mut self.keypad, &mut &mut self.delay)? {
                Some(key) => pc!(++ if key != vx!()),
                _ => (),
            },

            0xF if byte == 0x07 => vx!(*dt),

            // LD Vx, K
            // All execution means what? Also stop secrementing timers?
            0xF if byte == 0x0A => {
                let key = loop {
                    if let Some(key) = Self::read_key(&mut self.keypad, &mut self.delay)? {
                        break key;
                    }

                    self.delay.delay_us(Self::POLL_FREQ).map_err(|e| e.into())?;
                };

                vx!(key);
            }

            // LD DT, Vx
            0xF if byte == 0x15 => *dt = vx!(),

            // LD ST, Vx
            0xF if byte == 0x18 => *st = vx!(),

            // ADD I, Vx
            0xF if byte == 0x1E => *i = i.wrapping_add(vx!() as u16),

            // LD F, Vx
            0xF if byte == 0x29 => *i = ram.get_sprite_addr(vx!())?,

            // LD B, Vx
            0xF if byte == 0x33 => {
                let x = vx!();
                ram.write_byte(*i, x / 100)?;
                ram.write_byte(i.saturating_add(1), (x / 10) % 10)?;
                ram.write_byte(i.saturating_add(2), x % 10)?;
            }

            // LD [I], Vx
            0xF if byte == 0x55 => {
                for loc in 0..=vx {
                    let val = reg.get(loc)?;
                    ram.write_byte(i.saturating_add(loc.into()), val)?;
                }
            }

            // Ld Vx, [I]
            0xF if byte == 0x65 => {
                for (&loc, val) in ram.read_bytes(*i, vx + 1)?.iter().zip(0..=vx) {
                    reg.set(loc, val)?;
                }
            }

            _ => Err(Error::Instruction(instruction))?,
        };

        pc!(++ if true);
        Ok(())
    }
}

impl<S, K, B, R, D> Chip8<S, K, B, R, D>
where
    S: Screen,
    K: Keypad,
    B: Buzzer,
    R: Rng,
    D: Delay,
{
    pub fn new(screen: S, keypad: K, buzzer: B, rng: R, delay: D) -> Self {
        Self::from_state(screen, keypad, buzzer, rng, delay, Mem::default())
    }

    pub fn from_state(screen: S, keypad: K, buzzer: B, rng: R, delay: D, mem: Mem) -> Self {
        Self {
            mem,
            screen,
            keypad,
            buzzer,
            rng,
            delay,
        }
    }

    pub fn state(&self) -> &Mem {
        &self.mem
    }

    pub fn free(self) -> (S, K, B, R, D, Mem) {
        let Chip8 {
            screen,
            keypad,
            buzzer,
            rng,
            delay,
            mem,
        } = self;

        (screen, keypad, buzzer, rng, delay, mem)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::Error;
    use std::vec;

    use crate::hal::{chip, ScreenCommand};

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

        assert_eq!(chip.mem.ram.read_bytes(0x200, 4).unwrap(), &[1, 2, 3, 4]);
    }

    #[test]
    fn read_instruction() {
        let mut chip = chip!();
        chip.load(&mut [0x11u16, 0x22u16, 0x33u16][..]).unwrap();

        assert_eq!(chip.read_inst(0x200).unwrap(), 0x11);
        assert_eq!(chip.read_inst(0x202).unwrap(), 0x22);
        assert_eq!(chip.read_inst(0x204).unwrap(), 0x33);
        assert_eq!(chip.read_inst(0x201).unwrap_err(), Error::NotAligned(0x201))
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
        assert_eq!(chip.mem.pc, 0x0123);

        chip.exec(0x1456).unwrap();
        assert_eq!(chip.mem.pc, 0x0456);
    }

    #[test]
    fn call() {
        let mut chip = chip!();
        chip.mem.pc = 0x0123;
        chip.exec(0x2456).unwrap();

        assert_eq!(chip.mem.pc, 0x0456);
        assert_eq!(chip.mem.stack.pop().unwrap(), 0x0123);
    }

    #[test]
    fn se_3xkk() {}
}
