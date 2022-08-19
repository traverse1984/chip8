use std::ops::{Deref, DerefMut};

use super::Screen;
use super::{draw::Draw, viewport::*};

pub struct HalScreen {
    screen: Screen<Sc64x32>,
}

impl HalScreen {
    pub fn new() -> Result<Self, String> {
        Screen::new(Sc64x32::new()).map(|screen| Self { screen })
    }
}

impl Deref for HalScreen {
    type Target = Screen<Sc64x32>;
    fn deref(&self) -> &Self::Target {
        &self.screen
    }
}

impl DerefMut for HalScreen {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.screen
    }
}

impl crate::hal::Screen for HalScreen {
    type Error = crate::hal::Error;

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.screen.clear();
        Ok(())
    }

    fn draw(&mut self, x: u8, y: u8, data: &[u8]) -> Result<bool, Self::Error> {
        Ok(self.screen.draw(x, y, data))
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Sc64x32 {
    gd: [u64; 32],
}

impl Sc64x32 {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Viewport for Sc64x32 {
    fn width(&self) -> usize {
        64
    }

    fn height(&self) -> usize {
        32
    }

    fn clear(&mut self) {
        self.gd = [0; 32];
    }

    fn draw(&mut self, vx: u8, vy: u8, buf: &[u8]) -> Draw {
        let mut draw = Draw::new(buf.len());

        for (y, &scan) in buf.iter().enumerate() {
            let ypos = (y + vy as usize) % 32;
            let curr = self.gd[ypos];
            let sprite = (scan as u64).rotate_right(8 + (vx as u32));
            let next = curr ^ sprite;

            if next != curr {
                let mut scan = String::with_capacity(64);
                for idx in (0..64).rev() {
                    scan.push_str(if (next >> idx) & 1 == 1 { "█" } else { " " })
                }

                draw.scans.push((ypos, scan));
                self.gd[ypos] = next;

                if curr & next != curr {
                    draw.collision = true;
                }
            }
        }

        draw
    }
}
