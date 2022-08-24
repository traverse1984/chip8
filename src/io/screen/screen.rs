use crate::prelude::*;
use std::io::{self, Read, Stdout, Write};
use termion::{
    clear, cursor,
    raw::{IntoRawMode, RawTerminal},
};

use super::viewport::Viewport;

pub struct Screen<V: Viewport> {
    stdout: RawTerminal<Stdout>,
    offset: (u16, u16),
    viewport: V,
    blank: String,
}

impl<V: Viewport> Screen<V> {
    pub fn new(viewport: V) -> Result<Self, String> {
        let mut stdout = io::stdout()
            .into_raw_mode()
            .or(Err(String::from("Could not open output in raw mode.")))?;

        print!("{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Hide);

        stdout
            .flush()
            .or(Err(String::from("IO error when constructing screen.")))?;

        let blank = " ".repeat(viewport.width());

        Ok(Self {
            stdout,
            viewport,
            blank,
            offset: (0, 0),
        })
    }

    pub fn clean_exit(&mut self) {
        io::stdin()
            .take(u64::MAX)
            .read_vectored(&mut Vec::new())
            .ok();

        self.stdout
            .suspend_raw_mode()
            .expect("Failed to exit terminal raw mode.");

        print!(
            "{}{}{}{}",
            clear::All,
            cursor::Goto(1, 1),
            cursor::Show,
            clear::All
        );

        self.flush();
    }

    pub fn terminal_size(&self) -> (u16, u16) {
        termion::terminal_size().unwrap()
    }

    pub fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }

    fn scan(&self, x: u16, y: u16, scan: &str) {
        self.print_raw(
            x.saturating_add(self.offset.0),
            y.saturating_add(self.offset.1),
            scan,
        )
    }

    pub fn clear(&mut self) {
        self.viewport.clear();

        for y in 0..self.viewport.height() {
            self.scan(0, y as u16, &self.blank);
        }

        self.flush();
    }

    pub fn draw(&mut self, vx: u8, vy: u8, buf: &[u8]) -> bool {
        let draw = self.viewport.draw(vx, vy, buf);
        for (y, scan) in draw.scans {
            self.scan(0, y as u16, &scan);
        }

        self.flush();
        draw.collision
    }

    pub fn viewport(&self) -> &V {
        &self.viewport
    }

    pub fn offset(&self) -> (u16, u16) {
        self.offset
    }

    pub fn set_offset(&mut self, x: u16, y: u16) {
        self.offset = (x, y)
    }

    pub fn print_raw(&self, x: u16, y: u16, str: &str) {
        let (tw, th) = self.terminal_size();

        if y < th {
            let max = tw.saturating_sub(x) as usize;
            let goto = cursor::Goto(x.saturating_add(1), y.saturating_add(1));

            if str.chars().count() > max {
                print!("{}{}", goto, str.chars().take(max).collect::<String>());
            } else {
                print!("{}{}", goto, str);
            };
        }
    }
}
