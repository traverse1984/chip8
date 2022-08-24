#![no_std]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub(crate) mod prelude {
    pub use std::prelude::rust_2021::*;
    pub use std::{format, print, println, write};
}

#[cfg(feature = "std")]
pub mod io;

pub mod hal;
mod program;

pub mod inst;

pub mod vm;

pub use program::*;
pub use vm::Chip8;
