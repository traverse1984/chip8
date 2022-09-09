use super::macros::hal;

hal! {
    /// Timer
    impl timer
    where
        Timer: TimerExt,
        Screen: ScreenExt,
        Keypad: KeypadExt,
        Buzzer: BuzzerExt,
        Rng: RngExt
    {
        type Timer;
        trait TimerExt;
        struct TimerWrapper;

        /// Pause execution for a number of microseconds
        fn delay_us(&mut self, us: u32) -> ();

        /// Reset the timers 60Hz ticks to 0 and return the number
        /// that had elapsed.
        fn reset_ticks(&mut self) -> u8;
    };

    /// Screen
    impl screen
    where
        Timer: TimerExt,
        Screen: ScreenExt,
        Keypad: KeypadExt,
        Buzzer: BuzzerExt,
        Rng: RngExt
    {
        type Screen;
        trait ScreenExt;
        struct ScreenWrapper;

        /// XOR the [&\[u8\]](`u8`) into the current display starting at position
        /// `(x,y)`, then update the display. Returns a boolean indicating whether
        /// pixels were erased by this operation.
        fn draw(&mut self, x: u8, y: u8, data: &[u8]) -> bool;

        /// Clear the entire display
        fn clear(&mut self) -> ();
    };

    /// Keypad
    impl keypad
    where
        Timer: TimerExt,
        Screen: ScreenExt,
        Keypad: KeypadExt,
        Buzzer: BuzzerExt,
        Rng: RngExt
    {
        type Keypad;
        trait KeypadExt;
        struct KeypadWrapper;

        /// Returns true if any key is pressed, false otherwise.
        fn key_is_pressed(&mut self) -> bool;

        /// Try to determine which key is pressed (if any).
        fn read_key<Tm: TimerExt>(&mut self, timer: &mut Tm) -> Option<u8>;
    };

    /// Buzzer
    impl buzzer
    where
        Timer: TimerExt,
        Screen: ScreenExt,
        Keypad: KeypadExt,
        Buzzer: BuzzerExt,
        Rng: RngExt
    {
        type Buzzer;
        trait BuzzerExt;
        struct BuzzerWrapper;

        /// Set the state of the buzzer, true being on and false being off.
        fn set_state(&mut self, state: bool) -> ();
    };

    /// Rng
    impl rng
    where
        Timer: TimerExt,
        Screen: ScreenExt,
        Keypad: KeypadExt,
        Buzzer: BuzzerExt,
        Rng: RngExt
    {
        type Rng;
        trait RngExt;
        struct RngWrapper;

        /// Generate a random byte
        fn rand(&mut self) -> u8;
    };
}
