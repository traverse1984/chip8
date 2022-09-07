mod hal;
pub use hal::*;

#[cfg(test)]
#[macro_use]
pub mod mocks;

#[cfg(test)]
pub(crate) use chip;

#[cfg(test)]
pub use mocks::ScreenCommand;

mod generic;
mod hal2;
