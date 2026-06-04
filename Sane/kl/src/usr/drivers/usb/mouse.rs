#[repr(C)]
pub struct MouseReport {
    pub buttons: u8,
    pub dx:      i8,
    pub dy:      i8,
    pub wheel:   i8,
}

pub mod Button {
    pub const LEFT:   u8 = 1 << 0;
    pub const RIGHT:  u8 = 1 << 1;
    pub const MIDDLE: u8 = 1 << 2;
}

static mut CURSOR_X: i32 = 0;
static mut CURSOR_Y: i32 = 0;
static mut BUTTONS:  u8  = 0;

const SCREEN_W: i32 = 1920;
const SCREEN_H: i32 = 1080;

pub fn start() {}

#[inline]
pub fn on_report(r: &MouseReport) {
    unsafe {
        CURSOR_X = (CURSOR_X + r.dx as i32).clamp(0, SCREEN_W - 1);
        CURSOR_Y = (CURSOR_Y + r.dy as i32).clamp(0, SCREEN_H - 1);
        BUTTONS  = r.buttons;
    }
}

#[inline] pub fn cursor()  -> (i32, i32) { unsafe { (CURSOR_X, CURSOR_Y) } }
#[inline] pub fn buttons() -> u8         { unsafe { BUTTONS } }
