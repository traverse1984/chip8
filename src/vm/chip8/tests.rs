extern crate std;
use super::Error;
use super::{INST_STEP, REG_FLAG};
use crate::hal::{chip, ScreenCommand};
use crate::inst::ops;
use crate::vm::mem::{self, Load};
use std::vec;

// Create a Chip8 mock with some preset registers
macro_rules! preset {
    ($($reg: literal = $val: literal),+) => {{
        let mut chip = chip!();
        $(chip.mem.reg.set($reg, $val).unwrap();)+
        chip
    }};
}

// Read a single register from the Chip8 instance
macro_rules! reg {
    ($chip: ident $reg: expr) => {
        $chip.mem.reg.get($reg).unwrap()
    };
}

// Execute an instruction on the Chip8 instance
macro_rules! exec {
    ($chip: ident $($inst: tt)*) => {
        $chip.exec( $crate::chip8_inst!($($inst)*) ).unwrap()
    };
}

// #[test]
// fn load() {
//     let mut chip = chip!();

//     chip.main(&[1u8, 2, 3, 4][..]).unwrap();
//     assert_eq!(chip.mem.ram.read_bytes(0x200, 4).unwrap(), &[1, 2, 3, 4]);

//     chip.sub(0x300, &[5, 6, 7, 8][..]).unwrap();
//     assert_eq!(chip.mem.ram.read_bytes(0x300, 4).unwrap(), &[5, 6, 7, 8]);
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
    let (screen, ..) = chip.free();
    assert_eq!(screen.commands, vec![ScreenCommand::Clear])
}

#[test]
fn ret() {
    let mut chip = chip!();
    chip.mem.stack.push(0x344).unwrap();

    exec!(chip ret);
    assert_eq!(chip.mem.stack.pop().unwrap_err(), mem::Error::StackEmpty);
    assert_eq!(chip.mem.pc, 0x346);
}

#[test]
fn jp() {
    let mut chip = chip!();

    exec!(chip jp 0x123);
    assert_eq!(chip.mem.pc, 0x0123);

    exec!(chip jp 0x456);
    assert_eq!(chip.mem.pc, 0x0456);
}

#[test]
fn call() {
    let mut chip = chip!();
    chip.mem.pc = 0x0123;

    exec!(chip call 0x456);
    assert_eq!(chip.mem.pc, 0x0456);
    assert_eq!(chip.mem.stack.pop().unwrap(), 0x0123);
}

#[test]
fn se() {
    let mut chip = preset!(0 = 0x23);

    exec!(chip se 0, 0x23);
    assert_eq!(chip.mem.pc, 2 * INST_STEP);

    chip.mem.pc = 0;
    exec!(chip se 0, 0x24);
    assert_eq!(chip.mem.pc, INST_STEP);
}

#[test]
fn sne() {
    let mut chip = preset!(0 = 0x23);

    exec!(chip sne 0, 0x23);
    assert_eq!(chip.mem.pc, INST_STEP);

    chip.mem.pc = 0;
    exec!(chip sne 0, 0x24);
    assert_eq!(chip.mem.pc, 2 * INST_STEP);
}

#[test]
fn sev() {
    let mut chip = preset!(0 = 0x23, 1 = 0x23, 2 = 0x34);

    exec!(chip sev 0, 1);
    assert_eq!(chip.mem.pc, 2 * INST_STEP);

    chip.mem.pc = 0;
    exec!(chip sev 0, 2);
    assert_eq!(chip.mem.pc, INST_STEP);
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
    let mut chip = preset!(0 = 10, 1 = 254);

    exec!(chip add 0, 0x02);
    assert_eq!(reg!(chip 0), 12);

    exec!(chip add 1, 0x02);
    assert_eq!(reg!(chip 1), 0);
}

#[test]
fn ldv() {
    let mut chip = preset!(1 = 123);

    exec!(chip ldv 0, 1);
    assert_eq!(reg!(chip 0), 123);
    assert_eq!(reg!(chip 1), 123);
}

