mod chip8;

pub mod parser;

mod error;
mod program;

pub use self::chip8::Chip8;
pub use program::Program;
