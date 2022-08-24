use core::fmt;

#[macro_export]
macro_rules! chip8_asm {
    ( $( $fn: ident $($arg: expr),*; )+ ) => {
        [ $( $crate::chip8_inst!( $fn $($arg),* ) ),+ ]
    };
}

#[macro_export]
macro_rules! chip8_inst {
    ($fn: ident $($arg: expr),*) => {
        $crate::inst::ops::$fn( $($arg),* )
    };
}

macro_rules! instruction_set {
    (
        $(
            #[$doc: meta]
            $code: literal $name: ident = $op: ident
            $( [ $($arg: ident $type: ty),+ ] )?;
        )+
    ) => {
        pub mod ops {
            $(
                op! {
                    #[$doc]
                    $op ( $( $($arg $type),+ )? ) -> $code
                }
            )+
        }

        /// An instruction.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[non_exhaustive]
        pub enum Instruction {
            $(
                #[$doc]
                $name $( ( $($type),+ ) )?,
            )+
        }

        impl Instruction {
            /// Generate a description of the instruction.
            pub fn describe(&self) -> Description {
                match self {
                    $(
                        &Self::$name $( ( $($arg),+ ) )? => (
                            Description {
                                name: stringify!($op),
                                code: $code,
                                operands: operands!( $( $($arg)+ => $($arg),+ )? )
                            }
                        ),
                    )+
                }
            }

            /// Encode this instruction to it's bytecode representation.
            pub fn encode(&self) -> u16 {
                match self {
                    $(
                        &Self::$name $( ( $($arg),+ ) )? => {
                            $crate::inst::ops::$op( $( $($arg),+ )? )
                        }
                    )+
                }
            }
        }

        #[cfg(test)]
        #[allow(overflowing_literals)]
        mod ops_tests {
            use $crate::inst::bytecode::{encode, decode};
            use super::{ops, Instruction, Operands, Description};

            $(
                #[test]
                fn $op() {
                    $( $( let $arg = decode::$arg(encode::$arg(0x0ABC)); )+ )?
                    let inst = ops::$op( $( $($arg),+ )? );
                    let decoded = Instruction::decode(inst).unwrap();

                    assert_eq!(decoded.encode(), inst);
                    assert_eq!(
                        decoded.describe(),
                        Description {
                            name: stringify!($op),
                            code: $code,
                            operands: operands!( $( $($arg)+ => $($arg),+ )? ),
                        }
                    );
                }
            )+
        }
    };
}

macro_rules! op {
    (
        #[$doc: meta]
        $name: ident ( $( $($arg: ident $type: ty),+ )? ) -> $code: literal
    ) => {
        #[$doc]
        #[inline]
        pub fn $name( $( $($arg: $type),+ )? ) -> u16 {
            $code $( $( | $crate::inst::bytecode::encode::$arg($arg) )+ )?
        }
    };
}

macro_rules! operands {
    () => {
        Operands::Exact
    };

    (addr => $addr: expr) => {
        Operands::Addr($addr)
    };

    (vx => $vx: expr) => {
        Operands::Vx($vx)
    };

    (vx vy => $vx: expr, $vy: expr) => {
        Operands::VxVy($vx, $vy)
    };

    (vx byte => $vx: expr, $byte: expr) => {
        Operands::VxByte($vx, $byte)
    };

    (vx vy nibble => $vx: expr, $vy: expr, $nibble: expr) => {
        Operands::VxVyNibble($vx, $vy, $nibble)
    };
}

