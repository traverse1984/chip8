use super::draw::Draw;

pub trait Viewport {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn draw(&mut self, vx: u8, vy: u8, buf: &[u8]) -> Draw;
    fn clear(&mut self);
}
