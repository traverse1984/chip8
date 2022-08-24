use super::bytecode::decode;

#[macro_export]
macro_rules! chip8_asm {
    ( $( $fn: ident $($arg: expr),*; )+ ) => {
        [ $( $crate::inst::ops::$fn($($arg),*) ),+ ]
    };
}

macro_rules! instruction_set {
    (
        $(
            $doc: expr;
            $name: ident $($varname: ident),*
                -> $mask: literal;
        )+
    ) => {
        pub mod ops {
            $(instruction!($doc; $name $($varname),* -> $mask);)+
        }

        impl Instruction {
            pub fn decode(inst: u16) -> Option<Self> {
                let instructions = [
                    $(DecodeInstruction {
                        name: stringify!($name),
                        mask: $mask,
                        operands: instruction_operands!($($varname),*),
                    }),+
                ];

                for DecodeInstruction { name, operands, mask } in instructions {
                    if operands.unmask(inst) == mask {
                        return Some(Instruction {
                            name,
                            operands: operands.decode_from(inst),
                            decoded_from: inst,
                        });
                    }
                }

                None
            }
        }
    };
}

macro_rules! instruction {
    (
        $doc: expr;
        $name: ident ( $($arg: ident : $type: ty),* )
            ->  $body: expr
    ) => {
        #[inline]
        #[doc = $doc]
        pub fn $name( $($arg: $type),* ) -> u16 {
            use $crate::inst::bytecode::encode as enc;
            $body
        }
    };

    ($doc: expr; $name: ident -> $mask: literal) => {
        instruction! { $doc; $name () -> $mask }
    };

    ($doc: expr; $name: ident addr -> $mask: literal) => {
        instruction! {
            $doc; $name (addr: u16)
                -> $mask | enc::addr(addr)
        }
    };

    ($doc: expr; $name: ident vx $(, any)? -> $mask: literal) => {
        instruction! {
            $doc; $name (vx: u8)
                -> $mask | enc::vx(vx)
        }
    };

    ($doc: expr; $name: ident vx, vy -> $mask: literal) => {
        instruction! {
            $doc; $name (vx: u8, vy: u8)
                -> $mask | enc::vx(vx) | enc::vy(vy)
        }
    };

    ($doc: expr; $name: ident vx, byte -> $mask: literal) => {
        instruction! {
            $doc; $name (vx: u8, byte: u8)
                -> $mask | enc::vx(vx) | enc::byte(byte)
        }
    };

    ($doc: expr; $name: ident vx, vy, nibble -> $mask: literal) => {
        instruction! {
            $doc; $name (vx: u8, vy: u8, nibble: u8)
                -> $mask | enc::vx(vx) | enc::vy(vy) | enc::nibble(nibble)
        }
    };
}

macro_rules! instruction_operands {
    () => {
        DecodeOperands::Exact
    };

    (addr) => {
        DecodeOperands::Addr
    };

    (vx) => {
        DecodeOperands::Vx
    };

    (vx, any) => {
        DecodeOperands::VxAny
    };

    (vx, vy) => {
        DecodeOperands::VxVy
    };

    (vx, byte) => {
        DecodeOperands::VxByte
    };

    (vx, vy, nibble) => {
        DecodeOperands::VxVyNibble
    };
}

instruction_set! {
    "Clear the display.";
        cls -> 0x00E0;
    "Return from a subroutine.";
        ret -> 0x00EE;
    "Jump to location `addr`.";
        jp addr -> 0x1000;
    "Call subroutine at `addr`.";
        call addr -> 0x2000;
    "Skip next instruction if `vx` == `byte`.";
        se vx, byte -> 0x3000;
    "Skip next instruction if `vx` != `byte`.";
        sne vx, byte -> 0x4000;
    "Skip next instruction if `vx` == `vy`.";
        sev vx, vy -> 0x5000;
    "Set `vx` = `byte`.";
        ld vx, byte -> 0x6000;
    "Set `vx` = `vx` + `byte`.";
        add vx, byte -> 0x7000;
    "Set `vx` = `vy`.";
        ldv vx, vy -> 0x8000;
    "Set `vx` = `vx` OR `vy`.";
        or vx, vy -> 0x8001;
    "Set `vx` = `vx` AND `vy`.";
        and vx, vy -> 0x8002;
    "Set `vx` = `vx` XOR `vy`.";
        xor vx, vy -> 0x8003;
    "Set `vx` = `vx` + `vy`, set `vf` = carry.";
        addv vx, vy -> 0x8004;
    "Set `vx` = `vx` - `vy`, set `vf` = NOT borrow.";
        sub vx, vy -> 0x8005;
    "Set `vx` = `vx` SHR 1.";
        shr vx, any -> 0x8006;
    "Set `vx` = `vy` - `vx`. Set `vf` = NOT borrow.";
        subn vx, vy -> 0x8007;
    "Set `vx` = `vx` SHL 1.";
        shl vx, any -> 0x800E;
    "Skip next instruction if `vx` != `vy`.";
        snev vx, vy -> 0x9000;
    "Set **I** = `addr`.";
        ldi addr -> 0xA000;
    "Jump to location `addr` + `v0`.";
        jp0 addr -> 0xB000;
    "Set `vx` = random byte AND `byte`";
        rnd vx, byte -> 0xC000;
    "Display n-byte sprite at (`vx`, `vy`) starting at memory location **I**. Set `vf` = collision.";
        drw vx, vy, nibble -> 0xD000;
    "Skip next instruction if key with the value of `vx` is pressed.";
        skp vx -> 0xE09E;
    "Skip next instruction if key with the value of `vx` is not pressed.";
        sknp vx -> 0xE0A1;
    "Set `vx` = delay timer value.";
        lddtv vx -> 0xF007;
    "Wait for a key press, store the value of the key in `vx`.";
        ldkey vx -> 0xF00A;
    "Set delay timer = `vx`.";
        lddt vx -> 0xF015;
    "Set sound timer = `vx`.";
        ldst vx -> 0xF018;
    "Set **I** = **I** + `vx`.";
        addi vx -> 0xF01E;
    "Set **I** = location of sprite for digit `vx`.";
        sprite vx -> 0xF029;
    "Store BCD representation of `vx` in memory locations **I**, **I**+1, and **I**+2.";
        bcd vx -> 0xF033;
    "Store registers `v0` through `vx` in memory starting at location **I**.";
        sviv vx -> 0xF055;
    "Read registers `v0` through `vx` from memory starting at location **I**.";
        ldiv vx -> 0xF065;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub name: &'static str,
    pub operands: Operands,
    decoded_from: u16,
}

