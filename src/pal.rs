/// General peripheral error, indicates only which component failed.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Screen,
    Keypad,
    Buzzer,
    Delay,
}

/// Delay handler
pub trait Delay
where
    Self::Error: Into<Error>,
{
    type Error;

    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error>;
}

/// Screen
pub trait Screen
where
    Self::Error: Into<Error>,
{
    type Error;

    /// XOR the [&\[u8\]](`u8`) into the current display starting at position
    /// `(x,y)`, then update the display. Returns a boolean indicating whether
    /// pixels were erased by this operation.
    fn xor(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool, Self::Error>;

    /// Clear the entire display
    fn clear(&mut self) -> Result<(), Self::Error>;
}

/// Keypad
pub trait Keypad
where
    Self::Error: Into<Error>,
{
    type Error;

    /// Returns true if any key is pressed, false otherwise.
    fn key_is_pressed(&self) -> Result<bool, Self::Error>;

    /// Try to determine which key is pressed (if any).
    fn read_key<D: Delay>(&mut self, delay: &mut D) -> Result<Option<u8>, Self::Error>;
}

/// Buzzer
pub trait Buzzer
where
    Self::Error: Into<Error>,
{
    type Error;

    fn on(&mut self) -> Result<(), Self::Error>;
    fn off(&mut self) -> Result<(), Self::Error>;
}
