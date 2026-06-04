use super::{
    surface::{Canvas, Rect},
    theme::{Color, Palette},
    layout::Layout,
    animate::{Tween, Easing},
};
use crate::evp::event::Event;
use crate::wdg::desktop::Desktop;
use arrayvec::ArrayVec;

const MAX_FB_PIXELS: usize = 1920 * 1080;
const TITLEBAR_H:    i32   = 26;
const BORDER_W:      i32   = 1;
const TOPBAR_H:      i32   = 30;
const SHADOW_SPREAD: i32   = 8;

static mut FRAMEBUFFER: [u32; MAX_FB_PIXELS] = [0u32; MAX_FB_PIXELS];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowState { Normal, Minimised, Maximised }

pub struct Window {
    pub wid:      u32,
    pub title:    [u8; 32],
    pub title_len: usize,
    pub state:    WindowState,
    pub opacity:  Tween,
    pub scale:    Tween,
    pub pixels:   *mut u32,
    pub buf_w:    u16,
    pub buf_h:    u16,
}

pub struct Compositor {
    pub screen_w:  u16,
    pub screen_h:  u16,
    pub layout:    Layout,
    pub desktop:   Option<Desktop>,
    pub windows:   ArrayVec<Window, 32>,
    pub next_wid:  u32,
    pub cursor_x:  i32,
    pub cursor_y:  i32,
    pub drag_wid:  Option<u32>,
    pub drag_ox:   i32,
    pub drag_oy:   i32,
    pub resize_wid: Option<u32>,
    pub clock_tick: u64,
    pub wallpaper_hue: Tween,
}

impl Compositor {
    pub fn new(w: u16, h: u16) -> Self {
        Self {
            screen_w: w, screen_h: h,
            layout:   Layout::new(w, h),
            desktop:  None,
            windows:  ArrayVec::new(),
            next_wid: 1,
            cursor_x: (w / 2) as i32, cursor_y: (h / 2) as i32,
            drag_wid: None, drag_ox: 0, drag_oy: 0,
            resize_wid: None,
            clock_tick: 0,
            wallpaper_hue: Tween::new(0.0, 360.0, 30.0, Easing::Linear),
        }
    }

    pub fn push_workspace(&mut self, d: Desktop) { self.desktop = Some(d); }

