use super::{sprites::SPRITES, Error, Result};

pub trait Load<T> {
    fn load(&mut self, addr: u16, words: &[T]) -> Result<u16>;
}

#[derive(Debug, Copy, Clone)]
pub struct Ram {
    mem: [u8; 4096],
}

impl Default for Ram {
    fn default() -> Self {
        let mut ram = Self { mem: [0; 4096] };
        let sprites_loc = &mut ram.mem[0x1B0..0x200];
        let sprites = unsafe { core::mem::transmute::<[[u8; 5]; 16], [u8; 80]>(SPRITES) };
        sprites_loc.copy_from_slice(&sprites);
        ram
    }
}

impl Load<u8> for Ram {
    fn load(&mut self, addr: u16, bytes: &[u8]) -> Result<u16> {
        let index = self.to_read_addr(addr)? as usize;

        if bytes.len() > 4096 - index {
            return Err(Error::LoadTooLong {
                addr,
                len: bytes.len(),
            });
        }

        let target = &mut self.mem[index..index + bytes.len()];
        target.copy_from_slice(&bytes);

        Ok(bytes.len() as u16)
    }
}

impl Load<u16> for Ram {
    fn load(&mut self, addr: u16, words: &[u16]) -> Result<u16> {
        let mut index = self.to_read_addr(addr)? as usize;

        if words.len() * 2 > 4096 - index {
            return Err(Error::LoadTooLong {
                addr,
                len: words.len() * 2,
            });
        }

        for [msb, lsb] in words.iter().map(|word| word.to_be_bytes()) {
            self.mem[index] = msb;
            self.mem[index + 1] = lsb;
            index += 2;
        }

        Ok((words.len() * 2) as u16)
    }
}

impl Ram {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_read_addr(&self, addr: u16) -> Result<u16> {
        match addr {
            0..=0xFFF => Ok(addr),
            _ => Err(Error::InvalidAddress { addr }),
        }
    }

    pub fn to_write_addr(&self, addr: u16) -> Result<u16> {
        match addr {
            0x200.. => self.to_read_addr(addr),
            _ => Err(Error::NotWritable { addr }),
        }
    }

    pub fn read_byte(&self, addr: u16) -> Result<u8> {
        self.to_read_addr(addr).map(|loc| self.mem[loc as usize])
    }

    pub fn read_bytes(&self, addr: u16, len: u16) -> Result<&[u8]> {
        match (addr, addr + len) {
            (0..=0xFFF, end @ 0..=0xFFF) => Ok(&self.mem[addr as usize..end as usize]),
            _ => Err(Error::InvalidSlice { addr, len }),
        }
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) -> Result {
        self.to_write_addr(addr)
            .map(|loc| self.mem[loc as usize] = data)
    }

    pub fn write_bytes(&mut self, addr: u16, data: &[u8]) -> Result {
        self.load(self.to_write_addr(addr)?, data)?;
        Ok(())
    }

    pub fn to_sprite_addr(&self, sprite: u8) -> Result<u16> {
        if sprite < 16 {
            Ok(0x1B0 + sprite as u16 * 5)
        } else {
            Err(Error::InvalidSprite { sprite })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use super::Load;
    use super::Ram;

    #[test]
    fn load() {
        let mut ram = Ram::new();

        ram.load(0x100, &[1u8, 2, 3]).unwrap();
        assert_eq!(ram.read_bytes(0x100, 3).unwrap(), &[1, 2, 3]);

        ram.load(0x200, &[4u8, 5, 6]).unwrap();
        assert_eq!(ram.read_bytes(0x200, 3).unwrap(), &[4, 5, 6]);

        ram.load(0x300, &[0x0102u16, 0x0304]).unwrap();
        assert_eq!(ram.read_bytes(0x300, 4).unwrap(), &[0x1, 0x2, 0x3, 0x4]);

        assert_eq!(
            ram.load(0x1000, &[1u8, 2, 3]).unwrap_err(),
            Error::InvalidAddress { addr: 0x1000 }
        );

        let too_long = [0u8; 1024];
        assert_eq!(
            ram.load(0xF00, &too_long).unwrap_err(),
            Error::LoadTooLong {
                addr: 0xF00,
                len: 1024
            }
        )
    }

    #[test]
    fn sprites() {
        let ram = Ram::new();
        assert_eq!(&ram.mem[0x1B0..0x1B5], [0xF0, 0x90, 0x90, 0x90, 0xF0]);
        assert_eq!(&ram.mem[0x1FB..0x200], [0xF0, 0x80, 0xF0, 0x80, 0x80]);
    }
}
