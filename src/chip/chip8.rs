use super::hw::HwChip8;

use super::clock::Clock;

use crate::mem::{Load, Mem, Ram, SPRITES};

use super::error::{Error, Result, RuntimeError, RuntimeResult};
use crate::hal::{BuzzerExt, DelayExt, Hardware, HardwareExt, KeypadExt, RngExt, ScreenExt};

use crate::inst::{bytecode::decode, Opcode};

pub(super) const POLL_FREQ: u32 = 1000;
pub(super) const INST_STEP: u16 = 2;
pub(super) const REG_FLAG: u8 = 0x0F;

#[derive(Debug, Clone, Copy)]
pub struct Chip8 {
    pub(super) mem: Mem,
    pub(super) clock_division: u32,
}

impl Chip8 {
    /// Create a new chip8 instance with empty RAM
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach hardware to the chip, simplifying some calls.
    pub fn with_hardware<H: HardwareExt>(self, hw: H) -> HwChip8<H> {
        HwChip8::from_chip(hw, self)
    }

    /// Set a clock division for the run method. Defaults to 1 (do not divide).
    /// This can be used to slow down actual execution speed.
    pub fn set_clock_division(&mut self, multiplier: u32) {
        self.clock_division = multiplier;
    }

    /// The current mem (stack, ram, registers) state.
    pub fn state(&self) -> &Mem {
        &self.mem
    }

    /// Read the next instruction and execute it with provided hardware
    pub fn step<H: HardwareExt>(&mut self, hw: &mut H) -> RuntimeResult<H::Error> {
        let inst = self.read_inst(self.mem.pc)?;
        self.exec(inst, hw)
    }

    /// Run the chip8 emulator from it's current state. The before_tick
    /// closure is executed prior to each clock "tick"
    pub fn run<H, F>(
        &mut self,
        hw: &mut H,
        speed_hz: u32,
        mut before_tick: F,
    ) -> RuntimeResult<H::Error>
    where
        H: HardwareExt,
        F: FnMut(&mut Chip8, &mut H),
    {
        let mut clock = Clock::new(speed_hz)?;

        loop {
            before_tick(self, hw);

            self.step(hw)?;

            let delay = clock.delay() * self.clock_division;
            let macro_tick = clock.tick();

            hw.delay()
                .delay_micros(delay)
                .map_err(RuntimeError::Hardware)?;

            if macro_tick {
                self.mem.dt = self.mem.dt.saturating_sub(1);
                self.mem.st = self.mem.st.saturating_sub(1);

                hw.buzzer()
                    .set_state(self.mem.st > 0)
                    .map_err(RuntimeError::Hardware)?;
            }
        }
    }

    /// Try and read the next instruction from memory
    fn read_inst(&mut self, addr: u16) -> Result<u16> {
        if self.mem.ram.to_read_addr(addr)? % INST_STEP == 0 {
            Ok(u16::from_be_bytes([
                self.mem.ram.read_byte(addr)?,
                self.mem.ram.read_byte(addr + 1)?,
            ]))
        } else {
            Err(Error::OffsetNotAligned(addr))
        }
    }

    /// Helper method for reading a key press.
    fn read_key<H: HardwareExt>(hw: &mut H) -> RuntimeResult<H::Error, Option<u8>> {
        let Hardware { keypad, delay, .. } = hw.hardware();

        keypad.read_key(delay).map_err(RuntimeError::Hardware)
    }

    /// Execute an instruction with provided hardware and update state.
    pub(super) fn exec<H: HardwareExt>(
        &mut self,
        inst: u16,
        hw: &mut H,
    ) -> RuntimeResult<H::Error> {
        let addr = decode::addr(inst);
        let byte = decode::byte(inst);
        let vx_reg = decode::vx(inst);
        let vy_reg = decode::vy(inst);

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

        /// Set the program counter based on the provided expression, and then
        /// return from this function.
        macro_rules! jump {
            ($($val: tt)+) => {{
                *pc = $($val)+;
                return Ok(());
            }};
        }

        /// If the condition is true, increment the program counter to the next
        /// instruction address.
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
            None => return Err(Error::UnknownInstruction(inst))?,
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

                    hw.delay()
                        .delay_micros(POLL_FREQ)
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

impl Default for Chip8 {
    fn default() -> Self {
        Self {
            mem: Mem::default(),
            clock_division: 1,
        }
    }
}

impl From<Mem> for Chip8 {
    fn from(mem: Mem) -> Self {
        Self {
            mem,
            ..Default::default()
        }
    }
}

impl From<Ram> for Chip8 {
    fn from(ram: Ram) -> Self {
        let mem = Mem::from(ram);
        Self::from(mem)
    }
}
