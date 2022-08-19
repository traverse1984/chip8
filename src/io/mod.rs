pub mod debug;
mod keyboard;
mod screen;

pub use keyboard::*;
pub use screen::*;

use crate::hal::{Buzzer, Delay, Error, Rng};

pub struct ThreadDelay;

impl Delay for ThreadDelay {
    type Error = Error;

    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        std::thread::sleep(std::time::Duration::from_micros(us.into()));
        Ok(())
    }
}

pub struct NilBuzzer;

impl Buzzer for NilBuzzer {
    type Error = Error;

    fn off(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn on(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct NilRng;

impl Rng for NilRng {
    type Error = Error;

    fn random(&mut self) -> Result<u8, Self::Error> {
        Ok(1)
    }
}
