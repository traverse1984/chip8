mod timer;
use timer::Timer;

use crate::vm::mem::{Load, Mem, Ram, SPRITES};

use super::error::{Error, Result};
use crate::hal::{Buzzer, Delay, Keypad, Rng, Screen};

use crate::inst::Opcode;

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
    // pub fn main<'a, P: Into<Program<'a>>>(&mut self, program: P) -> Result<u16> {
    //     self.sub(0x200, program)
    // }

    // pub fn sub<'a, P: Into<Program<'a>>>(&mut self, addr: u16, program: P) -> Result<u16> {
    //     self.data(addr, program.into().bytes())
    // }

    // pub fn data(&mut self, addr: u16, data: &[u8]) -> Result<u16> {
    //     self.mem.ram.load(addr, data)?;
    //     Ok(addr)
    // }

    pub fn screen(&mut self) -> &mut S {
        &mut self.screen
    }

    pub fn load(mut self, ram: Ram) -> Self {
        self.mem = Mem::from(ram);
        self.init();
        self
    }

    pub fn init(&mut self) {
        self.mem.pc = 0x200;
    }

    pub fn step(&mut self) -> Result {
        self.read_inst(self.mem.pc).and_then(|inst| self.exec(inst))
    }

    pub fn run(&mut self, hz: u32) -> Result {
        let tick = if hz >= 60 {
            Timer::hertz_to_us(hz).ok_or(Error::ClockSpeed(hz))
        } else {
            Err(Error::ClockSpeed(hz))
        }?;

        let mut dt = Timer::new(60).unwrap();
        let mut st = Timer::new(60).unwrap();

        loop {
            if dt.update(self.mem.dt > 0, tick) {
                self.mem.dt -= 1;
            }

            if st.update(self.mem.st > 0, tick) {
                self.mem.st -= 1;
            }

            self.step()?;
            self.delay.delay_us(tick).map_err(|e| e.into())?;
        }
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

        let opcode = match Opcode::decode(instruction) {
            Some(opcode) => opcode,
            None => return Err(Error::Instruction(instruction)),
        };

        use Opcode::*;
        match opcode {
            Cls => self.screen.clear().map_err(|e| e.into())?,
            Ret => jump!(stack.pop()? + 2),
            Jp => jump!(addr),
            Call => {
                stack.push(*pc)?;
                jump!(addr);
            }
            Se => skip!(vx == byte),
            Sne => skip!(vx != byte),
            Sev => skip!(vx == vy),
            Ld => set!(byte),
            Add => set!(byte.wrapping_add(vx)),
            Ldv => set!(vy),
            Or => set!(vx | vy),
            And => set!(vx & vy),
            Xor => set!(vx ^ vy),
            Addv => match vx.checked_add(vy) {
                Some(val) => set!(val, vf = 0),
                None => set!(vx.wrapping_add(vy), vf = 1),
            },
            Sub => set!(vx.wrapping_sub(vy), vf = (vx > vy) as u8),
            Shr => set!(vx >> 1, vf = vx & 1),
            Subn => set!(vy.wrapping_sub(vx), vf = (vy > vx) as u8),
            Shl => set!(vx << 1, vf = vx >> 7),
            Snev => skip!(vx != vy),
            Ldi => *i = addr,
            Jp0 => jump!(addr + reg.get(0)? as u16),
            Rnd => set!(byte & self.rng.random().map_err(|e| e.into())?),
            Drw => {
                let data = ram.read_bytes(*i, nibble as u16)?;
                let erased = self.screen.draw(vx, vy, data).map_err(|e| e.into())?;
                set!(vf = erased as u8);
            }
            Skp => match Self::read_key(&mut self.keypad, &mut self.delay)? {
                Some(key) => skip!(key == vx),
                _ => (),
            },
            Sknp => match Self::read_key(&mut self.keypad, &mut &mut self.delay)? {
                Some(key) => skip!(key != vx),
                _ => (),
            },
            Lddtv => set!(*dt),
            Ldkey => {
                let key = loop {
                    if let Some(key) = Self::read_key(&mut self.keypad, &mut self.delay)? {
                        break key;
                    }

                    self.delay.delay_us(POLL_FREQ).map_err(|e| e.into())?;
                };

                set!(key);
            }
            Lddt => *dt = vx,
            Ldst => *st = vx,
            Addi => *i = i.wrapping_add(vx as u16),
            Sprite => *i = ram.to_sprite_addr(vx)?,
            Bcd => {
                ram.write_byte(*i, vx / 100)?;
                ram.write_byte(i.saturating_add(1), (vx / 10) % 10)?;
                ram.write_byte(i.saturating_add(2), vx % 10)?;
            }
            Sviv => {
                for loc in 0..=vx_addr {
                    let val = reg.get(loc)?;
                    ram.write_byte(i.saturating_add(loc.into()), val)?;
                }
            }
            Ldiv => {
                for (&val, loc) in ram
                    .read_bytes(*i, vx_addr as u16 + 1)?
                    .iter()
                    .zip(0..=vx_addr)
                {
                    reg.set(loc, val)?;
                }
            }
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
