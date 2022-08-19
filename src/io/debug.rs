use crate::instruction;
use crate::io::{Screen, Viewport};
use crate::vm::mem::Mem;

const FRAME: &str = "▒";

pub fn draw_frame<V: Viewport>(screen: &mut Screen<V>) {
    let width = screen.viewport().width();
    let height = screen.viewport().height() as u16;
    let offset = 1 + width as u16;

    screen.set_offset(1, 1);

    screen.print_raw(0, 0, &FRAME.repeat(width + 2));
    screen.print_raw(0, height + 1, &FRAME.repeat(width + 2));

    for y in 0..height + 1 {
        screen.print_raw(0, y, FRAME);
        screen.print_raw(offset, y, FRAME);
    }

    screen.flush();
}

pub fn draw_registers<V: Viewport>(mem: Mem, screen: &mut Screen<V>) {
    let (w, h) = screen.terminal_size();
    let viewport = screen.viewport();
    let (vw, vh) = (viewport.width(), viewport.height());

    let x = 4 + screen.viewport().width() as u16;
    let mut y = 0;

    let mut print = move |val: &str| {
        screen.print_raw(x, y, val);
        y += 1;
    };

    print(&format!("term: {}x{} ({}x{})", vw, vh, w, h));
    print("");

    let (pc, i, dt, st) = (mem.pc, mem.i, mem.dt, mem.st);
    print(&format!(" [I]: 0x{:04X}     [DT]: 0x{:02X}", i, dt));
    print(&format!("[PC]: 0x{:03X}      [ST]: 0x{:02X}", pc, st));
    print("");

    for la in 0u8..8 {
        let ra = la + 8;
        let (lv, rv) = (mem.reg.get(la).unwrap(), mem.reg.get(ra).unwrap());

        print(&format!(
            "    [V{:X}]: 0x{:02X}   [V{:X}]: 0x{:02X}",
            la, lv, ra, rv
        ));
    }

    print("");
    print("[INSTRUCTION]");

    let inst = mem.ram.read_bytes(mem.pc, 2).unwrap();
    let inst = u16::from_be_bytes([inst[0], inst[1]]);
    let (name, decoded) = instruction::reverse_inst(inst);
    print(&decoded.to_string(name));

    print("");
    print("[STACK]");

    let mut stack = mem.stack;
    for _ in 0..16 {
        if let Ok(frame) = stack.pop() {
            print(&format!("0x{:04X}", frame));
        } else {
            print("      ");
        }
    }

    if stack.depth() == 0 {
        print("<empty>");
    }
}