    pub fn dispatch(&mut self, e: Event) {
        match e {
            Event::Mouse { x, y, dx, dy, buttons } => {
                self.cursor_x = x; self.cursor_y = y;
                if buttons & 1 != 0 {
                    if let Some(wid) = self.drag_wid {
                        if let Some(slot) = self.layout.slots.iter_mut().flatten().find(|s| s.wid == wid) {
                            slot.rect.x = x - self.drag_ox;
                            slot.rect.y = y - self.drag_oy;
                        }
                    } else {
                        let hit = self.layout.slot_at(x, y);
                        if let Some(wid) = hit {
                            if let Some(slot) = self.layout.slots.iter().flatten().find(|s| s.wid == wid) {
                                if y >= slot.rect.y && y < slot.rect.y + TITLEBAR_H {
                                    self.drag_wid = Some(wid);
                                    self.drag_ox  = x - slot.rect.x;
                                    self.drag_oy  = y - slot.rect.y;
                                    self.layout.focus(wid);
                                }
                            }
                        }
                    }
                } else { self.drag_wid = None; }
            }
            Event::Key { code, modifiers, pressed: true } => {
                if modifiers & 0x04 != 0 {
                    match code {
                        0x11 => self.layout.cycle_mode(),
                        0x13 => self.close_focused(),
                        _ => {}
                    }
                }
            }
            Event::Close { wid } => self.close_window(wid),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        const DT: f32 = 1.0 / 60.0;
        self.clock_tick = self.clock_tick.wrapping_add(1);
        self.wallpaper_hue.tick(DT);
        if self.wallpaper_hue.done() {
            self.wallpaper_hue = Tween::new(0.0, 360.0, 30.0, Easing::Linear);
        }
        for w in self.windows.iter_mut() {
            w.opacity.tick(DT);
            w.scale.tick(DT);
        }
    }

    pub fn render(&mut self) {
        let stride = self.screen_w as usize;
        let pixels = unsafe { &mut FRAMEBUFFER[..self.screen_w as usize * self.screen_h as usize] };
        let mut canvas = Canvas { pixels, stride, width: self.screen_w as usize, height: self.screen_h as usize };

        self.draw_wallpaper(&mut canvas);
        self.draw_topbar(&mut canvas);

        for slot in self.layout.sorted_slots() {
            self.draw_window_frame(&mut canvas, slot.wid, slot.rect, slot.wid == self.layout.focused);
        }

        self.draw_cursor(&mut canvas);
    }

    fn draw_wallpaper(&self, c: &mut Canvas) {
        let w = c.width as i32;
        let h = c.height as i32;
        let t = self.clock_tick as f32 / 60.0;

        for y in 0..h {
            let ny = y as f32 / h as f32;
            for x in 0..w {
                let nx = x as f32 / w as f32;
                let wave = ((nx * 6.0 + t * 0.3).sin() * 0.5 + 0.5) * 0.15;
                let wave2 = ((ny * 4.0 - t * 0.2).sin() * 0.5 + 0.5) * 0.1;
                let v = ny * 0.6 + wave + wave2;

                let r = (v * 15.0 + 8.0).min(255.0) as u8;
                let g = (v * 10.0 + 6.0).min(255.0) as u8;
                let b = (v * 30.0 + 18.0 + (t * 0.1).sin() * 5.0).min(255.0) as u8;

                let idx = y as usize * c.stride + x as usize;
                c.pixels[idx] = ((r as u32) << 16) | ((g as u32) << 8) | b as u32 | 0xFF000000;
            }
        }

        let noise_t = self.clock_tick % 120;
        for i in 0..200u32 {
            let px = ((i * 7919 + noise_t as u32 * 1234) % c.width as u32) as i32;
            let py = ((i * 6271 + noise_t as u32 * 4321) % (c.height as u32 - 30)) as i32 + 30;
            let alpha = (i % 3 + 1) as u8 * 20;
            c.put(px, py, Color::rgba(160, 170, 255, alpha));
        }
    }

    fn draw_topbar(&self, c: &mut Canvas) {
        let w = self.screen_w as i32;
        let bar = Rect { x: 0, y: 0, w: self.screen_w, h: TOPBAR_H as u16 };
        c.fill_rect_solid(bar, Palette::GLASS);

        let t = self.clock_tick as f32 / 60.0;
        for x in 0..w {
            let phase = x as f32 / w as f32;
            let wave = ((phase * 8.0 + t * 0.5).sin() * 0.5 + 0.5);
            let alpha = (wave * 15.0) as u8;
            c.put(x, TOPBAR_H - 1, Color::rgba(120, 130, 255, alpha + 30));
        }

        draw_text_pixel(c, 8, 8, b"Sane OS", Palette::TEXT_PRIMARY, 1);

        let mode_str: &[u8] = match self.layout.mode {
            crate::wmc::layout::LayoutMode::Floating => b"float",
            crate::wmc::layout::LayoutMode::TileH    => b"tile-h",
            crate::wmc::layout::LayoutMode::TileV    => b"tile-v",
            crate::wmc::layout::LayoutMode::Monocle  => b"mono",
            crate::wmc::layout::LayoutMode::Spiral   => b"spiral",
        };
        let tx = w / 2 - mode_str.len() as i32 * 4;
        draw_text_pixel(c, tx, 9, mode_str, Palette::ACCENT, 1);

        let seconds = self.clock_tick / 60;
        let hh = (seconds / 3600) % 24;
        let mm = (seconds / 60) % 60;
        let ss = seconds % 60;
        let mut time_buf = [b'0'; 8];
        time_buf[0] = b'0' + (hh / 10) as u8; time_buf[1] = b'0' + (hh % 10) as u8;
        time_buf[2] = b':';
        time_buf[3] = b'0' + (mm / 10) as u8; time_buf[4] = b'0' + (mm % 10) as u8;
        time_buf[5] = b':';
        time_buf[6] = b'0' + (ss / 10) as u8; time_buf[7] = b'0' + (ss % 10) as u8;
        draw_text_pixel(c, w - 72, 9, &time_buf, Palette::TEXT_MUTED, 1);

        let win_count = self.layout.count as u8;
        let dot_x_start = w - 110;
        for i in 0..win_count.min(8) {
            let dx = dot_x_start + i as i32 * 10;
            let active = self.layout.slots[i as usize].map_or(false, |s| s.wid == self.layout.focused);
            let col = if active { Palette::ACCENT } else { Palette::TEXT_DIM };
            c.fill_circle(dx, 15, if active { 4 } else { 2 }, col);
            if active { c.fill_circle(dx, 15, 4, Color::rgba(99, 102, 241, 40)); }
        }
    }

    fn draw_window_frame(&self, c: &mut Canvas, wid: u32, rect: Rect, focused: bool) {
        if rect.w == 0 || rect.h == 0 { return; }

        c.glow_rect(
            rect,
            if focused { Palette::ACCENT } else { Palette::SHADOW },
            if focused { SHADOW_SPREAD } else { SHADOW_SPREAD / 2 },
        );

        let bg = Palette::BG_SURFACE;
        c.rounded_rect(rect, bg, 6);

        let title_rect = Rect { x: rect.x, y: rect.y, w: rect.w, h: TITLEBAR_H as u16 };
        let tb_top = if focused { Palette::BG_ELEVATED } else { Palette::BG_SURFACE };
        let tb_bot = Palette::BG_SURFACE;
        c.gradient_v(title_rect, tb_top, tb_bot);

        if focused {
            c.draw_hline(rect.x, rect.x + rect.w as i32, rect.y, Palette::ACCENT);
        }

        let accent_bar_w = (rect.w as i32 * 30 / 100).min(60);
        let t = self.clock_tick as f32 / 60.0;
        let shift = ((t * 1.5).sin() * 5.0) as i32;
        c.fill_rect(Rect { x: rect.x + 8 + shift, y: rect.y + TITLEBAR_H - 2, w: accent_bar_w as u16, h: 1 },
            if focused { Palette::ACCENT } else { Palette::TEXT_DIM });

        let btn_y = rect.y + TITLEBAR_H / 2;
        let btn_x_close  = rect.x + rect.w as i32 - 12;
        let btn_x_max    = rect.x + rect.w as i32 - 28;
        let btn_x_min    = rect.x + rect.w as i32 - 44;
        let hover = self.cursor_x > rect.x + rect.w as i32 - 52
                 && self.cursor_y > rect.y
                 && self.cursor_y < rect.y + TITLEBAR_H;
        c.fill_circle(btn_x_close, btn_y, 5, if hover { Palette::DANGER } else { Color::rgba(60, 30, 30, 220) });
        c.fill_circle(btn_x_max,   btn_y, 5, if hover { Palette::WARNING } else { Color::rgba(40, 40, 15, 220) });
        c.fill_circle(btn_x_min,   btn_y, 5, if hover { Palette::SUCCESS } else { Color::rgba(15, 40, 20, 220) });

        c.stroke_rect(
            rect,
            if focused { Palette::BORDER_FOCUS } else { Palette::BORDER },
            BORDER_W,
        );

        let content = Rect {
            x: rect.x + BORDER_W,
            y: rect.y + TITLEBAR_H,
            w: (rect.w as i32 - 2 * BORDER_W).max(0) as u16,
            h: (rect.h as i32 - TITLEBAR_H - BORDER_W).max(0) as u16,
        };
        c.fill_rect_solid(content, Palette::BG_DEEP);

        let scan_y = rect.y + TITLEBAR_H + ((self.clock_tick / 2 + wid as u64 * 17) % content.h as u64) as i32;
        c.draw_hline(content.x, content.x + content.w as i32, scan_y,
            Color::rgba(99, 102, 241, 6));

        if focused {
            c.glow_rect(Rect { x: rect.x, y: rect.y, w: rect.w, h: TITLEBAR_H as u16 },
                Palette::ACCENT_GLOW, 3);
        }
    }

    fn draw_cursor(&self, c: &mut Canvas) {
        let x = self.cursor_x;
        let y = self.cursor_y;

        c.put(x, y, Palette::TEXT_PRIMARY);
        c.put(x + 1, y, Palette::TEXT_PRIMARY);
        c.put(x, y + 1, Palette::TEXT_PRIMARY);

        c.put(x + 2, y + 2, Color::rgba(99, 102, 241, 180));
        c.put(x + 3, y + 1, Color::rgba(99, 102, 241, 120));
        c.put(x + 1, y + 3, Color::rgba(99, 102, 241, 80));

        for r in 3..=5 {
            let alpha = (6 - r) as u8 * 18;
            c.fill_circle(x, y, r, Color::rgba(99, 102, 241, alpha));
        }
    }

    fn close_window(&mut self, wid: u32) {
        self.layout.remove(wid);
        if let Some(i) = self.windows.iter().position(|w| w.wid == wid) {
            self.windows.remove(i);
        }
    }

    fn close_focused(&mut self) {
        let wid = self.layout.focused;
        if wid != 0 { self.close_window(wid); }
    }
}

fn draw_text_pixel(c: &mut Canvas, x: i32, y: i32, text: &[u8], col: Color, scale: i32) {
    let mut cx = x;
    for &b in text {
        draw_char_pixel(c, cx, y, b, col, scale);
        cx += 6 * scale;
    }
}

fn draw_char_pixel(c: &mut Canvas, x: i32, y: i32, ch: u8, col: Color, scale: i32) {
    let glyph = FONT_5X7.get(ch.wrapping_sub(0x20) as usize).copied().unwrap_or([0u8; 7]);
    for (row, bits) in glyph.iter().enumerate() {
        for col_bit in 0..5usize {
            if bits & (1 << (4 - col_bit)) != 0 {
                let px = x + col_bit as i32 * scale;
                let py = y + row as i32 * scale;
                for sy in 0..scale { for sx in 0..scale { c.put(px + sx, py + sy, col); } }
            }
        }
    }
}

static FONT_5X7: [[u8; 7]; 96] = [
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00], // ' '
    [0x04,0x04,0x04,0x04,0x00,0x04,0x00], // '!'
    [0x0A,0x0A,0x00,0x00,0x00,0x00,0x00], // '"'
    [0x0A,0x1F,0x0A,0x1F,0x0A,0x00,0x00], // '#'
    [0x04,0x0F,0x14,0x0E,0x05,0x1E,0x04], // '$'
    [0x18,0x19,0x02,0x04,0x13,0x03,0x00], // '%'
    [0x0C,0x12,0x14,0x08,0x15,0x12,0x0D], // '&'
    [0x04,0x04,0x00,0x00,0x00,0x00,0x00], // '\''
    [0x02,0x04,0x08,0x08,0x08,0x04,0x02], // '('
    [0x08,0x04,0x02,0x02,0x02,0x04,0x08], // ')'
    [0x00,0x04,0x15,0x0E,0x15,0x04,0x00], // '*'
    [0x00,0x04,0x04,0x1F,0x04,0x04,0x00], // '+'
    [0x00,0x00,0x00,0x00,0x06,0x04,0x08], // ','
    [0x00,0x00,0x00,0x1F,0x00,0x00,0x00], // '-'
    [0x00,0x00,0x00,0x00,0x00,0x06,0x06], // '.'
    [0x01,0x02,0x02,0x04,0x08,0x10,0x10], // '/'
    [0x0E,0x11,0x13,0x15,0x19,0x11,0x0E], // '0'
    [0x04,0x0C,0x04,0x04,0x04,0x04,0x0E], // '1'
    [0x0E,0x11,0x01,0x02,0x04,0x08,0x1F], // '2'
    [0x1F,0x02,0x04,0x02,0x01,0x11,0x0E], // '3'
    [0x02,0x06,0x0A,0x12,0x1F,0x02,0x02], // '4'
    [0x1F,0x10,0x1E,0x01,0x01,0x11,0x0E], // '5'
    [0x06,0x08,0x10,0x1E,0x11,0x11,0x0E], // '6'
    [0x1F,0x01,0x02,0x04,0x08,0x08,0x08], // '7'
    [0x0E,0x11,0x11,0x0E,0x11,0x11,0x0E], // '8'
    [0x0E,0x11,0x11,0x0F,0x01,0x02,0x0C], // '9'
    [0x00,0x06,0x06,0x00,0x06,0x06,0x00], // ':'
    [0x00,0x06,0x06,0x00,0x06,0x04,0x08], // ';'
    [0x02,0x04,0x08,0x10,0x08,0x04,0x02], // '<'
    [0x00,0x00,0x1F,0x00,0x1F,0x00,0x00], // '='
    [0x08,0x04,0x02,0x01,0x02,0x04,0x08], // '>'
    [0x0E,0x11,0x01,0x02,0x04,0x00,0x04], // '?'
    [0x0E,0x11,0x17,0x15,0x17,0x10,0x0E], // '@'
    [0x04,0x0A,0x11,0x11,0x1F,0x11,0x11], // 'A'
    [0x1E,0x11,0x11,0x1E,0x11,0x11,0x1E], // 'B'
    [0x0E,0x11,0x10,0x10,0x10,0x11,0x0E], // 'C'
    [0x1C,0x12,0x11,0x11,0x11,0x12,0x1C], // 'D'
    [0x1F,0x10,0x10,0x1E,0x10,0x10,0x1F], // 'E'
    [0x1F,0x10,0x10,0x1E,0x10,0x10,0x10], // 'F'
    [0x0E,0x11,0x10,0x17,0x11,0x11,0x0F], // 'G'
    [0x11,0x11,0x11,0x1F,0x11,0x11,0x11], // 'H'
    [0x0E,0x04,0x04,0x04,0x04,0x04,0x0E], // 'I'
    [0x07,0x02,0x02,0x02,0x02,0x12,0x0C], // 'J'
    [0x11,0x12,0x14,0x18,0x14,0x12,0x11], // 'K'
    [0x10,0x10,0x10,0x10,0x10,0x10,0x1F], // 'L'
    [0x11,0x1B,0x15,0x15,0x11,0x11,0x11], // 'M'
    [0x11,0x19,0x15,0x13,0x11,0x11,0x11], // 'N'
    [0x0E,0x11,0x11,0x11,0x11,0x11,0x0E], // 'O'
    [0x1E,0x11,0x11,0x1E,0x10,0x10,0x10], // 'P'
    [0x0E,0x11,0x11,0x11,0x15,0x12,0x0D], // 'Q'
    [0x1E,0x11,0x11,0x1E,0x14,0x12,0x11], // 'R'
    [0x0F,0x10,0x10,0x0E,0x01,0x01,0x1E], // 'S'
    [0x1F,0x04,0x04,0x04,0x04,0x04,0x04], // 'T'
    [0x11,0x11,0x11,0x11,0x11,0x11,0x0E], // 'U'
    [0x11,0x11,0x11,0x11,0x11,0x0A,0x04], // 'V'
    [0x11,0x11,0x11,0x15,0x15,0x15,0x0A], // 'W'
    [0x11,0x11,0x0A,0x04,0x0A,0x11,0x11], // 'X'
    [0x11,0x11,0x0A,0x04,0x04,0x04,0x04], // 'Y'
    [0x1F,0x01,0x02,0x04,0x08,0x10,0x1F], // 'Z'
    [0x0E,0x08,0x08,0x08,0x08,0x08,0x0E], // '['
    [0x10,0x08,0x08,0x04,0x02,0x02,0x01], // '\'
    [0x0E,0x02,0x02,0x02,0x02,0x02,0x0E], // ']'
    [0x04,0x0A,0x11,0x00,0x00,0x00,0x00], // '^'
    [0x00,0x00,0x00,0x00,0x00,0x00,0x1F], // '_'
    [0x08,0x04,0x00,0x00,0x00,0x00,0x00], // '`'
    [0x00,0x00,0x0E,0x01,0x0F,0x11,0x0F], // 'a'
    [0x10,0x10,0x1E,0x11,0x11,0x11,0x1E], // 'b'
    [0x00,0x00,0x0E,0x10,0x10,0x10,0x0E], // 'c'
    [0x01,0x01,0x0F,0x11,0x11,0x11,0x0F], // 'd'
    [0x00,0x00,0x0E,0x11,0x1F,0x10,0x0E], // 'e'
    [0x06,0x09,0x08,0x1C,0x08,0x08,0x08], // 'f'
    [0x00,0x0F,0x11,0x11,0x0F,0x01,0x0E], // 'g'
    [0x10,0x10,0x1E,0x11,0x11,0x11,0x11], // 'h'
    [0x04,0x00,0x04,0x04,0x04,0x04,0x04], // 'i'
    [0x02,0x00,0x02,0x02,0x02,0x12,0x0C], // 'j'
    [0x10,0x10,0x11,0x12,0x1C,0x12,0x11], // 'k'
    [0x0C,0x04,0x04,0x04,0x04,0x04,0x0E], // 'l'
    [0x00,0x00,0x1A,0x15,0x15,0x15,0x15], // 'm'
    [0x00,0x00,0x16,0x19,0x11,0x11,0x11], // 'n'
    [0x00,0x00,0x0E,0x11,0x11,0x11,0x0E], // 'o'
    [0x00,0x1E,0x11,0x11,0x1E,0x10,0x10], // 'p'
    [0x00,0x0F,0x11,0x11,0x0F,0x01,0x01], // 'q'
    [0x00,0x00,0x16,0x19,0x10,0x10,0x10], // 'r'
    [0x00,0x00,0x0E,0x10,0x0E,0x01,0x0E], // 's'
    [0x08,0x08,0x1C,0x08,0x08,0x09,0x06], // 't'
    [0x00,0x00,0x11,0x11,0x11,0x13,0x0D], // 'u'
    [0x00,0x00,0x11,0x11,0x11,0x0A,0x04], // 'v'
    [0x00,0x00,0x11,0x15,0x15,0x15,0x0A], // 'w'
    [0x00,0x00,0x11,0x0A,0x04,0x0A,0x11], // 'x'
    [0x00,0x00,0x11,0x11,0x0F,0x01,0x0E], // 'y'
    [0x00,0x00,0x1F,0x02,0x04,0x08,0x1F], // 'z'
    [0x06,0x08,0x08,0x18,0x08,0x08,0x06], // '{'
    [0x04,0x04,0x04,0x00,0x04,0x04,0x04], // '|'
    [0x0C,0x02,0x02,0x03,0x02,0x02,0x0C], // '}'
    [0x08,0x15,0x02,0x00,0x00,0x00,0x00], // '~'
    [0x00,0x00,0x00,0x00,0x00,0x00,0x00], // DEL
];