#[test]
fn or() {
    let mut chip = preset!(0 = 123, 1 = 45);

    exec!(chip or 0, 1);
    assert_eq!(reg!(chip 0), 123 | 45);
}

#[test]
fn and_x_y() {
    let mut chip = preset!(0 = 123, 1 = 45);

    chip.exec(0x8012).unwrap();
    assert_eq!(reg!(chip 0), 123 & 45);
}

#[test]
fn xor_x_y() {
    let mut chip = preset!(0 = 123, 1 = 45);

    chip.exec(0x8013).unwrap();
    assert_eq!(reg!(chip 0), 123 ^ 45);
}

#[test]
fn add_x_y() {
    let mut chip = preset!(0 = 254, 1 = 2, 2 = 3);

    chip.exec(0x8014).unwrap();
    assert_eq!(reg!(chip 0), 0);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8124).unwrap();
    assert_eq!(reg!(chip 1), 5);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn sub_x_y() {
    let mut chip = preset!(0 = 3, 1 = 1, 2 = 2);

    chip.exec(0x8015).unwrap();
    assert_eq!(reg!(chip 0), 2);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8125).unwrap();
    assert_eq!(reg!(chip 1), 255);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn shr_x() {
    let mut chip = preset!(0 = 0b00000101);

    chip.exec(0x8006).unwrap();
    assert_eq!(reg!(chip 0), 0b00000010);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8006).unwrap();
    assert_eq!(reg!(chip 0), 0b00000001);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn subn_x_y() {
    let mut chip = preset!(0 = 3, 1 = 1, 2 = 2);

    chip.exec(0x8107).unwrap();
    assert_eq!(reg!(chip 1), 2);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8027).unwrap();
    assert_eq!(reg!(chip 0), 255);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn shl_x() {
    let mut chip = preset!(0 = 0b10100000);

    chip.exec(0x800E).unwrap();
    assert_eq!(reg!(chip 0), 0b01000000);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x800E).unwrap();
    assert_eq!(reg!(chip 0), 0b10000000);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn sne_x_y() {
    let mut chip = preset!(0 = 1, 1 = 2, 2 = 1);

    chip.exec(0x9010).unwrap();
    assert_eq!(chip.mem.pc, 2 * INST_STEP);

    chip.mem.pc = 0;
    chip.exec(0x9020).unwrap();
    assert_eq!(chip.mem.pc, INST_STEP);
}

#[test]
fn ld_i_nnn() {
    let mut chip = chip!();

    chip.exec(0xA123).unwrap();
    assert_eq!(chip.mem.i, 0x123);
}

#[test]
fn jp0_nnn() {
    let mut chip = preset!(0 = 3);

    chip.exec(0xB120).unwrap();
    assert_eq!(chip.mem.pc, 0x123);
}

#[test]
fn rnd_x_kk() {
    let mut chip = chip!(rand = [3, 2, 5]);

    chip.exec(0xC0FF).unwrap();
    assert_eq!(reg!(chip 0), 3);

    chip.exec(0xC0FF).unwrap();
    assert_eq!(reg!(chip 0), 2);

    chip.exec(0xC004).unwrap();
    assert_eq!(reg!(chip 0), 4);
}

#[test]
fn drw_x_y_n() {
    let mut chip = chip!();
    let data = [0x01, 0x02, 0x03, 0x04];
    let (x, y) = (5, 10);

    chip.screen.set_collision(true);
    chip.mem.reg.set(0, x).unwrap();
    chip.mem.reg.set(1, y).unwrap();
    chip.mem.ram.load(0x300, &data).unwrap();
    chip.mem.i = 0x300;
    chip.exec(0xD014).unwrap();

    assert_eq!(reg!(chip REG_FLAG), 1);
    assert_eq!(
        chip.screen.commands,
        vec![ScreenCommand::Draw {
            x,
            y,
            data: data.to_vec()
        }]
    );

    chip.screen.set_collision(false);
    chip.exec(0xD014).unwrap();

    assert_eq!(reg!(chip REG_FLAG), 0);
}

