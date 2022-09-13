use crate::mem;

#[derive(Debug)]
pub enum CompileError {
    Todo,
    Memory(mem::Error),
}

impl From<mem::Error> for CompileError {
    fn from(err: mem::Error) -> Self {
        CompileError::Memory(err)
    }
}

pub type Result<T = ()> = core::result::Result<T, CompileError>;
