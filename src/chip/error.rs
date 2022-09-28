use core::convert::Infallible;

use crate::{hal, mem};

pub type Result<T = ()> = core::result::Result<T, Error>;
pub type RuntimeResult<E, T = ()> = core::result::Result<T, RuntimeError<E>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeError<E> {
    Hardware(E),
    Software(Error),
}

/// Chip8 Software errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// An error relating to the memory module
    Memory(mem::Error),

    /// The address was not aligned to the start of an instruction
    OffsetNotAligned(u16),

    /// The instruction is not known
    UnknownInstruction(u16),

    /// The requested clock speed can't be emulated
    InvalidClockSpeed(u32),
}

impl From<mem::Error> for Error {
    fn from(err: mem::Error) -> Self {
        Error::Memory(err)
    }
}

impl<E> From<Error> for RuntimeError<E> {
    fn from(err: Error) -> Self {
        RuntimeError::Software(err)
    }
}

impl<E> From<mem::Error> for RuntimeError<E> {
    fn from(err: mem::Error) -> Self {
        RuntimeError::Software(Error::Memory(err))
    }
}
