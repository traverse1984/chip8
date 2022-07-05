use super::mem;
use crate::pal;

pub type Status = Result<(), Error>;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    Peripheral(pal::Error),
    Memory(mem::Error),
    Instruction(u16),
}

impl From<pal::Error> for Error {
    fn from(err: pal::Error) -> Self {
        Error::Peripheral(err)
    }
}

impl From<mem::Error> for Error {
    fn from(err: mem::Error) -> Self {
        Error::Memory(err)
    }
}
