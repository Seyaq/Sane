#[derive(Clone, Copy, Debug)]
pub struct Color(pub u32);

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self(((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }
    #[inline] pub fn r(self) -> u8 { ((self.0 >> 16) & 0xFF) as u8 }
    #[inline] pub fn g(self) -> u8 { ((self.0 >>  8) & 0xFF) as u8 }
    #[inline] pub fn b(self) -> u8 { (self.0 & 0xFF) as u8 }
    #[inline] pub fn a(self) -> u8 { ((self.0 >> 24) & 0xFF) as u8 }

    pub fn lerp(self, other: Color, t: u8) -> Color {
        let t16 = t as u16;
        let inv = 255 - t16;
        Color::rgba(
            ((self.r() as u16 * inv + other.r() as u16 * t16) >> 8) as u8,
            ((self.g() as u16 * inv + other.g() as u16 * t16) >> 8) as u8,
            ((self.b() as u16 * inv + other.b() as u16 * t16) >> 8) as u8,
            ((self.a() as u16 * inv + other.a() as u16 * t16) >> 8) as u8,
        )
    }

    pub fn alpha_blend(self, dst: Color) -> Color {
        let a = self.a() as u16;
        let ia = 255 - a;
        Color::rgba(
            ((self.r() as u16 * a + dst.r() as u16 * ia) >> 8) as u8,
            ((self.g() as u16 * a + dst.g() as u16 * ia) >> 8) as u8,
            ((self.b() as u16 * a + dst.b() as u16 * ia) >> 8) as u8,
            255,
        )
    }
}

pub struct Palette;
impl Palette {
    pub const BG_DEEP:      Color = Color::rgba( 10,  10,  18, 255);
    pub const BG_SURFACE:   Color = Color::rgba( 16,  16,  28, 240);
    pub const BG_ELEVATED:  Color = Color::rgba( 22,  22,  38, 230);
    pub const ACCENT:       Color = Color::rgba( 99, 102, 241, 255);
    pub const ACCENT_GLOW:  Color = Color::rgba( 99, 102, 241, 80);
    pub const ACCENT_HOVER: Color = Color::rgba(129, 140, 248, 255);
    pub const TEXT_PRIMARY: Color = Color::rgba(240, 240, 255, 255);
    pub const TEXT_MUTED:   Color = Color::rgba(148, 148, 180, 255);
    pub const TEXT_DIM:     Color = Color::rgba( 80,  80, 110, 255);
    pub const BORDER:       Color = Color::rgba( 40,  40,  65, 200);
    pub const BORDER_FOCUS: Color = Color::rgba( 99, 102, 241, 180);
    pub const DANGER:       Color = Color::rgba(239,  68,  68, 255);
    pub const SUCCESS:      Color = Color::rgba( 34, 197,  94, 255);
    pub const WARNING:      Color = Color::rgba(245, 158,  11, 255);
    pub const GLASS:        Color = Color::rgba( 18,  18,  32, 200);
    pub const SHADOW:       Color = Color::rgba(  0,   0,   0, 120);
}
