#[derive(Debug, Clone, Copy)]
pub struct Timer {
    tick: u32,
    acc: u32,
}

impl Timer {
    pub fn new(hz: u32) -> Option<Self> {
        Self::hertz_to_us(hz).map(|tick| Self { tick, acc: 0 })
    }

    pub fn update(&mut self, state: bool, add: u32) -> bool {
        if state {
            self.acc += add;
            if self.acc >= self.tick {
                self.acc -= self.tick;
                true
            } else {
                false
            }
        } else {
            self.acc = 0;
            false
        }
    }

    pub fn hertz_to_us(hz: u32) -> Option<u32> {
        if hz > 0 && hz < 1_000_000 {
            Some(1_000_000 / hz)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Timer;

    #[test]
    fn timer() {
        assert!(Timer::new(0).is_none());
        assert!(Timer::new(1_000_001).is_none());

        let mut timer = Timer::new(60).unwrap();
        let tick = timer.tick;

        assert_eq!(timer.update(true, tick - 1), false);
        assert_eq!(timer.update(true, 1), true);
        assert_eq!(timer.acc, 0);

        timer.update(true, tick - 1);
        assert_eq!(timer.update(false, 1), false);
        assert_eq!(timer.acc, 0);
    }
}
