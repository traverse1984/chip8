mod hal;
mod macros;

pub use hal::{BuzzerExt, DelayExt, Hardware, HardwareExt, KeypadExt, RngExt, ScreenExt};

pub mod generic {
    pub use super::hal::{
        BuzzerWrapper, DelayWrapper, GenericHardware, GenericHardwareError, KeypadWrapper,
        RngWrapper, ScreenWrapper,
    };
}

#[cfg(test)]
pub mod mocks {
    pub use super::hal::{
        MockBuzzer, MockDelay, MockDraw, MockHardware, MockKeypad, MockRng, MockScreen,
    };
}
