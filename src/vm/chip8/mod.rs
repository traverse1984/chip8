pub mod mem;

use mem::{Mem, Ram};

use super::error::{Error, Result};
use super::program::Program;
use crate::hal::{Buzzer, Delay, Keypad, Rng, Screen};

#[cfg(test)]
mod tests;

const POLL_FREQ: u32 = 1000;
const INST_STEP: u16 = 2;
const REG_FLAG: u8 = 0x0F;

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
    pub fn main<'a, P: Into<Program<'a>>>(&mut self, program: P) -> Result<u16> {
        self.sub(0x200, program)
    }

    pub fn sub<'a, P: Into<Program<'a>>>(&mut self, addr: u16, program: P) -> Result<u16> {
        self.data(addr, program.into().bytes())
    }

    pub fn data(&mut self, addr: u16, data: &[u8]) -> Result<u16> {
        self.mem.ram.load(addr, data)?;
        Ok(addr)
    }

    pub fn init(&mut self) -> Result {
        self.mem.pc = 0x200;
        Ok(())
    }

    pub fn run(&mut self) -> Result {
        loop {
            self.step()?;
        }
    }

    pub fn step(&mut self) -> Result {
        self.read_inst(self.mem.pc).and_then(|inst| self.exec(inst))
    }

    fn read_inst(&mut self, addr: u16) -> Result<u16> {
        if self.mem.ram.to_read_addr(addr)? % INST_STEP == 0 {
            Ok(u16::from_be_bytes([
                self.mem.ram.read_byte(addr)?,
                self.mem.ram.read_byte(addr + 1)?,
            ]))
        } else {
            Err(Error::NotAligned(addr))
        }
    }

    fn read_key(keypad: &mut K, delay: &mut D) -> Result<Option<u8>> {
        keypad.read_key(delay).map_err(|e| e.into().into())
    }

    fn exec(&mut self, instruction: u16) -> Result {
        let cmd = instruction >> 12;
        let addr = instruction & 0x0FFF;
        let byte = addr as u8;
        let nibble = byte & 0xF;
        let vx_addr = (addr >> 8) as u8;

        let Mem {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
        } = &mut self.mem;

        let vx = reg.get(vx_addr)?;
        let vy = reg.get(byte >> 4)?;

        /// Set or increment the program counter
        macro_rules! jump {
            ($($code: tt)*) => {{
                *pc = $($code)*;
                return Ok(());
            }};
        }

        macro_rules! skip {
            ($($cond: tt)+) => {
                if $($cond)+ {
                    *pc += INST_STEP;
                }
            };
        }

        /// Set the `vx` and flag registers
        macro_rules! set {
            (vf = $flag: expr) => {{
                reg.set(REG_FLAG, $flag)?;
            }};

            ($val: expr $(, vf = $flag: expr)?) => {{
                reg.set(vx_addr, $val)?;
                $( reg.set(REG_FLAG, $flag)?; )?
            }};
        }

        match cmd {
            // CLS
            0 if addr == 0x0E0 => self.screen.clear().map_err(|e| e.into())?,

            // RET
            0 if addr == 0x0EE => jump!(stack.pop()?),

            // JP addr
            1 => jump!(addr),

            // CALL addr
            2 => {
                stack.push(*pc)?;
                jump!(addr);
            }

            // SE Vx, byte
            3 => skip!(vx == byte),

            // SNE Vx, byte
            4 => skip!(vx != byte),

            // // SE Vx, Vy, 0
            5 if nibble == 0 => skip!(vx == vy),

            // // LD Vx, byte
            6 => set!(byte),

            // // ADD Vx, byte
            7 => set!(byte.wrapping_add(vx)),

            // // XOR
            8 => match nibble {
                // LD Vx, Vy
                0 => set!(vy),

                // OR Vx, Vy
                1 => set!(vx | vy),

                // AND Vx, Vy
                2 => set!(vx & vy),

                // XOR Vx, Vy
                3 => set!(vx ^ vy),

                // ADD Vx, Vy
                4 => match vx.checked_add(vy) {
                    Some(val) => set!(val, vf = 0),
                    None => set!(vx.wrapping_add(vy), vf = 1),
                },

                // SUB Vx, Vy
                5 => set!(vx.wrapping_sub(vy), vf = (vx > vy) as u8),

                // SHR Vx (, Vy)
                6 => set!(vx >> 1, vf = vx & 1),

                // SUBN Vx, Vy
                7 => set!(vy.wrapping_sub(vx), vf = (vy > vx) as u8),

                // SHL Vx (, Vy)
                0xE => set!(vx << 1, vf = vx >> 7),

                _ => return Err(Error::Instruction(instruction)),
            },

            // // SNE
            9 if nibble == 0 => skip!(vx != vy),

            // // LD I, addr
            0xA => *i = addr,

            // // JP V0, addr
            0xB => jump!(addr + reg.get(0)? as u16),

            // // RND Vx, byte
            0xC => set!(byte & self.rng.random().map_err(|e| e.into())?),

            // // DRW Vx, Vy, len
            0xD => {
                let data = ram.read_bytes(*i, nibble)?;
                let erased = self.screen.draw(vx, vy, data).map_err(|e| e.into())?;
                set!(vf = erased as u8);
            }

            // // SKP Vx
            0xE if byte == 0x9E => match Self::read_key(&mut self.keypad, &mut self.delay)? {
                Some(key) => skip!(key == vx),
                _ => (),
            },

            // // SKNP Vx
            0xE if byte == 0xA1 => match Self::read_key(&mut self.keypad, &mut &mut self.delay)? {
                Some(key) => skip!(key != vx),
                _ => (),
            },

            0xF if byte == 0x07 => set!(*dt),

            // LD Vx, K
            // All execution means what? Also stop secrementing timers?
            0xF if byte == 0x0A => {
                let key = loop {
                    if let Some(key) = Self::read_key(&mut self.keypad, &mut self.delay)? {
                        break key;
                    }

                    self.delay.delay_us(POLL_FREQ).map_err(|e| e.into())?;
                };

                set!(key);
            }

            // LD DT, Vx
            0xF if byte == 0x15 => *dt = vx,

            // LD ST, Vx
            0xF if byte == 0x18 => *st = vx,

            // ADD I, Vx
            0xF if byte == 0x1E => *i = i.wrapping_add(vx as u16),

            // LD F, Vx
            0xF if byte == 0x29 => *i = ram.get_sprite_addr(vx)?,

            // LD B, Vx
            0xF if byte == 0x33 => {
                ram.write_byte(*i, vx / 100)?;
                ram.write_byte(i.saturating_add(1), (vx / 10) % 10)?;
                ram.write_byte(i.saturating_add(2), vx % 10)?;
            }

            // LD [I], Vx
            0xF if byte == 0x55 => {
                for loc in 0..=vx_addr {
                    let val = reg.get(loc)?;
                    ram.write_byte(i.saturating_add(loc.into()), val)?;
                }
            }

            // Ld Vx, [I]
            0xF if byte == 0x65 => {
                for (&val, loc) in ram.read_bytes(*i, vx_addr + 1)?.iter().zip(0..=vx_addr) {
                    reg.set(loc, val)?;
                }
            }

            _ => Err(Error::Instruction(instruction))?,
        };

        skip!(true);
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
