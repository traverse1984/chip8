use crate::chip8_asm;
extern crate std;
use super::error::{CompileError, Result};
use super::refs::{Ref, Refs};
use crate::inst::ops;
use crate::mem::{Error, Load, Ram};
use std::{dbg, print, println};

pub struct CompiledProgram {
    pub ram: Ram,
}

#[derive(Debug, Clone)]
pub struct Program {
    tmp: Ram,
    addr: u16,
    main: Option<Ref>,
    subs: Refs<{ Self::SUBROUTINES }>,
    vars: Refs<{ Self::VARS }>,
}

impl Load<u8> for Program {
    fn load(&mut self, addr: u16, words: &[u8]) -> crate::mem::Result<u16> {
        self.tmp.load(addr, words)
    }
}

impl Load<u16> for Program {
    fn load(&mut self, addr: u16, words: &[u16]) -> crate::mem::Result<u16> {
        self.tmp.load(addr, words)
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            tmp: Ram::new(),
            addr: 0x200,
            main: None,
            subs: Refs::new(),
            vars: Refs::new(),
        }
    }
}

impl Program {
    const SUBROUTINES: usize = 32;
    const VARS: usize = 64;

    pub fn new() -> Self {
        Self::default()
    }

    fn add_addr(&mut self, len: usize) {
        self.addr += len as u16;
    }

    fn load_to_tmp<T>(&mut self, data: &[T]) -> Result<Ref>
    where
        Self: Load<T>,
    {
        let addr = self.addr;
        let len = self.load(addr, data)?;
        self.addr += len;
        Ok(Ref::new(addr, len))
    }

    fn load_program_data<T>(&mut self, program: &[T]) -> Result<Ref>
    where
        Self: Load<T>,
    {
        let refr = self.load_to_tmp(program)?;

        if refr.is_aligned() {
            Ok(refr)
        } else {
            Err(CompileError::Todo)
        }
    }

    pub fn main<T>(&mut self, program: &[T]) -> Result
    where
        Self: Load<T>,
    {
        let main_ref = self.load_program_data(program)?;
        self.main.replace(main_ref);
        Ok(())
    }

    pub fn sub<T>(&mut self, program: &[T]) -> Result<u16>
    where
        Self: Load<T>,
    {
        let sub_ref = self.load_program_data(program)?;
        self.subs.create(sub_ref)
    }

    pub fn repeat<T>(&mut self, program: &[T]) -> Result<u16>
    where
        Self: Load<T> + Load<u16>,
    {
        let Ref { addr, len } = self.load_program_data(program)?;
        let next_ref = self.subs.peek_next_id();
        let jp_ref = self.load_to_tmp(&[ops::jp(next_ref)])?;
        self.subs.create(Ref::new(addr, len + jp_ref.len))
    }

    pub fn data(&mut self, data: &[u8]) -> Result<u16> {
        let refr = self.load_to_tmp(data)?;
        let id = self.vars.create(refr)?;
        Ok(id + Self::SUBROUTINES as u16)
    }

    pub fn var(&mut self, data: u8) -> Result<u16> {
        self.data(&[data])
    }

    pub fn compile(mut self) -> Result<CompiledProgram> {
        let mut addr = 0x200;
        let mut ram = Ram::new();

        let mut main = self.main.ok_or(Error::StackEmpty)?;
        let main = main.read_and_update(0x200, &self.tmp)?;
        addr += ram.load(addr, main)?;

        //println!("Add subroutines at {addr}");
        self.subs.copy(&self.tmp, &mut ram, &mut addr)?;

        let last_inst = addr;
        //println!("Last instruction at {last_inst}");

        //println!("Add vars at {addr}");
        self.vars.copy(&self.tmp, &mut ram, &mut addr)?;

        // @todo Improve this implementation so it doesn't use
        // magic numbers.
        for addr in (0x200..last_inst).filter(|idx| idx % 2 == 0) {
            match ram.read_bytes(addr, 2)? {
                &[inst @ (0x10 | 0x20 | 0xA0 | 0xB0), id]
                    if id < (Self::SUBROUTINES + Self::VARS) as u8 =>
                {
                    let refr = if id < Self::SUBROUTINES as u8 {
                        self.subs.get(id as u16)
                    } else {
                        let adjusted_id = id - Self::SUBROUTINES as u8;
                        self.vars.get(adjusted_id as u16)
                    };

                    let [msb, lsb] = refr?.to_be_bytes();
                    ram.write_bytes(addr, &[inst | msb, lsb])?;
                }
                _ => continue,
            }
        }

        Ok(CompiledProgram { ram })
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{dbg, println};
    #[test]
    fn test() {
        let mut program = Program::new();

        let sub_inst = chip8_asm! {
            ld 2, 22;
            add 3, 1;
            ret;
        };

        let sub = program.sub(&sub_inst).unwrap();
        let data = program.data(&[1, 2, 3, 4]).unwrap();

        let prog = chip8_asm! {
            call sub;
            ldi data;
            add 1, 2;
            add 3, 4;
        };

        program.main(&prog).unwrap();

        let prog = program.compile().unwrap();

        let prog = prog.ram.read_bytes(0x200, 32).unwrap();
        hexdump(prog);
    }
}

pub fn hexdump(byt: &[u8]) {
    for (i, b) in byt.iter().enumerate() {
        if i % 8 == 0 {
            //println!("");
        }
        //print!("{:02x} ", b);
    }
    //println!("\n");
}
