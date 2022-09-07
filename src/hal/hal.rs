pub trait Hwg
where
    Self::Timer: TimerExt<Error = Self::Error>,
    Self::Screen: ScreenExt<Error = Self::Error>,
    Self::Keypad: KeypadExt<Error = Self::Error>,
    Self::Buzzer: BuzzerExt<Error = Self::Error>,
    Self::Rng: RngExt<Error = Self::Error>,
{
    type Error;
    type Timer;
    type Screen;
    type Keypad;
    type Buzzer;
    type Rng;

    fn hardware(
        &mut self,
    ) -> Hardware<'_, Self::Timer, Self::Screen, Self::Keypad, Self::Buzzer, Self::Rng>;

    fn timer(&mut self) -> &mut Self::Timer {
        self.hardware().timer
    }

    fn screen(&mut self) -> &mut Self::Screen {
        self.hardware().screen
    }

    fn keypad(&mut self) -> &mut Self::Keypad {
        self.hardware().keypad
    }

    fn buzzer(&mut self) -> &mut Self::Buzzer {
        self.hardware().buzzer
    }

    fn rng(&mut self) -> &mut Self::Rng {
        self.hardware().rng
    }
}

pub struct Hardware<'a, Timer, Screen, Keypad, Buzzer, Rng>
where
    Timer: TimerExt,
    Screen: ScreenExt,
    Keypad: KeypadExt,
    Buzzer: BuzzerExt,
    Rng: RngExt,
{
    pub timer: &'a mut Timer,
    pub screen: &'a mut Screen,
    pub keypad: &'a mut Keypad,
    pub buzzer: &'a mut Buzzer,
    pub rng: &'a mut Rng,
}

pub trait TimerExt {
    type Error;

    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error>;
    fn reset_ticks(&mut self) -> Result<u8, Self::Error>;
}

/// Screen
pub trait ScreenExt {
    type Error;
    /// XOR the [&\[u8\]](`u8`) into the current display starting at position
    /// `(x,y)`, then update the display. Returns a boolean indicating whether
    /// pixels were erased by this operation.
    fn draw(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool, Self::Error>;
    /// Clear the entire display
    fn clear(&mut self) -> Result<(), Self::Error>;
}

/// Keypad
pub trait KeypadExt {
    type Error;
    /// Returns true if any key is pressed, false otherwise.
    fn key_is_pressed(&self) -> Result<bool, Self::Error>;
    /// Try to determine which key is pressed (if any).
    fn read_key<T: TimerExt>(&mut self, timer: &mut T) -> Result<Option<u8>, Self::Error>;
}

/// Buzzer
pub trait BuzzerExt {
    type Error;
    fn set_state(&mut self, state: bool) -> Result<(), Self::Error>;
}

/// Rng
pub trait RngExt {
    type Error;
    fn rand(&mut self) -> Result<u8, Self::Error>;
}
