use crate::hal::KeypadExt;
use device_query::{DeviceQuery, DeviceState, Keycode};

use std::panic;
use std::prelude::rust_2021::*;

pub enum Key {
    Symbol(u8),
    Exit,
}

pub struct Keyboard {
    dev: DeviceState,
}

impl KeypadExt for Keyboard {
    type Error = ();

    fn key_is_pressed(&mut self) -> Result<bool, Self::Error> {
        Ok(self.query().is_some())
    }

    fn read_key<D: crate::hal::TimerExt>(
        &mut self,
        delay: &mut D,
    ) -> Result<Option<u8>, Self::Error> {
        delay.delay_us(1000).ok();
        match self.query() {
            Some(Key::Symbol(key)) => Ok(Some(key)),
            Some(Key::Exit) => std::process::exit(0),
            None => Ok(None),
        }
    }
}

impl Keyboard {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            dev: Self::try_device()?,
        })
    }

    pub fn query(&self) -> Option<Key> {
        let keys = self.dev.get_keys();
        for key in keys.into_iter().rev() {
            if let Some(event) = Self::keycode_to_event(key) {
                return Some(event);
            }
        }
        None
    }

    fn try_device() -> Result<DeviceState, String> {
        let orig_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));

        let dev = panic::catch_unwind(|| {
            let device_state = DeviceState::new();
            device_state.get_keys();
            device_state
        })
        .map_err(|_| String::from("This program needs to read keyboard input."));

        panic::set_hook(orig_hook);
        dev
    }

    fn keycode_to_event(key: Keycode) -> Option<Key> {
        use Keycode::*;
        let value: u8 = match key {
            Escape => 0xFE,
            Key0 => 0x0,
            Key1 => 0x1,
            Key2 => 0x2,
            Key3 => 0x3,
            Key4 => 0x4,
            Key5 => 0x5,
            Key6 => 0x6,
            Key7 => 0x7,
            Key8 => 0x8,
            Key9 => 0x9,
            A => 0xA,
            B => 0xB,
            C => 0xC,
            D => 0xD,
            E => 0xE,
            F => 0xF,
            _ => 0xFF,
        };

        match value {
            0xFF => None,
            0xFE => Some(Key::Exit),
            _ => Some(Key::Symbol(value)),
        }
    }
}
