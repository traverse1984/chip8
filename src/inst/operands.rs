use core::fmt;

/// Operands for an instruction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Operands {
    /// An instruction which accepts no operands.
    Exact,
    /// An instruction which expects address `addr`.
    Addr(u16),
    /// An instruction which expects register `vx`.
    Vx(u8),
    /// An instruction which expects registers `vx` and `vy`.
    VxVy(u8, u8),
    /// An instruction which expects register `vx` and a `byte` of data.
    VxByte(u8, u8),
    /// An instruction which expects registers `vx`, `vy` and a `nibble` of
    /// data. In the chip8 instruction set, only `drw` has this signature.
    VxVyNibble(u8, u8, u8),
}

impl Operands {
    pub fn encode(&self) -> u16 {
        use Operands::*;

        macro_rules! enc {
            ($($arg: ident),+) => {
                $( $crate::inst::bytecode::encode::$arg($arg) )|+
            };
        }

        match self {
            &Exact => 0x0000,
            &Addr(addr) => enc!(addr),
            &Vx(vx) => enc!(vx),
            &VxVy(vx, vy) => enc!(vx, vy),
            &VxByte(vx, byte) => enc!(vx, byte),
            &VxVyNibble(vx, vy, nibble) => enc!(vx, vy, nibble),
        }
    }
}

impl fmt::Display for Operands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Operands::*;
        match self {
            Exact => Ok(()),
            Addr(addr) => write!(f, "{:03X}", addr),
            Vx(vx) => write!(f, "{:X}", vx),
            VxVy(vx, vy) => write!(f, "{:X}, {:X}", vx, vy),
            VxByte(vx, byte) => write!(f, "{:X}, {:02X}", vx, byte),
            VxVyNibble(vx, vy, nibble) => write!(f, "{:X}, {:X}, {:X}", vx, vy, nibble),
        }
    }
}
