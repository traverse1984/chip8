#[macro_export]
macro_rules! chip8_asm {

    ( @output $( $fn: ident $($reg: expr),* ; )+ ) => {
        [ $( chip8_inst!( $fn $($reg),* ) ),+ ]
    };

    ( @mut $($tail: tt)* ) => {
        &mut chip8_asm!( @output $($tail)* )[..]
    };

    ( output; $($tail: tt)* ) => {
        chip8_asm!( @output $($tail)* )
    };

    ( $($tail: tt)* ) => {
        chip8_asm!( @mut $($tail)* )
    };
}

macro_rules! instruction_set {
    (
        ($d: tt);
        $(
            $mask: literal
            $name: ident
            $($varname: ident),*
            ;
        )+
    ) => {
        #[macro_export]
        macro_rules! chip8_inst {
            $(
                ( $name $($d $varname: expr),* ) => {
                    $crate::__chip8_inst!($mask $($varname $d $varname),*)
                };
            )+
        }
    };
}

instruction_set! {
    ($);
    0x00E0 cls;
    0x00EE ret;
    0x1000 jp nnn;
    0x2000 call nnn;
    0x3000 se vx, kk;
    0x4000 sne vx, kk;
    0x5000 sexy vx, vy;
    0x6000 ld vx, kk;
    0x7000 addkk vx, kk;
    0x8000 ldxy vx, vy;
    0x8001 or vx, vy;
    0x8002 and vx, vy;
    0x8003 xor vx, vy;
    0x8004 add vx, vy;
    0x8005 sub vx, vy;
    0x8006 shr vx;
    0x8007 subn vx, vy;
    0x800E shl vx;
    0x9000 snexy vx, vy;
    0xA000 ldi nnn;
    0xB000 jp0 nnn;
    0xC000 rnd vx, kk;
    0xD000 drw vx, vy, n;
    0xE09E skp vx;
    0xE0A1 sknp vx;
    0xF007 ld_xdt vx;
    0xF00A ld_xkey vx;
    0xF015 ld_dtx vx;
    0xF018 ld_stx vx;
    0xF01E addix vx;
    0xF029 sprite vx;
    0xF033 bcd vx;
    0xF055 save vx;
    0xF065 load vx;
}

#[macro_export]
macro_rules! __chip8_inst {
    (
        $mask: literal
        $( $($reg: ident $val: expr),+ )?
    ) => {
        ($mask as u16) $(| $($crate::__chip8_inst!($reg $val))|* )?
    };

    ( vx $vx: expr ) => {
        (($vx as u16) << 8)
    };

    ( vy $vy: expr ) => {
        (($vy as u16) << 4)
    };

    ( nnn $nnn: expr ) => {
        ($nnn as u16)
    };

    ( kk $kk: expr ) => {
        ($kk as u16)
    };

    ( n $n: expr ) => {
        ($n as u16)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn registers() {
        assert_eq!(__chip8_inst!(0x1234), 0x1234);
        assert_eq!(__chip8_inst!(0x1000 vx 2, vy 3, n 4), 0x1234);
        assert_eq!(__chip8_inst!(0x1000 nnn 0x0234), 0x1234);
        assert_eq!(__chip8_inst!(0x1200 kk 0x34), 0x1234);
    }

    #[test]
    fn instructions() {
        assert_eq!(chip8_inst!(cls), 0x00E0);
        assert_eq!(chip8_inst!(ret), 0x00EE);
        assert_eq!(chip8_inst!(jp 0x123), 0x1123);
        assert_eq!(chip8_inst!(call 0x123), 0x2123);
        assert_eq!(chip8_inst!(se 1, 0x23), 0x3123);
        assert_eq!(chip8_inst!(sne 1, 0x23), 0x4123);
        assert_eq!(chip8_inst!(sexy 1, 2), 0x5120);
        assert_eq!(chip8_inst!(ld 1, 0x23), 0x6123);
        assert_eq!(chip8_inst!(addkk 1, 0x23), 0x7123);
        assert_eq!(chip8_inst!(ldxy 1, 2), 0x8120);
        assert_eq!(chip8_inst!(or 1, 2), 0x8121);
        assert_eq!(chip8_inst!(and 1, 2), 0x8122);
        assert_eq!(chip8_inst!(xor 1, 2), 0x8123);
        assert_eq!(chip8_inst!(add 1, 2), 0x8124);
        assert_eq!(chip8_inst!(sub 1, 2), 0x8125);
        assert_eq!(chip8_inst!(shr 1), 0x8106);
        assert_eq!(chip8_inst!(subn 1, 2), 0x8127);
        assert_eq!(chip8_inst!(shl 1), 0x810E);
        assert_eq!(chip8_inst!(snexy 1, 2), 0x9120);
        assert_eq!(chip8_inst!(ldi 0x123), 0xA123);
        assert_eq!(chip8_inst!(jp0 0x123), 0xB123);
        assert_eq!(chip8_inst!(rnd 1, 0x23), 0xC123);
        assert_eq!(chip8_inst!(drw 1, 2, 3), 0xD123);
        assert_eq!(chip8_inst!(skp 1), 0xE19E);
        assert_eq!(chip8_inst!(sknp 1), 0xE1A1);
        assert_eq!(chip8_inst!(ld_xdt 1), 0xF107);
        assert_eq!(chip8_inst!(ld_xkey 1), 0xF10A);
        assert_eq!(chip8_inst!(ld_dtx 1), 0xF115);
        assert_eq!(chip8_inst!(ld_stx 1), 0xF118);
        assert_eq!(chip8_inst!(addix 1), 0xF11E);
        assert_eq!(chip8_inst!(sprite 1), 0xF129);
        assert_eq!(chip8_inst!(bcd 1), 0xF133);
        assert_eq!(chip8_inst!(save 1), 0xF155);
        assert_eq!(chip8_inst!(load 1), 0xF165);
    }

    #[test]
    fn program() {
        let prog = chip8_asm! {
            cls;
            jp 0x123;
            call 0x234;
            ret;
        };

        assert_eq!(prog, [0x00E0, 0x1123, 0x2234, 0x00EE]);
    }
}
