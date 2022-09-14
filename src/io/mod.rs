pub mod debug;
mod keyboard;
mod screen;

pub use keyboard::*;
pub use screen::*;

use crate::hal::{BuzzerExt, DelayExt, RngExt};

pub struct ThreadDelay;

impl DelayExt for ThreadDelay {
    type Error = ();

    fn delay_micros(&mut self, us: u32) -> Result<(), Self::Error> {
        std::thread::sleep(std::time::Duration::from_micros(us.into()));
        Ok(())
    }
}

pub struct NilBuzzer;

impl BuzzerExt for NilBuzzer {
    type Error = ();

    fn set_state(&mut self, state: bool) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct NilRng;

impl RngExt for NilRng {
    type Error = ();

    fn rand(&mut self) -> Result<u8, Self::Error> {
        Ok(1)
    }
}
