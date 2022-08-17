use core::ops::Sub;

use crate::chip8_asm;

use crate::vm::mem::{Error, Load, Ram, Result};

#[derive(Debug, Clone, Copy)]
pub enum Marker {
    Reserved,
    Used,
    Free,
    Call(u16),
    Jp(u16),
    Jp0(u16),
    Ldi(u16),
}

#[derive(Debug, Clone, Copy)]
pub struct Subroutine<'a> {
    id: u16,
    addr: u16,
    instructions: &'a [u16],
}

impl<'a> Subroutine<'a> {
    pub fn new(id: u16, instructions: &'a [u16]) -> Self {
        Self {
            id,
            addr: 4096,
            instructions,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Var<'a> {
    id: u16,
    addr: u16,
    data: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct Program<'a> {
    ram: Ram,
    main: &'a [u16],
    mask: [Marker; 4096],
    sub: [Option<Subroutine<'a>>; 32],
    sub_ptr: usize,
    var: [Option<Var<'a>>; 64],
    var_ptr: usize,
}

impl<'a> Default for Program<'a> {
    fn default() -> Self {
        // let mut mask = [Marker::Free; 4096];
        // let reserved = &mut mask[0..0x200];
        // reserved.copy_from_slice(&[Marker::Reserved; 512]);

        Self {
            main: &[],
            ram: Ram::default(),
            sub: [None; 32],
            sub_ptr: 0,
            var: [None; 64],
            var_ptr: 32,
            mask: [Marker::Free; 4096],
        }
    }
}

impl<'a> Program<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn main(&mut self, instructions: &'a [u16]) {
        self.main = instructions;
    }

    pub fn sub(&mut self, instructions: &'a [u16]) -> Option<u16> {
        self.sub
            .get_mut(self.sub_ptr)?
            .replace(Subroutine::new(self.sub_ptr as u16, instructions));

        let sub_ptr = self.sub_ptr;
        self.sub_ptr += 1;
        Some(sub_ptr as u16)
    }

    pub fn var(&mut self, data: &'a u8) -> Option<u16> {
        self.data(unsafe { *(data as *const _ as *const &[u8]) })
    }

    pub fn data(&mut self, data: &'a [u8]) -> Option<u16> {
        self.var.get_mut(self.var_ptr - 32)?.replace(Var {
            id: self.var_ptr as u16,
            addr: 4096,
            data,
        });

        let var_ptr = self.var_ptr;
        self.var_ptr += 1;
        Some(var_ptr as u16)
    }

    pub fn compile(&mut self) -> &[u8] {
        extern crate std;
        use std::println;

        let mut addr = 0x200;
        let ram = &mut self.ram;

        addr += ram.load(addr, self.main).unwrap() as u16;
        for sub in self.sub.iter_mut().flatten() {
            sub.addr = addr;
            addr += ram.load(addr, sub.instructions).unwrap() as u16;
        }

        let last_inst = addr;

        for var in self.var.iter_mut().flatten() {
            var.addr = addr;
            addr += ram.load(addr, var.data).unwrap() as u16;
        }

        for addr in (0x200..last_inst).filter(|idx| idx % 2 == 0) {
            match ram.read_bytes(addr, 2).unwrap() {
                [i @ 0x10 | i @ 0x20 | i @ 0xA0 | i @ 0xB0, id @ 0..=95] => {
                    let id = *id as usize;
                    let [msb, lsb] = if id < 32 {
                        self.sub[id].unwrap().addr
                    } else {
                        self.var[id - 32].unwrap().addr
                    }
                    .to_be_bytes();

                    let i = *i;
                    ram.write_bytes(addr, &[i | msb, lsb]).unwrap()
                }
                _ => continue,
            }
        }

        ram.read_bytes(0x200, 32).unwrap()
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
        };

        let sub = program.sub(&sub_inst).unwrap();
        let data = program.data(&[1, 2, 3, 4]).unwrap();

        println!("sub={sub}, data={data}");

        let prog = chip8_asm! {
            call sub;
            ldi data;
        };

        program.main(&prog);

        dbg!(program.compile());
    }
}