instruction_set! {
    /// Clear the display.
    0x00E0 Cls = cls;
    /// Return from a subroutine.
    0x00EE Ret = ret;
    /// Jump to location `addr`.
    0x1000 Jp = jp [addr u16];
    /// Call subroutine at `addr`.
    0x2000 Call = call [addr u16];
    /// Skip next instruction if `vx` == `byte`.
    0x3000 Se = se [vx u8, byte u8];
    /// Skip next instruction if `vx` != `byte`.
    0x4000 Sne = sne [vx u8, byte u8];
    /// Skip next instruction if `vx` == `vy`.
    0x5000 Sev = sev [vx u8, vy u8];
    /// Set `vx` = `byte`.
    0x6000 Ld = ld [vx u8, byte u8];
    /// Set `vx` = `vx` + `byte`.
    0x7000 Add = add [vx u8, byte u8];
    /// Set `vx` = `vy`.
    0x8000 Ldv = ldv [vx u8, vy u8];
    /// Set `vx` = `vx` OR `vy`.
    0x8001 Or = or [vx u8, vy u8];
    /// Set `vx` = `vx` AND `vy`.
    0x8002 And = and [vx u8, vy u8];
    /// Set `vx` = `vx` XOR `vy`.
    0x8003 Xor = xor [vx u8, vy u8];
    /// Set `vx` = `vx` + `vy`, set `vf` = carry.
    0x8004 Addv = addv [vx u8, vy u8];
    /// Set `vx` = `vx` - `vy`, set `vf` = NOT borrow.
    0x8005 Sub = sub [vx u8, vy u8];
    /// Set `vx` = `vx` SHR 1.
    0x8006 Shr = shr [vx u8];
    /// Set `vx` = `vy` - `vx`. Set `vf` = NOT borrow.
    0x8007 Subn = subn [vx u8, vy u8];
    /// Set `vx` = `vx` SHL 1.
    0x800E Shl = shl [vx u8];
    /// Skip next instruction if `vx` != `vy`.
    0x9000 Snev = snev [vx u8, vy u8];
    /// Set **I** = `addr`.
    0xA000 Ldi = ldi [addr u16];
    /// Jump to location `addr` + `v0`.
    0xB000 Jp0 = jp0 [addr u16];
    /// Set `vx` = random byte AND `byte`
    0xC000 Rnd = rnd [vx u8, byte u8];
    /// Display n-byte sprite at (`vx`, `vy`) starting at memory location **I**. Set `vf` = collision.
    0xD000 Drw = drw [vx u8, vy u8, nibble u8];
    /// Skip next instruction if key with the value of `vx` is pressed.
    0xE09E Skp = skp [vx u8];
    /// Skip next instruction if key with the value of `vx` is not pressed.
    0xE0A1 Sknp = sknp [vx u8];
    /// Set `vx` = delay timer value.
    0xF007 Lddtv = lddtv [vx u8];
    /// Wait for a key press, store the value of the key in `vx`.
    0xF00A Ldkey = ldkey [vx u8];
    /// Set delay timer = `vx`.
    0xF015 Lddt = lddt [vx u8];
    /// Set sound timer = `vx`.
    0xF018 Ldst = ldst [vx u8];
    /// Set **I** = **I** + `vx`.
    0xF01E Addi = addi [vx u8];
    /// Set **I** = location of sprite for digit `vx`.
    0xF029 Sprite = sprite [vx u8];
    /// Store BCD representation of `vx` in memory locations **I**, **I**+1, and **I**+2.
    0xF033 Bcd = bcd [vx u8];
    /// Store registers `v0` through `vx` in memory starting at location **I**.
    0xF055 Sviv = sviv [vx u8];
    /// Read registers `v0` through `vx` from memory starting at location **I**.
    0xF065 Ldiv = ldiv [vx u8];
}

impl Instruction {
    /// Attempt to decode an instruction from bytecode.
    pub fn decode(inst: u16) -> Option<Self> {
        use super::bytecode::decode;
        use Instruction::*;

        macro_rules! de {
            ($name: ident $( $($arg: ident),+ )? ) => {
                Some($name $( ( $(decode::$arg(inst) ),+ ) )? )
            };
        }

        match (inst & 0xF000) >> 12 {
            0x0 if inst & 0xFFF == 0x0E0 => de!(Cls),
            0x0 if inst & 0xFFF == 0x0EE => de!(Ret),
            0x1 => de!(Jp addr),
            0x2 => de!(Call addr),
            0x3 => de!(Se vx, byte),
            0x4 => de!(Sne vx, byte),
            0x5 if inst & 0xF == 0x0 => de!(Sev vx, vy),
            0x6 => de!(Ld vx, byte),
            0x7 => de!(Add vx, byte),
            0x8 => match inst & 0xF {
                0x0 => de!(Ldv vx, vy),
                0x1 => de!(Or vx, vy),
                0x2 => de!(And vx, vy),
                0x3 => de!(Xor vx, vy),
                0x4 => de!(Addv vx, vy),
                0x5 => de!(Sub vx, vy),
                0x6 => de!(Shr vx),
                0x7 => de!(Subn vx, vy),
                0xE => de!(Shl vx),
                _ => None,
            },
            0x9 if inst & 0xF == 0x0 => de!(Snev vx, vy),
            0xA => de!(Ldi addr),
            0xB => de!(Jp0 addr),
            0xC => de!(Rnd vx, byte),
            0xD => de!(Drw vx, vy, nibble),
            0xE if inst & 0xFF == 0x9E => de!(Skp vx),
            0xE if inst & 0xFF == 0xA1 => de!(Sknp vx),
            0xF => match inst & 0xFF {
                0x07 => de!(Lddtv vx),
                0x0A => de!(Ldkey vx),
                0x15 => de!(Lddt vx),
                0x18 => de!(Ldst vx),
                0x1E => de!(Addi vx),
                0x29 => de!(Sprite vx),
                0x33 => de!(Bcd vx),
                0x55 => de!(Sviv vx),
                0x65 => de!(Ldiv vx),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Description of an instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Description {
    pub name: &'static str,
    pub code: u16,
    pub operands: Operands,
}

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

impl fmt::Display for Description {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.operands)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.describe().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction;

    #[test]
    fn invalid_instruction() {
        assert_eq!(Instruction::decode(0x0123), None);
    }

    #[test]
    fn assembler() {
        let prog = chip8_asm! {
            cls;
            jp 0x123;
            drw 1, 2, 3;
            ret;
        };

        assert_eq!(prog, [0x00E0, 0x1123, 0xD123, 0x00EE]);
    }
}
