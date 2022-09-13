use super::{Ram, Registers, Stack};

pub type Result<T = ()> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidAddress { addr: u16 },
    InvalidSlice { addr: u16, len: u16 },
    InvalidSprite { sprite: u8 },
    InvalidRegister { reg: u8 },
    NotWritable { addr: u16 },
    NotAligned { pc: u16 },
    LoadTooLong { addr: u16, len: usize },
    StackOverflow { frame: u16 },
    StackCorrupt { sp: u8 },
    StackEmpty,
}

#[derive(Debug, Copy, Clone, Default)]
pub struct Mem {
    pub i: u16,
    // Counter
    pub pc: u16,
    // Delay timer
    pub dt: u8,
    // Sound timer
    pub st: u8,
    // General purpose
    pub reg: Registers,
    // Ram
    pub stack: Stack,
    // Usually mem addr
    pub ram: Ram,
    // Stack
}

impl From<Ram> for Mem {
    fn from(ram: Ram) -> Self {
        Mem {
            ram,
            ..Default::default()
        }
    }
}
