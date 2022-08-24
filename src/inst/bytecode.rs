pub mod encode {
    pub fn addr(addr: u16) -> u16 {
        addr & 0x0FFF
    }

    #[inline]
    pub fn byte(byte: u8) -> u16 {
        byte as u16
    }
    #[inline]
    pub fn vx(vx: u8) -> u16 {
        ((vx & 0xF) as u16) << 8
    }

    #[inline]
    pub fn vy(vy: u8) -> u16 {
        ((vy & 0xF) as u16) << 4
    }

    #[inline]
    pub fn nibble(nibble: u8) -> u16 {
        (nibble & 0xF) as u16
    }
}

pub mod decode {
    #[inline]
    pub fn addr(inst: u16) -> u16 {
        inst & 0x0FFF
    }

    #[inline]
    pub fn byte(inst: u16) -> u8 {
        inst as u8
    }

    #[inline]
    pub fn vx(inst: u16) -> u8 {
        ((inst & 0x0F00) >> 8) as u8
    }

    #[inline]
    pub fn vy(inst: u16) -> u8 {
        ((inst & 0x00F0) >> 4) as u8
    }

    #[inline]
    pub fn nibble(inst: u16) -> u8 {
        (inst & 0xF) as u8
    }
}
