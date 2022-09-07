extern crate std;
use crate::hal::{generic, BuzzerExt, KeypadExt, RngExt, ScreenExt, TimerExt};
use std::{vec, vec::Vec};

use super::Hwg;

macro_rules! chip {
    (@make $peri: expr) => {{
        let crate::hal::mocks::Peripherals {
            screen,
            keypad,
            buzzer,
            rng,
            delay
        } = $peri;

        crate::vm::Chip8::new(screen, keypad, buzzer, rng, delay)
    }};

    () => {
        chip!(@make crate::hal::mocks::Peripherals::default())
    };

    (
        $(keys = [ $($key: expr),* ] $(,)? )?
        $(rand = [ $($rand: literal),* ] $(,)? )?
    ) => {{
        let mut peripherals = crate::hal::mocks::Peripherals::default();
        $(peripherals.keypad.set_sequence([ $($key),* ].to_vec());)?
        $(peripherals.rng.set_sequence([ $($rand),* ].to_vec());)?
        chip!(@make peripherals)
    }};
}

#[derive(Debug, Clone, Default)]
pub struct Peripherals {
    pub screen: MockScreen,
    pub keypad: MockKeypad,
    pub buzzer: MockBuzzer,
    pub rng: MockRng,
    pub delay: MockDelay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScreenCommand {
    Draw { x: u8, y: u8, data: Vec<u8> },
    Clear,
}

impl ScreenCommand {
    pub fn xor(x: u8, y: u8, data: &[u8]) -> Self {
        Self::Draw {
            x,
            y,
            data: data.to_vec(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockScreen {
    pub commands: Vec<ScreenCommand>,
    pub collision: bool,
}

impl MockScreen {
    pub fn set_collision(&mut self, collision: bool) {
        self.collision = collision;
    }
}

impl ScreenExt for MockScreen {
    type Error = ();

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.commands.push(ScreenCommand::Clear);
        Ok(())
    }

    fn draw(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool, Self::Error> {
        self.commands.push(ScreenCommand::Draw {
            x,
            y,
            data: data.to_vec(),
        });

        Ok(self.collision)
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockKeypad {
    pub sequence: Vec<Option<u8>>,
}

impl MockKeypad {
    pub fn set_sequence(&mut self, mut sequence: Vec<Option<u8>>) {
        sequence.reverse();
        self.sequence = sequence;
    }
}

impl KeypadExt for MockKeypad {
    type Error = ();

    fn key_is_pressed(&self) -> Result<bool, Self::Error> {
        Ok(self.sequence.last().map_or(false, |key| key.is_some()))
    }

    fn read_key<T: TimerExt>(&mut self, _delay: &mut T) -> Result<Option<u8>, Self::Error> {
        Ok(self.sequence.pop().map_or(None, |key| key))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MockBuzzer {
    pub state: Option<bool>,
}

impl BuzzerExt for MockBuzzer {
    type Error = ();
    fn set_state(&mut self, state: bool) -> Result<(), Self::Error> {
        self.state = Some(state);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockRng {
    sequence: Vec<u8>,
    ptr: usize,
}

impl MockRng {
    pub fn set_sequence(&mut self, sequence: Vec<u8>) {
        self.sequence = sequence;
    }
}

impl RngExt for MockRng {
    type Error = ();

    fn rand(&mut self) -> Result<u8, Self::Error> {
        if self.sequence.len() > 0 {
            let rand = self.sequence[self.ptr];
            self.ptr = (self.ptr + 1) % self.sequence.len();
            Ok(rand)
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MockDelay {
    ticks: u8,
}

impl MockDelay {
    pub fn set_ticks(&mut self, ticks: u8) {
        self.ticks = ticks;
    }
}

impl TimerExt for MockDelay {
    type Error = ();
    fn delay_us(&mut self, _us: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn reset_ticks(&mut self) -> Result<u8, Self::Error> {
        let ticks = self.ticks;
        self.ticks = 0;
        Ok(ticks)
    }
}

mod tests {
    use super::*;

    #[test]
    fn screen() {
        let mut screen = MockScreen::default();
        screen.draw(1, 1, &[1, 2, 3]).unwrap();
        screen.clear().unwrap();
        screen.draw(2, 2, &[2, 3, 4]).unwrap();

        assert_eq!(
            screen.commands,
            vec![
                ScreenCommand::xor(1, 1, &[1, 2, 3]),
                ScreenCommand::Clear,
                ScreenCommand::xor(2, 2, &[2, 3, 4]),
            ],
        );

        assert_eq!(screen.collision, false);

        screen.set_collision(true);
        assert_eq!(screen.collision, true);
    }

    #[test]
    fn keypad() {
        let mut keypad = MockKeypad::default();
        let mut delay = MockDelay::default();

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

    // #[test]
    // fn chip_macro() {
    //     let chip = chip!();
    //     let (mut screen, mut keypad, _, mut rng, mut delay, _) = chip.free();

    //     assert_eq!(screen.draw(0, 0, &[0]).unwrap(), false);
    //     assert_eq!(keypad.read_key(&mut delay).unwrap(), None);
    //     assert!(rng.rand().is_err());

    //     let chip = chip! {
    //         keys = [Some(1)],
    //         rand = [1]
    //     };

    //     let (_, mut keypad, _, mut rng, mut delay, _) = chip.free();

    //     assert_eq!(keypad.read_key(&mut delay).unwrap(), Some(1));
    //     assert_eq!(rng.rand().unwrap(), 1);
    // }
}
