mod timer;
use core::ops::{Deref, DerefMut};

use timer::Timer;

use crate::vm::mem::{Load, Mem, Ram, SPRITES};

use super::error::{Error, Result, RuntimeError, RuntimeResult};
use crate::hal::{BuzzerExt, Hardware, HardwareExt, KeypadExt, RngExt, ScreenExt, TimerExt};

use crate::inst::{bytecode::decode, Opcode};

#[cfg(test)]
mod tests;

const POLL_FREQ: u32 = 1000;
const INST_STEP: u16 = 2;
const REG_FLAG: u8 = 0x0F;

pub struct Chip8WithHardware<H: HardwareExt> {
    chip: Chip8,
    hw: H,
}

impl<H: HardwareExt> HardwareExt for Chip8WithHardware<H> {
    type Error = H::Error;
    type Timer = H::Timer;
    type Screen = H::Screen;
    type Keypad = H::Keypad;
    type Buzzer = H::Buzzer;
    type Rng = H::Rng;

    fn hardware(
        &mut self,
    ) -> Hardware<'_, Self::Timer, Self::Screen, Self::Keypad, Self::Buzzer, Self::Rng> {
        self.hw.hardware()
    }
}

impl<H: HardwareExt> Chip8WithHardware<H> {
    pub fn new(hw: H) -> Self {
        Self {
            chip: Chip8::new(),
            hw,
        }
    }

    pub fn from_state(hw: H, mem: Mem) -> Self {
        Self {
            chip: Chip8::from_state(mem),
            hw,
        }
    }

    pub fn hw(&mut self) -> &mut H {
        &mut self.hw
    }

    pub fn run(&mut self, hz: u32) -> RuntimeResult<H::Error> {
        let Self { chip, hw } = self;
        chip.run(hz, hw)
    }

    pub fn step(&mut self) -> RuntimeResult<H::Error> {
        let Self { chip, hw } = self;
        chip.step(hw)
    }

    pub fn exec(&mut self, inst: u16) -> RuntimeResult<H::Error> {
        let Self { chip, hw } = self;
        chip.exec(inst, hw)
    }
}

impl<H: HardwareExt> Deref for Chip8WithHardware<H> {
    type Target = Chip8;
    fn deref(&self) -> &Self::Target {
        &self.chip
    }
}

impl<H: HardwareExt> DerefMut for Chip8WithHardware<H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chip
    }
}

pub struct Chip8 {
    mem: Mem,
}

impl Chip8 {
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

    pub fn load(mut self, ram: Ram) -> Self {
        self.mem = Mem::from(ram);
        self.init();
        self
    }

    pub fn init(&mut self) {
        self.mem.pc = 0x200;
    }

    pub fn step<H: HardwareExt>(&mut self, hw: &mut H) -> RuntimeResult<H::Error> {
        let inst = self.read_inst(self.mem.pc)?;
        self.exec(inst, hw)
    }

    pub fn run<H: HardwareExt>(&mut self, hz: u32, hw: &mut H) -> RuntimeResult<H::Error> {
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

            self.step(hw)?;
            hw.timer().delay_us(tick).map_err(RuntimeError::Hardware)?;
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

    fn read_key<H: HardwareExt>(hw: &mut H) -> RuntimeResult<H::Error, Option<u8>> {
        let Hardware { keypad, timer, .. } = hw.hardware();

        keypad.read_key(timer).map_err(RuntimeError::Hardware)
    }

    fn exec<H: HardwareExt>(&mut self, inst: u16, hw: &mut H) -> RuntimeResult<H::Error> {
        let addr = decode::addr(inst);
        let vx_reg = decode::vx(inst);
        let vy_reg = decode::vy(inst);
        let byte = decode::byte(inst);

        let Mem {
            i,
            pc,
            dt,
            st,
            reg,
            stack,
            ram,
        } = &mut self.mem;

        let vx = reg.get(vx_reg)?;
        let vy = reg.get(vy_reg)?;

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
                reg.set(vx_reg, $val)?;
                $( reg.set(REG_FLAG, $flag)?; )?
            }};
        }

        let opcode = match Opcode::decode(inst) {
            Some(opcode) => opcode,
            None => return Err(Error::Instruction(inst))?,
        };

        use Opcode::*;
        match opcode {
            Cls => hw.screen().clear().map_err(RuntimeError::Hardware)?,
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
            Rnd => set!(byte & hw.rng().rand().map_err(RuntimeError::Hardware)?),
            Drw => {
                let data = ram.read_bytes(*i, decode::nibble(inst) as u16)?;
                let erased = hw
                    .screen()
                    .draw(vx, vy, data)
                    .map_err(RuntimeError::Hardware)?;
                set!(vf = erased as u8);
            }
            Skp => match Self::read_key(hw)? {
                Some(key) => skip!(key == vx),
                _ => (),
            },
            Sknp => match Self::read_key(hw)? {
                Some(key) => skip!(key != vx),
                _ => (),
            },
            Lddtv => set!(*dt),
            Ldkey => {
                let key = loop {
                    if let Some(key) = Self::read_key(hw)? {
                        break key;
                    }

                    hw.timer()
                        .delay_us(POLL_FREQ)
                        .map_err(RuntimeError::Hardware)?;
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
                for loc in 0..=vx_reg {
                    let val = reg.get(loc)?;
                    ram.write_byte(i.saturating_add(loc.into()), val)?;
                }
            }
            Ldiv => {
                for (&val, loc) in ram
                    .read_bytes(*i, vx_reg as u16 + 1)?
                    .iter()
                    .zip(0..=vx_reg)
                {
                    reg.set(loc, val)?;
                }
            }
        };

        skip!(true);
        Ok(())
    }
}

impl Chip8 {
    pub fn new() -> Self {
        Self {
            mem: Mem::default(),
        }
    }

    pub fn from_state(mem: Mem) -> Self {
        Self { mem }
    }

    pub fn state(&self) -> &Mem {
        &self.mem
    }
}
