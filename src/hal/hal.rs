use super::macros::hal;

#[cfg(test)]
extern crate std;

#[cfg(test)]
use std::vec::Vec;

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

        Mock {
            #[derive(Debug, Clone, Copy)]
            struct MockTimer {
                ticks: u8 = 0,
            }

            impl {
                pub fn set_ticks(&mut self, ticks: u8) {
                    self.ticks = ticks;
                }
            }

            trait {
                fn delay_us(&mut self, us: u32) -> Result<(), ()> {
                    Ok(())
                }

                fn reset_ticks(&mut self) -> Result<u8, ()> {
                    let ticks = self.ticks;
                    self.ticks = 0;
                    Ok(ticks)
                }
            }
        }
    }

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

        Mock {
            #[derive(Debug, Clone)]
            struct MockScreen {
                collision: bool = false,
                draws: Vec<MockDraw> = Vec::new(),
            }

            impl {
                pub fn set_collision(&mut self, collision: bool) {
                    self.collision = collision;
                }
            }

            trait {
                fn draw(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool, ()> {
                    self.draws.push(MockDraw::xor(x, y, data));
                    Ok(self.collision)
                }

                fn clear(&mut self) -> Result<(), ()> {
                    self.draws.push(MockDraw::Clear);
                    Ok(())
                }
            }
        }
    }

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

        Mock {
            #[derive(Debug, Clone)]
            struct MockKeypad {
                sequence: Vec<Option<u8>> = Vec::new(),
            }

            impl {
                pub fn set_sequence(&mut self, mut sequence: Vec<Option<u8>>) {
                    sequence.reverse();
                    self.sequence = sequence;
                }
            }

            trait {
                fn key_is_pressed(&mut self) -> Result<bool, ()> {
                    Ok(self.sequence.last().map_or(false, |key| key.is_some()))
                }

                fn read_key<Tm: TimerExt>(&mut self, timer: &mut Tm) -> Result<Option<u8>, ()> {
                    Ok(self.sequence.pop().map_or(None, |key| key))
                }
            }
        }
    }

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

        Mock {
            #[derive(Debug, Clone, Copy)]
            /// A mock buzzer
            struct MockBuzzer {
                state: Option<bool> = None,
            }

            trait {
                fn set_state(&mut self, state: bool) -> Result<(), ()> {
                    self.state = Some(state);
                    Ok(())
                }
            }
        }

    }

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

        Mock {
            #[derive(Debug, Clone)]
            struct MockRng {
                sequence: Vec<u8> = Vec::new(),
                ptr: usize = 0,
            }

            impl {
                pub fn set_sequence(&mut self, sequence: Vec<u8>) {
                    self.sequence = sequence;
                }
            }

            trait {
                fn rand(&mut self) -> Result<u8, ()> {
                    if self.sequence.len() > 0 {
                        let rand = self.sequence[self.ptr];
                        self.ptr = (self.ptr + 1) % self.sequence.len();
                        Ok(rand)
                    } else {
                        Err(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockDraw {
    Draw { x: u8, y: u8, data: Vec<u8> },
    Clear,
}

#[cfg(test)]
impl MockDraw {
    fn xor(x: u8, y: u8, data: &[u8]) -> Self {
        Self::Draw {
            x,
            y,
            data: data.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn screen() {
        let mut screen = MockScreen::default();
        screen.draw(1, 1, &[1, 2, 3]).unwrap();
        screen.clear().unwrap();
        screen.draw(2, 2, &[2, 3, 4]).unwrap();

        assert_eq!(
            screen.draws,
            vec![
                MockDraw::xor(1, 1, &[1, 2, 3]),
                MockDraw::Clear,
                MockDraw::xor(2, 2, &[2, 3, 4]),
            ],
        );

        assert_eq!(screen.collision, false);

        screen.set_collision(true);
        assert_eq!(screen.collision, true);
    }

    #[test]
    fn keypad() {
        let mut keypad = MockKeypad::default();
        let mut delay = MockTimer::default();

        assert_eq!(keypad.key_is_pressed().unwrap(), false);
        assert_eq!(keypad.read_key(&mut delay).unwrap(), None);

        keypad.set_sequence(vec![Some(1), Some(2), None, Some(3)]);

        assert_eq!(keypad.key_is_pressed().unwrap(), true);
        assert_eq!(keypad.read_key(&mut delay).unwrap(), Some(1));
        assert_eq!(keypad.read_key(&mut delay).unwrap(), Some(2));

        assert_eq!(keypad.key_is_pressed().unwrap(), false);
        assert_eq!(keypad.read_key(&mut delay).unwrap(), None);

        assert_eq!(keypad.read_key(&mut delay).unwrap(), Some(3));
        assert_eq!(keypad.read_key(&mut delay).unwrap(), None);
    }

    #[test]
    fn rng() {
        let mut rng = MockRng::default();

        assert!(rng.rand().is_err());

        rng.set_sequence(vec![1, 2, 3]);

        assert_eq!(rng.rand().unwrap(), 1);
        assert_eq!(rng.rand().unwrap(), 2);
        assert_eq!(rng.rand().unwrap(), 3);
        assert_eq!(rng.rand().unwrap(), 1);
    }

    #[test]
    fn buzzer() {
        let mut buzzer = MockBuzzer::default();

        assert_eq!(buzzer.state, None);

        buzzer.set_state(true).unwrap();
        assert_eq!(buzzer.state, Some(true));

        buzzer.set_state(false).unwrap();
        assert_eq!(buzzer.state, Some(false));
    }
}
