use crate::wmc::surface::{Canvas, Rect};
use crate::evp::event::Event;

pub trait Widget {
    fn rect(&self) -> Rect;
    fn handle(&mut self, e: &Event) -> bool;
    fn draw(&self, c: &mut Canvas, tick: u64);
}
