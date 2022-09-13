use crate::{
    hal::{Hardware, HardwareExt},
    mem::Mem,
};

use super::{error::RuntimeResult, Chip8};

use core::ops::{Deref, DerefMut};

pub struct HwChip8<H: HardwareExt> {
    hw: H,
    chip: Chip8,
}

impl<H: HardwareExt> HardwareExt for HwChip8<H> {
    type Error = H::Error;
    type Timer = H::Timer;
    type Screen = H::Screen;
    type Keypad = H::Keypad;
    type Buzzer = H::Buzzer;
    type Rng = H::Rng;

    fn hardware(
        &mut self,
    ) -> Hardware<'_, Self::Timer, Self::Screen, Self::Keypad, Self::Buzzer, Self::Rng> {
        self.hw.hardware()
    }
}

impl<H: HardwareExt> HwChip8<H> {
    pub(super) fn new(hw: H) -> Self {
        Self::from_chip(hw, Chip8::new())
    }

    pub(super) fn from_chip(hw: H, chip: Chip8) -> Self {
        Self { hw, chip }
    }

    #[cfg(test)]
    pub(super) fn mem(&mut self) -> &mut Mem {
        &mut self.chip.mem
    }

    pub fn split(self) -> (H, Chip8) {
        let Self { hw, chip } = self;
        (hw, chip)
    }

    pub fn hw(&mut self) -> &mut H {
        &mut self.hw
    }

    pub fn run(&mut self, hz: u32) -> RuntimeResult<H::Error> {
        let Self { chip, hw } = self;
        chip.run(hz, hw)
    }

    pub fn step(&mut self) -> RuntimeResult<H::Error> {
        let Self { chip, hw } = self;
        chip.step(hw)
    }

    pub fn exec(&mut self, inst: u16) -> RuntimeResult<H::Error> {
        let Self { chip, hw } = self;
        chip.exec(inst, hw)
    }
}

impl<H: HardwareExt> Deref for HwChip8<H> {
    type Target = Chip8;
    fn deref(&self) -> &Self::Target {
        &self.chip
    }
}

impl<H: HardwareExt> DerefMut for HwChip8<H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.chip
    }
}