#[test]
fn skp_x() {
    let mut chip = chip!(keys = [Some(1), Some(2)]);

    chip.mem.reg.set(0, 1).unwrap();
    chip.exec(0xE09E).unwrap();

    assert_eq!(chip.mem.pc, 2 * INST_STEP);

    chip.mem.pc = 0;
    chip.exec(0xE09E).unwrap();
    assert_eq!(chip.mem.pc, INST_STEP);
}

#[test]
fn sknp_x() {
    let mut chip = chip!(keys = [Some(1), Some(2)]);

    chip.mem.reg.set(0, 1).unwrap();
    chip.exec(0xE0A1).unwrap();

    assert_eq!(chip.mem.pc, INST_STEP);

    chip.mem.pc = 0;
    chip.exec(0xE0A1).unwrap();
    assert_eq!(chip.mem.pc, 2 * INST_STEP);
}

#[test]
fn ld_x_dt() {
    let mut chip = chip!();

    chip.mem.dt = 123;
    chip.exec(0xF007).unwrap();
    assert_eq!(reg!(chip 0), 123);
}

#[test]
fn ld_x_key() {
    let mut chip = chip!(keys = [None, None, Some(1)]);

    chip.exec(0xF00A).unwrap();
    assert_eq!(reg!(chip 0), 1);
}

#[test]
fn ld_dt_x() {
    let mut chip = preset!(0 = 123);

    chip.exec(0xF015).unwrap();
    assert_eq!(chip.mem.dt, 123);
}

#[test]
fn ld_st_x() {
    let mut chip = preset!(0 = 123);

    chip.exec(0xF018).unwrap();
    assert_eq!(chip.mem.st, 123);
}

#[test]
fn add_i_x() {
    let mut chip = preset!(0 = 0x03);

    chip.mem.i = 0x120;
    chip.exec(0xF01E).unwrap();
    assert_eq!(chip.mem.i, 0x123);
}

#[test]
fn ld_sprite_x() {
    let mut chip = preset!(0 = 0, 1 = 0xF);
    let s0 = chip.mem.ram.to_sprite_addr(0).unwrap();
    let sf = chip.mem.ram.to_sprite_addr(0xF).unwrap();

    chip.exec(0xF029).unwrap();
    assert_eq!(chip.mem.i, s0);

    chip.exec(0xF129).unwrap();
    assert_eq!(chip.mem.i, sf);
}

#[test]
fn ld_bcd_x() {
    let mut chip = preset!(0 = 123);

    chip.mem.i = 0x300;
    chip.exec(0xF033).unwrap();
    assert_eq!(chip.mem.ram.read_bytes(0x300, 3).unwrap(), &[1, 2, 3]);
}

#[test]
fn ld_i_x() {
    let mut chip = chip!();

    for vx in 0..16 {
        chip.mem.reg.set(vx, vx + 1).unwrap();
    }

    chip.mem.i = 0x300;
    chip.exec(0xFF55).unwrap();
    assert_eq!(
        chip.mem.ram.read_bytes(0x300, 16).unwrap(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
    );

    chip.mem.i = 0x400;
    chip.exec(0xF755).unwrap();
    assert_eq!(
        chip.mem.ram.read_bytes(0x400, 16).unwrap(),
        [1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0]
    )
}

#[test]
fn ld_x_i() {
    let mut chip = chip!();
    let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    chip.mem.ram.load(0x300, &data).unwrap();
    chip.mem.i = 0x300;
    chip.exec(0xFF65).unwrap();

    for vx in 0..16 {
        assert_eq!(reg!(chip vx), data[vx as usize]);
    }

    chip.mem.i = 0x308;
    chip.exec(0xF765).unwrap();

    for vx in 0..16 {
        assert_eq!(reg!(chip vx), (vx % 8) + 9);
    }
}
