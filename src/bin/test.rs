use std::thread::Thread;

use chip8;

use chip8::io::{HalScreen, Keyboard, NilBuzzer, NilRng, ThreadDelay};
use chip8::{hal, *};

use io::debug;

fn main() -> Result<(), chip8::vm::mem::Error> {
    let mut prog = Program::new();
    let bcd = prog.data(&[0, 0, 0])?;

    let update = prog.sub(&chip8_asm! {
       ldi bcd;
       ldv 10, 8;
       bcd 8;
       ldiv 2;
       cls;
       sprite 0;
       drw 4, 3, 5;
       sprite 1;
       drw 5, 3, 5;
       sprite 2;
       drw 6, 3, 5;
       ret;
    })?;

    let init = prog.sub(&chip8_asm! {
          ld 3, 2; // y
          ld 4, 2; // x0
          ld 5, 8; // x1
          ld 6, 14; // x2
          ld 8, 1; // Counter
          call update;
          jp 0x202;
    })?;

    prog.main(&chip8_asm! {
       jp init;

       ldkey 9;
       addv 8, 9;
       sev 8, 10;
       call update;

       jp 0x202;
    })?;

    let prog = prog.compile()?;
    hexdump(prog.ram.read_bytes(0x200, 96)?);

    let mut chip = Chip8::new(
        HalScreen::new().unwrap(),
        Keyboard::new().unwrap(),
        NilBuzzer,
        NilRng,
        ThreadDelay,
    )
    .load(prog.ram);

    loop {
        debug::draw_frame(chip.screen());
        debug::draw_registers(*chip.state(), chip.screen());

        chip.step().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
