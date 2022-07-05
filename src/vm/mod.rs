mod chip8;
pub mod mem;
mod status;

pub use self::chip8::Chip8;

#[cfg(test)]
#[macro_use]
mod mocks;

#[cfg(test)]
pub(super) use chip;
