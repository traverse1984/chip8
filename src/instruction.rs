#[macro_export]
macro_rules! chip8_asm {
    ( $( $fn: ident $($arg: expr),*; )+ ) => {
        [ $( $crate::instruction::$fn($($arg),*) ),+ ]
    };
}

macro_rules! instruction_set {
    (
        $(
            $doc: expr;
            $name: ident
            $($varname: ident),*
            ->
            $mask: literal;
        )+
    ) => {
        $(chip8_fn!($doc; $name $($varname),* -> $mask);)+
    };
}

macro_rules! chip8_fn {
    ($doc: expr; $name: ident -> $mask: literal) => {
        #[doc = $doc]
        pub fn $name() -> u16 {
            $mask
        }
    };

    ($doc: expr; $name: ident addr -> $mask: literal) => {
        #[doc = $doc]
        pub fn $name(addr: u16) -> u16 {
            $mask | addr
        }
    };

    ($doc: expr; $name: ident vx -> $mask: literal) => {
        #[doc = $doc]
        pub fn $name(vx: u8) -> u16 {
            $mask | shift_vx(vx)
        }
    };

    ($doc: expr; $name: ident vx, vy -> $mask: literal) => {
        #[doc = $doc]
        pub fn $name(vx: u8, vy: u8) -> u16 {
            $mask | shift_vx(vx) | shift_vy(vy)
        }
    };

    ($doc: expr; $name: ident vx, byte -> $mask: literal) => {
        #[doc = $doc]
        pub fn $name(vx: u8, byte: u8) -> u16 {
            $mask | shift_vx(vx) | byte as u16
        }
    };

    ($doc: expr; $name: ident vx, vy, nibble -> $mask: literal) => {
        #[doc = $doc]
        pub fn $name(vx: u8, vy: u8, nibble: u8) -> u16 {
            $mask | shift_vx(vx) | shift_vy(vy) | ((nibble & 0x0F) as u16)
        }
    };
}

#[inline]
fn shift_vx(vx: u8) -> u16 {
    ((vx & 0x0F) as u16) << 8
}

#[inline]
fn shift_vy(vy: u8) -> u16 {
    (vy << 4) as u16
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
        shr vx -> 0x8006;
    "Set `vx` = `vy` - `vx`. Set `vf` = NOT borrow.";
        subn vx, vy -> 0x8007;
    "Set `vx` = `vx` SHL 1.";
        shl vx -> 0x800E;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instructions() {
        assert_eq!(cls(), 0x00E0);
        assert_eq!(ret(), 0x00EE);
        assert_eq!(jp(0x123), 0x1123);
        assert_eq!(call(0x123), 0x2123);
        assert_eq!(se(1, 0x23), 0x3123);
        assert_eq!(sne(1, 0x23), 0x4123);
        assert_eq!(sev(1, 2), 0x5120);
        assert_eq!(ld(1, 0x23), 0x6123);
        assert_eq!(add(1, 0x23), 0x7123);
        assert_eq!(ldv(1, 2), 0x8120);
        assert_eq!(or(1, 2), 0x8121);
        assert_eq!(and(1, 2), 0x8122);
        assert_eq!(xor(1, 2), 0x8123);
        assert_eq!(addv(1, 2), 0x8124);
        assert_eq!(sub(1, 2), 0x8125);
        assert_eq!(shr(1), 0x8106);
        assert_eq!(subn(1, 2), 0x8127);
        assert_eq!(shl(1), 0x810E);
        assert_eq!(snev(1, 2), 0x9120);
        assert_eq!(ldi(0x123), 0xA123);
        assert_eq!(jp0(0x123), 0xB123);
        assert_eq!(rnd(1, 0x23), 0xC123);
        assert_eq!(drw(1, 2, 3), 0xD123);
        assert_eq!(skp(1), 0xE19E);
        assert_eq!(sknp(1), 0xE1A1);
        assert_eq!(lddtv(1), 0xF107);
        assert_eq!(ldkey(1), 0xF10A);
        assert_eq!(lddt(1), 0xF115);
        assert_eq!(ldst(1), 0xF118);
        assert_eq!(addi(1), 0xF11E);
        assert_eq!(sprite(1), 0xF129);
        assert_eq!(bcd(1), 0xF133);
        assert_eq!(sviv(1), 0xF155);
        assert_eq!(ldiv(1), 0xF165);
    }

    #[test]
    fn program() {
        let prog = chip8_asm! {
            cls;
            jp 0x123;
            drw 1, 2, 3;
            ret;
        };

        assert_eq!(prog, [0x00E0, 0x1123, 0xD123, 0x00EE]);
    }
}
