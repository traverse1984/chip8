extern crate std;
use super::chip8::{INST_STEP, REG_FLAG};
use super::HwChip8;
use crate::{
    hal::{
        mocks::{MockDraw, MockHardware},
        Hardware, HardwareExt,
    },
    mem::{self, Load},
};
use core::ops::{Deref, DerefMut};

use std::vec;

macro_rules! chip {
    ($hw: ident => { $($prepare: tt)+ }) => {
        HwChip8::new({
            let mut $hw = MockHardware::default();
            $($prepare)+
            $hw
        })
    };

    () => {
        HwChip8::new(MockHardware::default())
    };
}

// Create a Chip8 mock with some preset registers
macro_rules! chip_preset {
    ($($reg: literal = $val: literal),+) => {{
        let mut chip = chip!();
        $(chip.mem().reg.set($reg, $val).unwrap();)+
        chip
    }};
}

// Execute an instruction on the Chip8 instance
macro_rules! exec {
    ($chip: ident $($inst: tt)*) => {
        $chip.exec( $crate::chip8_inst!($($inst)*) ).unwrap()
    };
}

// Read or write a single register from the Chip8 instance
macro_rules! reg {
    ($chip: ident $reg: expr) => {
        $chip.mem().reg.get($reg).unwrap()
    };

    ($chip: ident $reg: literal => $val: expr) => {
        $chip.mem().reg.set($reg, $val).unwrap();
    };
}

// #[test]
// fn load() {
//     let mut chip = chip!();

//     chip.main(&[1u8, 2, 3, 4][..]).unwrap();
//     assert_eq!(chip.mem().ram.read_bytes(0x200, 4).unwrap(), &[1, 2, 3, 4]);

//     chip.sub(0x300, &[5, 6, 7, 8][..]).unwrap();
//     assert_eq!(chip.mem().ram.read_bytes(0x300, 4).unwrap(), &[5, 6, 7, 8]);
// }

// #[test]
// fn read_inst() {
//     let mut chip = chip!();

//     chip.main(&mut [0x11u16, 0x22u16, 0x33u16][..]).unwrap();
//     assert_eq!(chip.read_inst(0x200).unwrap(), 0x11);
//     assert_eq!(chip.read_inst(0x202).unwrap(), 0x22);
//     assert_eq!(chip.read_inst(0x204).unwrap(), 0x33);
//     assert_eq!(chip.read_inst(0x201).unwrap_err(), Error::NotAligned(0x201))
// }

#[test]
fn cls() {
    let mut chip = chip!();

    exec!(chip cls);
    let Hardware { screen, .. } = chip.hardware();
    assert_eq!(screen.draws, vec![MockDraw::Clear])
}

#[test]
fn ret() {
    let mut chip = chip!();
    chip.mem().stack.push(0x344).unwrap();

    exec!(chip ret);
    assert_eq!(chip.mem().stack.pop().unwrap_err(), mem::Error::StackEmpty);
    assert_eq!(chip.mem().pc, 0x346);
}

#[test]
fn jp() {
    let mut chip = chip!();

    exec!(chip jp 0x123);
    assert_eq!(chip.mem().pc, 0x0123);

    exec!(chip jp 0x456);
    assert_eq!(chip.mem().pc, 0x0456);
}

#[test]
fn call() {
    let mut chip = chip!();
    chip.mem().pc = 0x0123;

    exec!(chip call 0x456);
    assert_eq!(chip.mem().pc, 0x0456);
    assert_eq!(chip.mem().stack.pop().unwrap(), 0x0123);
}

#[test]
fn se() {
    let mut chip = chip_preset!(0 = 0x23);

    chip.mem().pc = 0;
    exec!(chip se 0, 0x23);
    assert_eq!(chip.mem().pc, 2 * INST_STEP);

    chip.mem().pc = 0;
    exec!(chip se 0, 0x24);
    assert_eq!(chip.mem().pc, INST_STEP);
}

#[test]
fn sne() {
    let mut chip = chip_preset!(0 = 0x23);

    chip.mem().pc = 0;
    exec!(chip sne 0, 0x23);
    assert_eq!(chip.mem().pc, INST_STEP);

    chip.mem().pc = 0;
    exec!(chip sne 0, 0x24);
    assert_eq!(chip.mem().pc, 2 * INST_STEP);
}

#[test]
fn sev() {
    let mut chip = chip_preset!(0 = 0x23, 1 = 0x23, 2 = 0x34);

    chip.mem().pc = 0;
    exec!(chip sev 0, 1);
    assert_eq!(chip.mem().pc, 2 * INST_STEP);

    chip.mem().pc = 0;
    exec!(chip sev 0, 2);
    assert_eq!(chip.mem().pc, INST_STEP);
}

