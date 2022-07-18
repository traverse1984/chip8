use super::{Error, Result};

#[derive(Debug, Copy, Clone)]
pub struct Ram {
    mem: [u8; 4096],
}

impl Ram {
    pub fn new() -> Self {
        let mut mem = [0; 4096];

        let sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];

        (&mut mem[0x1B0..0x200]).copy_from_slice(&sprites);

        Self { mem }
    }

    pub fn load(&mut self, addr: u16, bytes: &[u8]) -> Result {
        let index = self.to_read_addr(addr)? as usize;

        if bytes.len() <= 4096 - index {
            let target = &mut self.mem[index..index + bytes.len()];
            target.copy_from_slice(&bytes);
            Ok(())
        } else {
            Err(Error::LoadTooLong {
                addr,
                len: bytes.len(),
            })
        }
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

    pub fn read_bytes(&self, addr: u16, len: u8) -> Result<&[u8]> {
        match (addr, addr + len as u16) {
            (0..=0xFFF, end @ 0..=0xFFF) => Ok(&self.mem[addr as usize..end as usize]),
            _ => Err(Error::InvalidSlice { addr, len }),
        }
    }

    pub fn write_byte(&mut self, addr: u16, data: u8) -> Result {
        self.to_write_addr(addr)
            .map(|loc| self.mem[loc as usize] = data)
    }

    pub fn get_sprite_addr(&self, sprite: u8) -> Result<u16> {
        if sprite < 16 {
            Ok(0x1B0 + sprite as u16 * 5)
        } else {
            Err(Error::InvalidSprite { sprite })
        }
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self { mem: [0; 4096] }
    }
}

mod tests {
    use super::Error;
    use super::Ram;

    #[test]
    fn load() {
        let mut ram = Ram::new();

        ram.load(0x100, &[1, 2, 3]).unwrap();
        assert_eq!(ram.read_bytes(0x100, 3).unwrap(), &[1, 2, 3]);

        ram.load(0x200, &[4, 5, 6]).unwrap();
        assert_eq!(ram.read_bytes(0x200, 3).unwrap(), &[4, 5, 6]);

        assert_eq!(
            ram.load(0x1000, &[1, 2, 3]).unwrap_err(),
            Error::InvalidAddress { addr: 0x1000 }
        );

        let too_long = [0; 1024];
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