impl Instruction {
    pub fn decoded_from(&self) -> u16 {
        self.decoded_from
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Operands {
    Exact,
    Addr(u16),
    Vx(u8),
    VxVy(u8, u8),
    VxByte(u8, u8),
    VxVyNibble(u8, u8, u8),
}

#[cfg(feature = "std")]
impl std::fmt::Display for Operands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

#[cfg(feature = "std")]
impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.operands)
    }
}

#[derive(Debug, Copy, Clone)]
struct DecodeInstruction {
    name: &'static str,
    mask: u16,
    operands: DecodeOperands,
}

#[derive(Debug, Copy, Clone)]
enum DecodeOperands {
    Exact,
    Addr,
    Vx,
    VxVy,
    VxAny,
    VxByte,
    VxVyNibble,
}

impl DecodeOperands {
    fn unmask(&self, inst: u16) -> u16 {
        use DecodeOperands::*;
        inst & match self {
            Exact => 0xFFFF,
            Addr | VxByte | VxVyNibble => 0xF000,
            VxVy | VxAny => 0xF00F,
            Vx => 0xF0FF,
        }
    }

    fn decode_from(&self, inst: u16) -> Operands {
        use decode::*;
        use DecodeOperands::*;
        use Operands as Op;

        match self {
            Exact => Op::Exact,
            Addr => Op::Addr(addr(inst)),
            Vx | VxAny => Op::Vx(vx(inst)),
            VxVy => Op::VxVy(vx(inst), vy(inst)),
            VxByte => Op::VxByte(vx(inst), byte(inst)),
            VxVyNibble => Op::VxVyNibble(vx(inst), vy(inst), nibble(inst)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ops, Instruction, Operands};

    macro_rules! test_instructions {
        (
            $(
                $name: ident ( $( $($arg: literal),+ )? )
                    -> $ret: literal -> $variant: ident;
            )+
        ) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!( ops::$name( $( $($arg),+ )? ), $ret );
                    assert_eq!(
                        Instruction::decode($ret).unwrap(),
                        Instruction {
                            name: stringify!($name),
                            operands: Operands::$variant $( ( $($arg),+ ) )?,
                            decoded_from: $ret,
                        }
                    );
                }
            )+
        };
    }

    test_instructions! {
        cls () -> 0x00E0 -> Exact;
        ret () -> 0x00EE -> Exact;
        jp (0x123) -> 0x1123 -> Addr;
        call (0x123) -> 0x2123 -> Addr;
        se (1, 0x23) -> 0x3123 -> VxByte;
        sne (1, 0x23) -> 0x4123 -> VxByte;
        sev (1, 2) -> 0x5120 -> VxVy;
        ld (1, 0x23) -> 0x6123 -> VxByte;
        add (1, 0x23) -> 0x7123 -> VxByte;
        ldv (1, 2) -> 0x8120 -> VxVy;
        or (1, 2) -> 0x8121 -> VxVy;
        and (1, 2) -> 0x8122 -> VxVy;
        xor (1, 2) -> 0x8123 -> VxVy;
        addv (1, 2) -> 0x8124 -> VxVy;
        sub (1, 2) -> 0x8125 -> VxVy;
        shr (1) -> 0x8106 -> Vx;
        subn (1, 2) -> 0x8127 -> VxVy;
        shl (1) -> 0x810E -> Vx;
        snev (1, 2) -> 0x9120 -> VxVy;
        ldi (0x123) -> 0xA123 -> Addr;
        jp0 (0x123) -> 0xB123 -> Addr;
        rnd (1, 0x23) -> 0xC123 -> VxByte;
        drw (1, 2, 3) -> 0xD123 -> VxVyNibble;
        skp (1) -> 0xE19E -> Vx;
        sknp (1) -> 0xE1A1 -> Vx;
        lddtv (1) -> 0xF107 -> Vx;
        ldkey (1) -> 0xF10A -> Vx;
        lddt (1) -> 0xF115 -> Vx;
        ldst (1) -> 0xF118 -> Vx;
        addi (1) -> 0xF11E -> Vx;
        sprite (1) -> 0xF129 -> Vx;
        bcd (1) -> 0xF133 -> Vx;
        sviv (1) -> 0xF155 -> Vx;
        ldiv (1) -> 0xF165 -> Vx;
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
