pub mod encode {
    #[inline]
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

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn encode() {
        assert_eq!(encode::addr(0xABCD), 0x0BCD);
        assert_eq!(encode::vx(0xAB), 0x0B00);
        assert_eq!(encode::vy(0xAB), 0x00B0);
        assert_eq!(encode::byte(0xAB), 0x00AB);
        assert_eq!(encode::nibble(0xAB), 0x000B);
    }

    #[test]
    fn decode() {
        assert_eq!(decode::addr(0xABCD), 0x0BCD);
        assert_eq!(decode::vx(0xABCD), 0xB);
        assert_eq!(decode::vy(0xABCD), 0xC);
        assert_eq!(decode::byte(0xABCD), 0xCD);
        assert_eq!(decode::nibble(0xABCD), 0xD);
    }
}
