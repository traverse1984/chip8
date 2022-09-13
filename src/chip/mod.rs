pub(crate) mod chip8;
pub mod error;
mod hw;
mod timer;

#[cfg(test)]
mod tests;

pub use self::chip8::Chip8;
pub use hw::HwChip8;
