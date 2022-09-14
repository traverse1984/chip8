use super::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    delay: u32,
    acc: u32,
}

impl Default for Clock {
    fn default() -> Self {
        Self { delay: 500, acc: 0 }
    }
}

impl Clock {
    const MACRO_TICK: u32 = 16666;

    pub fn new(hertz: u32) -> Result<Self, Error> {
        if hertz >= 60 && hertz <= 1000000 {
            Ok(Self {
                delay: 1000000 / hertz,
                acc: 0,
            })
        } else {
            Err(Error::ClockSpeed(hertz))
        }
    }

    pub fn delay(&mut self) -> u32 {
        self.delay
    }

    pub fn tick(&mut self) -> bool {
        self.acc += self.delay;

        if self.acc >= Self::MACRO_TICK {
            self.acc = self.acc - Self::MACRO_TICK;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Clock;
    use crate::error::Error;

    #[test]
    fn clock() {
        assert_eq!(Clock::new(59).unwrap_err(), Error::ClockSpeed(59));
        assert_eq!(
            Clock::new(1_000_001).unwrap_err(),
            Error::ClockSpeed(1_000_001)
        );

        let mut clock = Clock::new(120).unwrap();

        assert_eq!(clock.tick(), false);
        assert_eq!(clock.tick(), true);
        assert_eq!(clock.tick(), false);
    }
}
