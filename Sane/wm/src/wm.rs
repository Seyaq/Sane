#![no_std]
#![no_main]
#![allow(dead_code)]

extern crate alloc;

mod evp;
mod wdg;
mod wmc;

use wmc::compositor::Compositor;
use evp::pump::EventPump;

#[no_mangle]
pub extern "C" fn wm_main() -> ! {
    let mut comp = Compositor::new(1920, 1080);
    let mut pump = EventPump::new();
    comp.push_workspace(wdg::desktop::Desktop::new());
    loop {
        pump.drain(&mut comp);
        comp.tick();
        comp.render();
    }
}
