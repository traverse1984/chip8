use super::macros::chip8_instruction_set;
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

chip8_instruction_set! {
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

impl Opcode {
    /// Attempt to decode an instruction from bytecode.
    pub fn decode(inst: u16) -> Option<Self> {
        use Opcode::*;

        match (inst & 0xF000) >> 12 {
            0x0 if inst & 0xFFF == 0x0E0 => Some(Cls),
            0x0 if inst & 0xFFF == 0x0EE => Some(Ret),
            0x1 => Some(Jp),
            0x2 => Some(Call),
            0x3 => Some(Se),
            0x4 => Some(Sne),
            0x5 if inst & 0xF == 0x0 => Some(Sev),
            0x6 => Some(Ld),
            0x7 => Some(Add),
            0x8 => match inst & 0xF {
                0x0 => Some(Ldv),
                0x1 => Some(Or),
                0x2 => Some(And),
                0x3 => Some(Xor),
                0x4 => Some(Addv),
                0x5 => Some(Sub),
                0x6 => Some(Shr),
                0x7 => Some(Subn),
                0xE => Some(Shl),
                _ => None,
            },
            0x9 if inst & 0xF == 0x0 => Some(Snev),
            0xA => Some(Ldi),
            0xB => Some(Jp0),
            0xC => Some(Rnd),
            0xD => Some(Drw),
            0xE if inst & 0xFF == 0x9E => Some(Skp),
            0xE if inst & 0xFF == 0xA1 => Some(Sknp),
            0xF => match inst & 0xFF {
                0x07 => Some(Lddtv),
                0x0A => Some(Ldkey),
                0x15 => Some(Lddt),
                0x18 => Some(Ldst),
                0x1E => Some(Addi),
                0x29 => Some(Sprite),
                0x33 => Some(Bcd),
                0x55 => Some(Sviv),
                0x65 => Some(Ldiv),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.opcode.name(), self.operands)
    }
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
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
