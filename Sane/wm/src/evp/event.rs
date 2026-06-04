#[derive(Clone, Copy, Debug)]
pub enum Event {
    Key    { code: u8, modifiers: u8, pressed: bool },
    Mouse  { x: i32, y: i32, dx: i32, dy: i32, buttons: u8 },
    Wheel  { delta: i8 },
    Focus  { wid: u32, gained: bool },
    Resize { wid: u32, w: u16, h: u16 },
    Close  { wid: u32 },
    Tick,
}
