use core::ops::{Deref, DerefMut};

use super::hal::*;

pub enum GenericHardwareError<TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> {
    Timer(TimerErr),
    Screen(ScreenErr),
    Keypad(KeypadErr),
    Buzzer(BuzzerErr),
    Rng(RngErr),
}

pub struct GenericHardware<Timer, Screen, Keypad, Buzzer, Rng>
where
    Timer: TimerExt,
    Screen: ScreenExt,
    Keypad: KeypadExt,
    Buzzer: BuzzerExt,
    Rng: RngExt,
{
    pub timer: GenericTimerWrapper<
        Timer,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >,

    pub screen: GenericScreenWrapper<
        Screen,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >,

    pub keypad: GenericKeypadWrapper<
        Keypad,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >,

    pub buzzer: GenericBuzzerWrapper<
        Buzzer,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >,

    pub rng: GenericRngWrapper<
        Rng,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >,
}

impl<Timer, Screen, Keypad, Buzzer, Rng> GenericHardware<Timer, Screen, Keypad, Buzzer, Rng>
where
    Timer: TimerExt,
    Screen: ScreenExt,
    Keypad: KeypadExt,
    Buzzer: BuzzerExt,
    Rng: RngExt,
{
    pub fn new(timer: Timer, screen: Screen, keypad: Keypad, buzzer: Buzzer, rng: Rng) -> Self {
        Self {
            timer: GenericTimerWrapper::new(timer),
            screen: GenericScreenWrapper::new(screen),
            keypad: GenericKeypadWrapper::new(keypad),
            buzzer: GenericBuzzerWrapper::new(buzzer),
            rng: GenericRngWrapper::new(rng),
        }
    }
}

impl<Timer, Screen, Keypad, Buzzer, Rng> HardwareExt
    for GenericHardware<Timer, Screen, Keypad, Buzzer, Rng>
where
    Timer: TimerExt,
    Screen: ScreenExt,
    Keypad: KeypadExt,
    Buzzer: BuzzerExt,
    Rng: RngExt,
{
    type Error =
        GenericHardwareError<Timer::Error, Screen::Error, Keypad::Error, Buzzer::Error, Rng::Error>;

    type Timer = GenericTimerWrapper<
        Timer,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >;

    type Screen = GenericScreenWrapper<
        Screen,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >;

    type Keypad = GenericKeypadWrapper<
        Keypad,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >;

    type Buzzer = GenericBuzzerWrapper<
        Buzzer,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >;

    type Rng = GenericRngWrapper<
        Rng,
        Timer::Error,
        Screen::Error,
        Keypad::Error,
        Buzzer::Error,
        Rng::Error,
    >;

    fn hardware(
        &mut self,
    ) -> Hardware<'_, Self::Timer, Self::Screen, Self::Keypad, Self::Buzzer, Self::Rng> {
        let GenericHardware {
            timer,
            screen,
            keypad,
            buzzer,
            rng,
        } = self;

        Hardware {
            timer,
            screen,
            keypad,
            buzzer,
            rng,
        }
    }
}

macro_rules! map_generic_error {
    ($trait: ident : $res: expr) => {
        $res.map_err(|err| {
            GenericHardwareError::<TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr>::$trait(err)
        })
    };
}

macro_rules! generic_hardware {
    ($name: ident ($trait: ident: $traiterr: ident) { $($impl: tt)* }) => {
        pub struct $name<T: $trait<Error = $traiterr>, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> {
            inner: T,
            _t: core::marker::PhantomData<TimerErr>,
            _s: core::marker::PhantomData<ScreenErr>,
            _k: core::marker::PhantomData<KeypadErr>,
            _b: core::marker::PhantomData<BuzzerErr>,
            _r: core::marker::PhantomData<RngErr>,
        }

        impl<T: $trait<Error = $traiterr>, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> $name<T, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> {
            fn new(inner: T) -> Self {
                Self {
                    inner,
                    _t: core::marker::PhantomData,
                    _s: core::marker::PhantomData,
                    _k: core::marker::PhantomData,
                    _b: core::marker::PhantomData,
                    _r: core::marker::PhantomData,
                }
            }
        }

        impl<T: $trait<Error = $traiterr>, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> Deref for $name<T, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> {
            type Target = T;
            fn deref(&self) -> &T {
                &self.inner
            }
        }

        impl<T: $trait<Error = $traiterr>, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> DerefMut for $name<T, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> {
            fn deref_mut(&mut self) -> &mut T {
                &mut self.inner
            }
        }

        impl<T: $trait<Error = $traiterr>, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> $trait for $name<T, TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr> {
            type Error = GenericHardwareError<TimerErr, ScreenErr, KeypadErr, BuzzerErr, RngErr>;
            $($impl)*
        }
    };
}

generic_hardware!(GenericTimerWrapper (TimerExt: TimerErr) {
    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        map_generic_error!(Timer: self.inner.delay_us(us))
    }

    fn reset_ticks(&mut self) -> Result<u8, Self::Error> {
        map_generic_error!(Timer: self.inner.reset_ticks())
    }
});

generic_hardware!(GenericScreenWrapper (ScreenExt: ScreenErr) {
    fn clear(&mut self) -> Result<(), Self::Error> {
        map_generic_error!(Screen: self.inner.clear())
    }

    fn draw(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool, Self::Error> {
        map_generic_error!(Screen: self.inner.draw(x, y, data))
    }
});

generic_hardware!(GenericKeypadWrapper (KeypadExt: KeypadErr) {
    fn key_is_pressed(&self) -> Result<bool, Self::Error> {
        map_generic_error!(Keypad: self.inner.key_is_pressed())
    }

    fn read_key<Timer: TimerExt>(&mut self, timer: &mut Timer) -> Result<Option<u8>, Self::Error> {
        map_generic_error!(Keypad: self.inner.read_key(timer))
    }
});

generic_hardware!(GenericBuzzerWrapper (BuzzerExt: BuzzerErr) {
    fn set_state(&mut self, state: bool) -> Result<(), Self::Error> {
        map_generic_error!(Buzzer: self.inner.set_state(state))
    }
});

generic_hardware!(GenericRngWrapper (RngExt: RngErr) {
    fn rand(&mut self) -> Result<u8, Self::Error> {
        map_generic_error!(Rng: self.inner.rand())
    }
});
