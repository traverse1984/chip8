use super::{Error, Result};

#[derive(Debug, Copy, Clone, Default)]
pub struct Registers {
    reg: [u8; 16],
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, reg: u8) -> Result<u8> {
        if reg < 16 {
            Ok(self.reg[reg as usize])
        } else {
            Err(Error::InvalidRegister { reg })
        }
    }

    pub fn set(&mut self, reg: u8, val: u8) -> Result {
        if reg < 16 {
            Ok(self.reg[reg as usize] = val)
        } else {
            Err(Error::InvalidRegister { reg })
        }
    }
}
