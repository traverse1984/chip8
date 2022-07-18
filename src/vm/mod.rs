mod chip8;

pub mod mem;

pub mod parser;

mod program;
mod status;

pub use self::chip8::Chip8;
pub use program::Program;