#[test]
fn ld() {
    let mut chip = chip!();

    exec!(chip ld 0, 0x12);
    assert_eq!(reg!(chip 0), 0x12);

    exec!(chip ld 0xE, 0x34);
    assert_eq!(reg!(chip 0xE), 0x34);
}

#[test]
fn add() {
    let mut chip = chip_preset!(0 = 10, 1 = 254);

    exec!(chip add 0, 0x02);
    assert_eq!(reg!(chip 0), 12);

    exec!(chip add 1, 0x02);
    assert_eq!(reg!(chip 1), 0);
}

#[test]
fn ldv() {
    let mut chip = chip_preset!(1 = 123);

    exec!(chip ldv 0, 1);
    assert_eq!(reg!(chip 0), 123);
    assert_eq!(reg!(chip 1), 123);
}

#[test]
fn or() {
    let mut chip = chip_preset!(0 = 123, 1 = 45);

    exec!(chip or 0, 1);
    assert_eq!(reg!(chip 0), 123 | 45);
}

#[test]
fn and() {
    let mut chip = chip_preset!(0 = 123, 1 = 45);

    exec!(chip and 0, 1);
    assert_eq!(reg!(chip 0), 123 & 45);
}

#[test]
fn xor() {
    let mut chip = chip_preset!(0 = 123, 1 = 45);

    exec!(chip xor 0, 1);
    assert_eq!(reg!(chip 0), 123 ^ 45);
}

