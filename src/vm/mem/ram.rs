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

    fn is_address(addr: u16) -> bool {
        addr < 0x1000
    }

    pub fn load(&mut self, addr: usize, bytes: &[u8]) -> Result {
        if bytes.len() <= 4096 - addr {
            let target = &mut self.mem[addr..addr + bytes.len()];
            target.copy_from_slice(&bytes);
            Ok(())
        } else {
            Err(Error::LoadTooLong {
                addr,
                len: bytes.len(),
            })
        }
    }

    pub fn to_valid_address(&self, addr: u16) -> Result<u16> {
        if Self::is_address(addr) {
            Ok(addr)
        } else {
            Err(Error::InvalidAddress { addr })
        }
    }

    pub fn to_pc_aligned(&self, addr: u16) -> Result<u16> {
        self.to_valid_address(addr).and_then(|pc| {
            if pc % 2 == 0 {
                Ok(pc)
            } else {
                Err(Error::NotAligned { pc })
            }
        })
    }

    pub fn read_byte(&self, addr: u16) -> Result<u8> {
        if Self::is_address(addr) {
            Ok(self.mem[addr as usize])
        } else {
            Err(Error::InvalidAddress { addr })
        }
    }

    pub fn write_byte(&mut self, addr: u16, val: u8) -> Result {
        if Self::is_address(addr) && addr >= 0x200 {
            Ok(self.mem[addr as usize] = val)
        } else if addr < 0x200 {
            Err(Error::NotWritable { addr })
        } else {
            Err(Error::InvalidAddress { addr })
        }
    }

    pub fn read_bytes(&self, addr: u16, len: u8) -> Result<&[u8]> {
        let offset = addr as usize;
        let end = offset + len as usize;

        if Self::is_address(addr) && end <= 0x1000 && len > 0 {
            Ok(&self.mem[offset..end])
        } else {
            Err(Error::InvalidSlice { addr, len })
        }
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
    fn sprite_placement() {
        let ram = Ram::new();
        assert_eq!(&ram.mem[0x1B0..0x1B5], [0xF0, 0x90, 0x90, 0x90, 0xF0]);
        assert_eq!(&ram.mem[0x1FB..0x200], [0xF0, 0x80, 0xF0, 0x80, 0x80]);
    }
}
