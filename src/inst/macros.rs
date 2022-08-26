macro_rules! chip8_op {
    (
        #[$doc: meta]
        $name: ident ( $( $($arg: ident $type: ty),+ )? ) -> $code: literal
    ) => {
        #[$doc]
        #[inline]
        pub fn $name( $( $($arg: $type),+ )? ) -> u16 {
            $code $(| $( $crate::inst::bytecode::encode::$arg($arg) )|+ )?
        }
    };
}

macro_rules! chip8_instruction_set {
    (
        $(
            #[$doc: meta]
            $code: literal $name: ident = $op: ident
            $( [ $($arg: ident $type: ty),+ ] )?;
        )+
    ) => {
        pub mod ops {
            $(
                $crate::inst::macros::chip8_op! {
                    #[$doc]
                    $op ( $( $($arg $type),+ )? ) -> $code
                }
            )+
        }

        /// An opcode
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[non_exhaustive]
        #[repr(u16)]
        pub enum Opcode {
            $( #[$doc] $name = $code, )+
        }

        impl Opcode {
            pub fn name(&self) -> &'static str {
                match self {
                    $( Self::$name => stringify!($op), )+
                }
            }
        }

        // An instruction
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct Instruction {
            opcode: Opcode,
            operands: $crate::inst::Operands,
        }

        impl Instruction {
            pub fn decode(inst: u16) -> Option<Self> {
                use Opcode::*;

                Some(match Opcode::decode(inst)? {
                    $(
                        opcode @ $name => Instruction {
                            opcode,
                            operands: $crate::inst::macros::chip8_operands!(
                                $( $($arg)+ )?; inst
                            ),
                        },
                    )+
                })
            }

            pub fn name(&self) -> &'static str {
                self.opcode.name()
            }

            pub fn operands(&self) -> & $crate::inst::Operands {
                &self.operands
            }

            pub fn opcode(&self) -> &Opcode {
                &self.opcode
            }

            pub fn code(&self) -> u16 {
                self.opcode as u16
            }
        }

        #[cfg(test)]
        #[allow(overflowing_literals)]
        mod ops_tests {
            use $crate::inst::bytecode::{encode, decode};
            use super::{ops, Instruction, Opcode};

            macro_rules! test_arg {
                ($argn: ident) => {
                    decode::$argn(encode::$argn(0x0ABC))
                };
            }

            $(
                #[test]
                fn $op() {
                    $( $( let $arg = test_arg!($arg); )+ )?
                    let inst = ops::$op( $( $($arg),+ )? );
                    let decoded = Instruction::decode(inst).unwrap();
                    let operands = $crate::inst::macros::chip8_operands!(
                        $( $($arg)+ )?; inst
                    );

                    assert_eq!(decoded.code(), $code);
                    assert_eq!(decoded.opcode(), &Opcode::$name);
                    assert_eq!(decoded.name(), stringify!($op));
                    assert_eq!(decoded.operands(), &operands);
                }
            )+
        }
    };
}

macro_rules! chip8_operands {
    ($($arg: ident)+ = $variant: ident $inst: expr) => {
        $crate::inst::Operands::$variant(
            $( $crate::inst::bytecode::decode::$arg($inst) ),+
        )
    };

    (; $inst: expr) => {
        $crate::inst::Operands::Exact
    };

    (addr; $inst: expr) => {
        $crate::inst::macros::chip8_operands!(addr = Addr $inst)
    };

    (vx; $inst: expr) => {
        $crate::inst::macros::chip8_operands!(vx = Vx $inst)
    };

    (vx vy; $inst: expr) => {
        $crate::inst::macros::chip8_operands!(vx vy = VxVy $inst)
    };

    (vx byte; $inst: expr) => {
        $crate::inst::macros::chip8_operands!(vx byte = VxByte $inst)
    };

    (vx vy nibble; $inst: expr) => {
        $crate::inst::macros::chip8_operands!(vx vy nibble = VxVyNibble $inst)
    };
 }

pub(super) use chip8_instruction_set;
pub(super) use chip8_op;
pub(super) use chip8_operands;
