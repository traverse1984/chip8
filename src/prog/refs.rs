use super::error::{CompileError, Result};
use crate::mem::{Load, Ram};

#[derive(Debug, Clone, Copy)]
pub struct Ref {
    pub(super) addr: u16,
    pub(super) len: u16,
}

impl Ref {
    pub(super) fn new(addr: u16, len: u16) -> Self {
        Self { addr, len }
    }

    pub(super) fn is_aligned(&self) -> bool {
        self.len % 2 == 0
    }

    pub(super) fn read_and_update<'a>(&'a mut self, addr: u16, ram: &'a Ram) -> Result<&[u8]> {
        let bytes = ram.read_bytes(self.addr, self.len)?;
        self.addr = addr;
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Refs<const LEN: usize> {
    index: usize,
    refs: [Option<Ref>; LEN],
}

impl<const LEN: usize> Default for Refs<LEN> {
    fn default() -> Self {
        Self {
            index: 0,
            refs: [None; LEN],
        }
    }
}

impl<const LEN: usize> Refs<LEN> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn peek_next_id(&self) -> u16 {
        self.index as u16
    }

    pub fn create(&mut self, new_ref: Ref) -> Result<u16> {
        let index = self.index;
        match self.refs.get_mut(index) {
            Some(refr) => {
                refr.replace(new_ref);
                self.index += 1;
                Ok(index as u16)
            }
            None => Err(CompileError::Todo),
        }
    }

    pub fn copy(&mut self, src: &Ram, dest: &mut Ram, addr: &mut u16) -> Result {
        for refr in self.refs.iter_mut().flatten() {
            //print!("  * Ref = ({}, {})", refr.addr, refr.len);
            let bytes = refr.read_and_update(*addr, src)?;
            //println!(" len={}", bytes.len());
            *addr += dest.load(*addr, bytes)?;
        }
        Ok(())
    }

    pub fn get(&self, id: u16) -> Result<u16> {
        match self.refs.get(id as usize) {
            Some(Some(refr)) => Ok(refr.addr),
            Some(None) => Err(CompileError::Todo),
            None => Err(CompileError::Todo),
        }
    }
}
