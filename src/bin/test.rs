use std::thread::Thread;

use chip8;
use chip8::Chip8;

use chip8::hal::*;
use chip8::io::{HalScreen, Keyboard, NilBuzzer, NilRng, ThreadDelay};
use chip8::vm::error::{Error, RuntimeError};
use chip8::{hal, hal::generic::GenericHardware, *};

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

    let looper = prog.repeat(&chip8_asm! {
        ldkey 9;
        addv 8, 9;
        sev 8, 10;
        call update;
    })?;

    prog.main(&chip8_asm! {
        ld 3, 2; // y
        ld 4, 2; // x0
        ld 5, 8; // x1
        ld 6, 14; // x2
        ld 8, 1; // Counter
        call update;
        call looper;
    })?;

    let prog = prog.compile()?;
    hexdump(prog.ram.read_bytes(0x200, 96)?);

    let chip = Chip8::new().load(prog.ram);
    let mut chip = chip.with_hardware(GenericHardware::new(
        ThreadDelay,
        HalScreen::new().unwrap(),
        Keyboard::new().unwrap(),
        NilBuzzer,
        NilRng,
    ));

    loop {
        debug::draw_frame(chip.screen());
        debug::draw_registers(*chip.state(), chip.screen());
        chip.screen().flush();

        match chip.step() {
            Ok(_) => {}
            Err(e) => match e {
                RuntimeError::Hardware(e) => println!("Hardware error"),
                RuntimeError::Software(e) => println!("{:?}", e),
            },
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
