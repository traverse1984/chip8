use super::{Error, Result};

#[derive(Debug, Copy, Clone)]
pub struct Stack {
    sp: u8,
    frames: [u16; 16],
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            sp: 0xFF,
            frames: [0; 16],
        }
    }
}

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn depth(&self) -> usize {
        self.sp as usize + 1
    }

    pub fn push(&mut self, frame: u16) -> Result {
        match self.sp {
            0..=14 | 0xFF => {
                self.sp = self.sp.overflowing_add(1).0;
                self.frames[self.sp as usize] = frame;
                Ok(())
            }
            15 => Err(Error::StackOverflow { frame }),
            _ => Err(Error::StackCorrupt { sp: self.sp }),
        }
    }

    pub fn pop(&mut self) -> Result<u16> {
        match self.sp {
            0..=15 => {
                let ptr = self.frames[self.sp as usize];
                self.sp = self.sp.overflowing_sub(1).0;
                Ok(ptr)
            }
            0xFF => Err(Error::StackEmpty),
            _ => Err(Error::StackCorrupt { sp: self.sp }),
        }
    }
}
