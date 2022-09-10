mod hal;
mod macros;

pub use hal::{BuzzerExt, Hardware, HardwareExt, KeypadExt, RngExt, ScreenExt, TimerExt};

pub mod generic {
    pub use super::hal::{
        BuzzerWrapper, GenericHardware, GenericHardwareError, KeypadWrapper, RngWrapper,
        ScreenWrapper, TimerWrapper,
    };
}

#[cfg(test)]
pub mod mocks {
    pub use super::hal::{
        MockBuzzer, MockDraw, MockHardware, MockKeypad, MockRng, MockScreen, MockTimer,
    };
}
