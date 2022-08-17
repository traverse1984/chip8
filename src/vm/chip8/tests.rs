extern crate std;
use super::Error;
use super::{INST_STEP, REG_FLAG};
use crate::hal::{chip, ScreenCommand};
use crate::vm::mem::{self, Load};
use std::vec;

/// Set multiple registers of a new chip8 mock, or read a single register.
macro_rules! reg {
    ($($reg: literal = $val: literal),+) => {{
        let mut chip = chip!();
        $(chip.mem.reg.set($reg, $val).unwrap();)+
        chip
    }};

    ($chip: ident $reg: expr) => {
        $chip.mem.reg.get($reg).unwrap()
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

    chip.exec(0x00E0).unwrap();
    let (screen, ..) = chip.free();
    assert_eq!(screen.commands, vec![ScreenCommand::Clear])
}

#[test]
fn ret() {
    let mut chip = chip!();

    chip.mem.stack.push(0x345).unwrap();
    chip.exec(0x00EE).unwrap();

    assert_eq!(chip.mem.stack.pop().unwrap_err(), mem::Error::StackEmpty);
    assert_eq!(chip.mem.pc, 0x345);
}

#[test]
fn jp_nnn() {
    let mut chip = chip!();

    chip.exec(0x1123).unwrap();
    assert_eq!(chip.mem.pc, 0x0123);

    chip.exec(0x1456).unwrap();
    assert_eq!(chip.mem.pc, 0x0456);
}

#[test]
fn call_nnn() {
    let mut chip = chip!();
    chip.mem.pc = 0x0123;
    chip.exec(0x2456).unwrap();

    assert_eq!(chip.mem.pc, 0x0456);
    assert_eq!(chip.mem.stack.pop().unwrap(), 0x0123);
}

// 3xkk
// Skip next instruction if Vx = kk.
// The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
#[test]
fn se_x_kk() {
    let mut chip = reg!(0 = 0x23);

    chip.exec(0x3023).unwrap();
    assert_eq!(chip.mem.pc, 2 * INST_STEP);

    chip.mem.pc = 0;
    chip.exec(0x3024).unwrap();
    assert_eq!(chip.mem.pc, INST_STEP);
}

// 4xkk
// Skip next instruction if Vx != kk.
// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
#[test]
fn sne_x_kk() {
    let mut chip = reg!(0 = 0x23);

    chip.exec(0x4023).unwrap();
    assert_eq!(chip.mem.pc, INST_STEP);

    chip.mem.pc = 0;
    chip.exec(0x4024).unwrap();
    assert_eq!(chip.mem.pc, 2 * INST_STEP);
}

// 5xy0
// Skip next instruction if Vx = Vy.
// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
#[test]
fn se_x_y() {
    let mut chip = reg!(0 = 0x23, 1 = 0x23, 2 = 0x34);

    chip.exec(0x5010).unwrap();
    assert_eq!(chip.mem.pc, 2 * INST_STEP);

    chip.mem.pc = 0;
    chip.exec(0x5020).unwrap();
    assert_eq!(chip.mem.pc, INST_STEP);
}

// 6xkk - LD Vx, byte
// Set Vx = kk.
// The interpreter puts the value kk into register Vx.
#[test]
fn ld_x_kk() {
    let mut chip = chip!();

    chip.exec(0x6012).unwrap();
    assert_eq!(reg!(chip 0), 0x12);

    chip.exec(0x6E34).unwrap();
    assert_eq!(reg!(chip 0xE), 0x34);
}

// 7xkk - ADD Vx, byte
// Set Vx = Vx + kk.
// Adds the value kk to the value of register Vx, then stores the result in Vx.
#[test]
fn add_x_kk() {
    let mut chip = reg!(0 = 10, 1 = 254);

    chip.exec(0x7002).unwrap();
    assert_eq!(reg!(chip 0), 12);

    chip.exec(0x7102).unwrap();
    assert_eq!(reg!(chip 1), 0);
}

// 8xy0 - LD Vx, Vy
// Set Vx = Vy.
// Stores the value of register Vy in register Vx.
#[test]
fn ld_x_y() {
    let mut chip = reg!(1 = 123);

    chip.exec(0x8010).unwrap();
    assert_eq!(reg!(chip 0), 123);
    assert_eq!(reg!(chip 1), 123);
}

// 8xy1 - OR Vx, Vy
// Set Vx = Vx OR Vy.
// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
#[test]
fn or_x_y() {
    let mut chip = reg!(0 = 123, 1 = 45);

    chip.exec(0x8011).unwrap();
    assert_eq!(reg!(chip 0), 123 | 45);
}

// 8xy2 - AND Vx, Vy
// Set Vx = Vx AND Vy.
// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
#[test]
fn and_x_y() {
    let mut chip = reg!(0 = 123, 1 = 45);

    chip.exec(0x8012).unwrap();
    assert_eq!(reg!(chip 0), 123 & 45);
}

// 8xy3 - XOR Vx, Vy
// Set Vx = Vx XOR Vy.
// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
#[test]
fn xor_x_y() {
    let mut chip = reg!(0 = 123, 1 = 45);

    chip.exec(0x8013).unwrap();
    assert_eq!(reg!(chip 0), 123 ^ 45);
}

// 8xy4 - ADD Vx, Vy
// Set Vx = Vx + Vy, set VF = carry.
// The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
#[test]
fn add_x_y() {
    let mut chip = reg!(0 = 254, 1 = 2, 2 = 3);

    chip.exec(0x8014).unwrap();
    assert_eq!(reg!(chip 0), 0);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8124).unwrap();
    assert_eq!(reg!(chip 1), 5);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

// 8xy5 - SUB Vx, Vy
// Set Vx = Vx - Vy, set VF = NOT borrow.
// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
#[test]
fn sub_x_y() {
    let mut chip = reg!(0 = 3, 1 = 1, 2 = 2);

    chip.exec(0x8015).unwrap();
    assert_eq!(reg!(chip 0), 2);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8125).unwrap();
    assert_eq!(reg!(chip 1), 255);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

// 8xy6 - SHR Vx {, Vy}
// Set Vx = Vx SHR 1.
// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
#[test]
fn shr_x() {
    let mut chip = reg!(0 = 0b00000101);

    chip.exec(0x8006).unwrap();
    assert_eq!(reg!(chip 0), 0b00000010);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8006).unwrap();
    assert_eq!(reg!(chip 0), 0b00000001);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

// 8xy7 - SUBN Vx, Vy
// Set Vx = Vy - Vx, set VF = NOT borrow.
// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
#[test]
fn subn_x_y() {
    let mut chip = reg!(0 = 3, 1 = 1, 2 = 2);

    chip.exec(0x8107).unwrap();
    assert_eq!(reg!(chip 1), 2);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x8027).unwrap();
    assert_eq!(reg!(chip 0), 255);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

// 8xyE - SHL Vx {, Vy}
// Set Vx = Vx SHL 1.
// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
#[test]
fn shl_x() {
    let mut chip = reg!(0 = 0b10100000);

    chip.exec(0x800E).unwrap();
    assert_eq!(reg!(chip 0), 0b01000000);
    assert_eq!(reg!(chip REG_FLAG), 1);

    chip.exec(0x800E).unwrap();
    assert_eq!(reg!(chip 0), 0b10000000);
    assert_eq!(reg!(chip REG_FLAG), 0);
}

// 9xy0 - SNE Vx, Vy
// Skip next instruction if Vx != Vy.
// The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
#[test]
fn sne_x_y() {
    let mut chip = reg!(0 = 1, 1 = 2, 2 = 1);

    chip.exec(0x9010).unwrap();
    assert_eq!(chip.mem.pc, 2 * INST_STEP);

    chip.mem.pc = 0;
    chip.exec(0x9020).unwrap();
    assert_eq!(chip.mem.pc, INST_STEP);
}

// Aaddr - LD I, addr
// Set I = addr.
// The value of register I is set to addr.
#[test]
fn ld_i_nnn() {
    let mut chip = chip!();

    chip.exec(0xA123).unwrap();
    assert_eq!(chip.mem.i, 0x123);
}

// Baddr - JP V0, addr
// Jump to location addr + V0.
// The program counter is set to addr plus the value of V0.
#[test]
fn jp0_nnn() {
    let mut chip = reg!(0 = 3);

    chip.exec(0xB120).unwrap();
    assert_eq!(chip.mem.pc, 0x123);
}

// Cxkk - RND Vx, byte
// Set Vx = random byte AND kk.
// The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
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

// Dxyn - DRW Vx, Vy, nibble
// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
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

// Ex9E - SKP Vx
// Skip next instruction if key with the value of Vx is pressed.
// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
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

// ExA1 - SKNP Vx
// Skip next instruction if key with the value of Vx is not pressed.
// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
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

// Fx07 - LD Vx, DT
// Set Vx = delay timer value.
// The value of DT is placed into Vx.
#[test]
fn ld_x_dt() {
    let mut chip = chip!();

    chip.mem.dt = 123;
    chip.exec(0xF007).unwrap();
    assert_eq!(reg!(chip 0), 123);
}

// Fx0A - LD Vx, K
// Wait for a key press, store the value of the key in Vx.
// All execution stops until a key is pressed, then the value of that key is stored in Vx.
#[test]
fn ld_x_key() {
    let mut chip = chip!(keys = [None, None, Some(1)]);

    chip.exec(0xF00A).unwrap();
    assert_eq!(reg!(chip 0), 1);
}

// Fx15 - LD DT, Vx
// Set delay timer = Vx.
// DT is set equal to the value of Vx.
#[test]
fn ld_dt_x() {
    let mut chip = reg!(0 = 123);

    chip.exec(0xF015).unwrap();
    assert_eq!(chip.mem.dt, 123);
}

// Fx18 - LD ST, Vx
// Set sound timer = Vx.
// ST is set equal to the value of Vx.
#[test]
fn ld_st_x() {
    let mut chip = reg!(0 = 123);

    chip.exec(0xF018).unwrap();
    assert_eq!(chip.mem.st, 123);
}

// Fx1E - ADD I, Vx
// Set I = I + Vx.
// The values of I and Vx are added, and the results are stored in I.
#[test]
fn add_i_x() {
    let mut chip = reg!(0 = 0x03);

    chip.mem.i = 0x120;
    chip.exec(0xF01E).unwrap();
    assert_eq!(chip.mem.i, 0x123);
}

// Fx29 - LD F, Vx
// Set I = location of sprite for digit Vx.
// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
#[test]
fn ld_sprite_x() {
    let mut chip = reg!(0 = 0, 1 = 0xF);
    let s0 = chip.mem.ram.to_sprite_addr(0).unwrap();
    let sf = chip.mem.ram.to_sprite_addr(0xF).unwrap();

    chip.exec(0xF029).unwrap();
    assert_eq!(chip.mem.i, s0);

    chip.exec(0xF129).unwrap();
    assert_eq!(chip.mem.i, sf);
}

// Fx33 - LD B, Vx
// Store BCD representation of Vx in memory locations I, I+1, and I+2.
// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
#[test]
fn ld_bcd_x() {
    let mut chip = reg!(0 = 123);

    chip.mem.i = 0x300;
    chip.exec(0xF033).unwrap();
    assert_eq!(chip.mem.ram.read_bytes(0x300, 3).unwrap(), &[1, 2, 3]);
}

// Fx55 - LD [I], Vx
// Store registers V0 through Vx in memory starting at location I.
// The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
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

// Fx65 - LD Vx, [I]
// Read registers V0 through Vx from memory starting at location I.
// The interpreter reads values from memory starting at location I into registers V0 through Vx.
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
