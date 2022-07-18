use super::chip8::mem;
use crate::hal;

pub type Result<T = ()> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Peripheral(hal::Error),
    Memory(mem::Error),
    NotAligned(u16),
    Instruction(u16),
}

impl From<hal::Error> for Error {
    fn from(err: hal::Error) -> Self {
        Error::Peripheral(err)
    }
}

impl From<mem::Error> for Error {
    fn from(err: mem::Error) -> Self {
        Error::Memory(err)
    }
}
