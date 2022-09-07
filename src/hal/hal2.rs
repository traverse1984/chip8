pub trait Hardware
where
    Self: Delay + Screen + Keypad + Buzzer + Rng,
{
    type Error;
}

pub struct GenericHardware<D, S, K, B, R>
where
    D: Delay,
    S: Screen,
    K: Keypad,
    B: Buzzer,
    R: Rng,
{
    pub delay: D,
    pub screen: S,
    pub keypad: K,
    pub buzzer: B,
    pub rng: R,
}

impl<D, S, K, B, R> GenericHardware<D, S, K, B, R>
where
    D: Delay,
    S: Screen,
    K: Keypad,
    B: Buzzer,
    R: Rng,
{
    pub fn new(delay: D, screen: S, keypad: K, buzzer: B, rng: R) -> Self {
        Self {
            delay,
            screen,
            keypad,
            buzzer,
            rng,
        }
    }
}

macro_rules! generic_impl {
    ($trait: ident { $($impl: tt)* }) => {
        impl<D, S, K, B, R> $trait for GenericHardware<D, S, K, B, R>
        where
            D: Delay,
            S: Screen,
            K: Keypad,
            B: Buzzer,
            R: Rng,
        {
            $($impl)*
        }
    };
}

generic_impl!(Delay {
    type Error = D::Error;
    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        self.delay.delay_us(us)
    }
});

// impl<D, S, K, B, R> Delay for GenericHardware<D, S, K, B, R>
// where
//     D: Delay,
//     S: Screen,
//     K: Keypad,
//     B: Buzzer,
//     R: Rng,
// {
//     type Error = D::Error;
//     fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
//         self.delay.delay_us(us)
//     }
// }

/// Delay handler
pub trait Delay {
    type Error;
    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error>;
}

/// Screen
pub trait Screen {
    type Error;

    /// XOR the [&\[u8\]](`u8`) into the current display starting at position
    /// `(x,y)`, then update the display. Returns a boolean indicating whether
    /// pixels were erased by this operation.
    fn draw(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool, Self::Error>;

    /// Clear the entire display
    fn clear(&mut self) -> Result<(), Self::Error>;
}

/// Keypad
pub trait Keypad {
    type Error;

    /// Returns true if any key is pressed, false otherwise.
    fn key_is_pressed(&self) -> Result<bool, Self::Error>;

    // /// Try to determine which key is pressed (if any).
    // fn read_key<D: Delay>(&mut self, delay: &mut D) -> Result<Option<u8>, Self::Error>;
}

/// Buzzer
pub trait Buzzer {
    type Error;

    fn on(&mut self) -> Result<(), Self::Error>;
    fn off(&mut self) -> Result<(), Self::Error>;
}

/// Rng
pub trait Rng {
    type Error;

    fn random(&mut self) -> Result<u8, Self::Error>;
}
