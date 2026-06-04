use crate::wmc::surface::{Canvas, Rect};
use crate::evp::event::Event;

pub struct Desktop {
    pub screen_w: u16,
    pub screen_h: u16,
}

impl Desktop {
    pub fn new() -> Self { Self { screen_w: 1920, screen_h: 1080 } }
}
