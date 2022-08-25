macro_rules! bc {
    ( $mod: ident $($arg: ident $val: expr),* ) => {
        ( $( $crate::inst::bytecode::$mod::$arg($val) ),* )
    };

    ( with $val: expr; $mod: ident $($arg: ident),+ ) => {
        $crate::inst::bytecode::bc!( $mod $($arg $val),+ )
    };

    ( opcode $op: expr $(; $( $($arg: ident $val: expr),+ )? )? ) => {
        ($op as u16) $( $(|
            $( $crate::inst::bytecode::encode::$arg($val) )|+
        )? )?
    };
}

pub(crate) use bc;

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
    use super::bc;

    #[test]
    fn encode() {
        assert_eq!(bc!(encode addr 0xABCD), 0x0BCD);
        assert_eq!(bc!(encode vx 0xA, vy 0xB), (0x0A00, 0x00B0));
        assert_eq!(bc!(with 0xAB; encode byte, nibble), (0x00AB, 0x000B));
    }

    #[test]
    fn decode() {
        assert_eq!(bc!(decode addr 0xABCD), 0xBCD);
        assert_eq!(bc!(decode vx 0xABCD, vy 0xABCD), (0xB, 0xC));
        assert_eq!(bc!(with 0xABCD; decode byte, nibble), (0xCD, 0xD));
    }

    #[test]
    fn opcode() {
        let op = 0xA000;
        let (x, y, nib) = (0xB, 0xC, 0xD);
        let inst = bc!(opcode op; vx x, vy y, nibble nib);

        assert_eq!(inst, 0xABCD);
    }
}