#[test]
fn addv() {
    let mut chip = chip_preset!(0 = 254, 1 = 2, 2 = 3);

    exec!(chip addv 0, 1);
    assert_eq!(reg!(chip 0), 0);
    assert_eq!(reg!(chip REG_FLAG), 1);

    exec!(chip addv 1, 2);
    assert_eq!(reg!(chip 1), 5);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn sub() {
    let mut chip = chip_preset!(0 = 3, 1 = 1, 2 = 2);

    exec!(chip sub 0, 1);
    assert_eq!(reg!(chip 0), 2);
    assert_eq!(reg!(chip REG_FLAG), 1);

    exec!(chip sub 1, 2);
    assert_eq!(reg!(chip 1), 255);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn shr() {
    let mut chip = chip_preset!(0 = 0b00000101);

    exec!(chip shr 0);
    assert_eq!(reg!(chip 0), 0b00000010);
    assert_eq!(reg!(chip REG_FLAG), 1);

    exec!(chip shr 0);
    assert_eq!(reg!(chip 0), 0b00000001);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn subn() {
    let mut chip = chip_preset!(0 = 3, 1 = 1, 2 = 2);

    exec!(chip subn 1, 0);
    assert_eq!(reg!(chip 1), 2);
    assert_eq!(reg!(chip REG_FLAG), 1);

    exec!(chip subn 0, 2);
    assert_eq!(reg!(chip 0), 255);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn shl() {
    let mut chip = chip_preset!(0 = 0b10100000);

    exec!(chip shl 0);
    assert_eq!(reg!(chip 0), 0b01000000);
    assert_eq!(reg!(chip REG_FLAG), 1);

    exec!(chip shl 0);
    assert_eq!(reg!(chip 0), 0b10000000);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn snev() {
    let mut chip = chip_preset!(0 = 1, 1 = 2, 2 = 1);

    chip.mem().pc = 0;
    exec!(chip snev 0, 1);
    assert_eq!(chip.mem().pc, 2 * INST_STEP);

    chip.mem().pc = 0;
    exec!(chip snev 0, 2);
    assert_eq!(chip.mem().pc, INST_STEP);
}

#[test]
fn ldi() {
    let mut chip = chip!();

    exec!(chip ldi 0x123);
    assert_eq!(chip.mem().i, 0x123);
}

#[test]
fn jp0() {
    let mut chip = chip_preset!(0 = 3);

    exec!(chip jp0 0x120);
    assert_eq!(chip.mem().pc, 0x123);
}

#[test]
fn rnd() {
    let mut chip = chip!(hw => {
        hw.rng().set_sequence([3, 2, 5].to_vec());
    });

    exec!(chip rnd 0, 0xFF);
    assert_eq!(reg!(chip 0), 3);

    exec!(chip rnd 0, 0xFF);
    assert_eq!(reg!(chip 0), 2);

    exec!(chip rnd 0, 0x04);
    assert_eq!(reg!(chip 0), 4);
}

#[test]
fn drw() {
    let mut chip = chip!();

    let data = [0x01, 0x02, 0x03, 0x04];
    let x = 5;
    let y = 10;

    chip.screen().set_collision(true);
    chip.mem().ram.load(0x300, &data).unwrap();
    chip.mem().i = 0x300;

    reg!(chip 0 => x);
    reg!(chip 1 => y);
    exec!(chip drw 0, 1, 4);

    assert_eq!(reg!(chip REG_FLAG), 1);
    assert_eq!(
        chip.screen().draws,
        vec![MockDraw::Draw {
            x,
            y,
            data: data.to_vec()
        }]
    );

    chip.screen().set_collision(false);
    exec!(chip drw 0, 1, 4);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn skp() {
    let mut chip = chip!(hw => {
        hw.keypad().set_sequence([Some(1), Some(2)].to_vec());
    });

    chip.mem().pc = 0;
    reg!(chip 0 => 1);
    exec!(chip skp 0);
    assert_eq!(chip.mem().pc, 2 * INST_STEP);

    chip.mem().pc = 0;
    exec!(chip skp 0);
    assert_eq!(chip.mem().pc, INST_STEP);
}

#[test]
fn sknp_x() {
    let mut chip = chip!(hw => {
        hw.keypad().set_sequence([Some(1), Some(2)].to_vec());
    });

    chip.mem().pc = 0;
    reg!(chip 0 => 1);
    exec!(chip sknp 0);
    assert_eq!(chip.mem().pc, INST_STEP);

    chip.mem().pc = 0;
    exec!(chip sknp 0);
    assert_eq!(chip.mem().pc, 2 * INST_STEP);
}

#[test]
fn lddtv() {
    let mut chip = chip!();

    chip.mem().dt = 123;
    exec!(chip lddtv 0);
    assert_eq!(reg!(chip 0), 123);
}

#[test]
fn ldkey() {
    let mut chip = chip!(hw => {
        hw.keypad().set_sequence([None, None, Some(1)].to_vec());
    });

    exec!(chip ldkey 0);
    assert_eq!(reg!(chip 0), 1);
}

#[test]
fn lddt() {
    let mut chip = chip_preset!(0 = 123);

    exec!(chip lddt 0);
    assert_eq!(chip.mem().dt, 123);
}

#[test]
fn ldst() {
    let mut chip = chip_preset!(0 = 123);

    exec!(chip ldst 0);
    assert_eq!(chip.mem().st, 123);
}

#[test]
fn addi() {
    let mut chip = chip_preset!(0 = 0x03);

    chip.mem().i = 0x120;
    exec!(chip addi 0);
    assert_eq!(chip.mem().i, 0x123);
}

#[test]
fn sprite() {
    let mut chip = chip_preset!(0 = 0, 1 = 0xF);
    let s0 = chip.mem().ram.to_sprite_addr(0).unwrap();
    let sf = chip.mem().ram.to_sprite_addr(0xF).unwrap();

    exec!(chip sprite 0);
    assert_eq!(chip.mem().i, s0);

    exec!(chip sprite 1);
    assert_eq!(chip.mem().i, sf);
}

#[test]
fn bcd() {
    let mut chip = chip_preset!(0 = 123);

    chip.mem().i = 0x300;
    exec!(chip bcd 0);
    assert_eq!(chip.mem().ram.read_bytes(0x300, 3).unwrap(), &[1, 2, 3]);
}

#[test]
fn sviv() {
    let mut chip = chip!();

    for vx in 0..16 {
        chip.mem().reg.set(vx, vx + 1).unwrap();
    }

    chip.mem().i = 0x300;
    exec!(chip sviv 0xF);
    assert_eq!(
        chip.mem().ram.read_bytes(0x300, 16).unwrap(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
    );

    chip.mem().i = 0x400;
    exec!(chip sviv 7);
    assert_eq!(
        chip.mem().ram.read_bytes(0x400, 16).unwrap(),
        [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0]
    )
}

#[test]
fn ldiv() {
    let mut chip = chip!();
    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    chip.mem().ram.load(0x300, &data).unwrap();
    chip.mem().i = 0x300;
    exec!(chip ldiv 0xF);

    for vx in 0..16 {
        assert_eq!(reg!(chip vx), data[vx as usize]);
    }

    chip.mem().i = 0x308;
    exec!(chip ldiv 7);

    for vx in 0..16 {
        assert_eq!(reg!(chip vx), (vx % 8) + 9);
    }
}