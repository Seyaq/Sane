use super::theme::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect { pub x: i32, pub y: i32, pub w: u16, pub h: u16 }

impl Rect {
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y
            && px < self.x + self.w as i32
            && py < self.y + self.h as i32
    }
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.w as i32 && self.x + self.w as i32 > other.x
            && self.y < other.y + other.h as i32 && self.y + self.h as i32 > other.y
    }
    pub fn shrink(&self, n: i32) -> Rect {
        Rect { x: self.x + n, y: self.y + n, w: (self.w as i32 - 2 * n).max(0) as u16, h: (self.h as i32 - 2 * n).max(0) as u16 }
    }
}

pub struct Canvas<'a> {
    pub pixels: &'a mut [u32],
    pub stride: usize,
    pub width:  usize,
    pub height: usize,
}

impl<'a> Canvas<'a> {
    #[inline]
    pub fn put(&mut self, x: i32, y: i32, c: Color) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height { return; }
        let idx = y as usize * self.stride + x as usize;
        let dst = Color(self.pixels[idx]);
        self.pixels[idx] = c.alpha_blend(dst).0;
    }

    pub fn fill_rect(&mut self, r: Rect, c: Color) {
        let x0 = r.x.max(0) as usize;
        let y0 = r.y.max(0) as usize;
        let x1 = (r.x + r.w as i32).min(self.width as i32) as usize;
        let y1 = (r.y + r.h as i32).min(self.height as i32) as usize;
        for y in y0..y1 {
            for x in x0..x1 {
                let dst = Color(self.pixels[y * self.stride + x]);
                self.pixels[y * self.stride + x] = c.alpha_blend(dst).0;
            }
        }
    }

    pub fn fill_rect_solid(&mut self, r: Rect, c: Color) {
        let x0 = r.x.max(0) as usize;
        let y0 = r.y.max(0) as usize;
        let x1 = (r.x + r.w as i32).min(self.width as i32) as usize;
        let y1 = (r.y + r.h as i32).min(self.height as i32) as usize;
        let packed = c.0;
        for y in y0..y1 {
            self.pixels[y * self.stride + x0..y * self.stride + x1].fill(packed);
        }
    }

    pub fn stroke_rect(&mut self, r: Rect, c: Color, thick: i32) {
        self.fill_rect(Rect { x: r.x, y: r.y, w: r.w, h: thick as u16 }, c);
        self.fill_rect(Rect { x: r.x, y: r.y + r.h as i32 - thick, w: r.w, h: thick as u16 }, c);
        self.fill_rect(Rect { x: r.x, y: r.y, w: thick as u16, h: r.h }, c);
        self.fill_rect(Rect { x: r.x + r.w as i32 - thick, y: r.y, w: thick as u16, h: r.h }, c);
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, c: Color) {
        let r2 = radius * radius;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= r2 { self.put(cx + dx, cy + dy, c); }
            }
        }
    }

    pub fn draw_hline(&mut self, x0: i32, x1: i32, y: i32, c: Color) {
        for x in x0..x1 { self.put(x, y, c); }
    }

    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, c: Color) {
        for y in y0..y1 { self.put(x, y, c); }
    }

    pub fn gradient_v(&mut self, r: Rect, top: Color, bot: Color) {
        for dy in 0..r.h as i32 {
            let t = (dy * 255 / r.h as i32) as u8;
            let c = top.lerp(bot, t);
            self.fill_rect(Rect { x: r.x, y: r.y + dy, w: r.w, h: 1 }, c);
        }
    }

    pub fn glow_rect(&mut self, r: Rect, c: Color, spread: i32) {
        for i in (1..=spread).rev() {
            let alpha = ((spread - i + 1) * 30 / spread) as u8;
            let gc = Color::rgba(c.r(), c.g(), c.b(), alpha);
            let gr = Rect { x: r.x - i, y: r.y - i, w: r.w + 2 * i as u16, h: r.h + 2 * i as u16 };
            self.stroke_rect(gr, gc, 1);
        }
    }

    pub fn rounded_rect(&mut self, r: Rect, c: Color, radius: i32) {
        self.fill_rect(Rect { x: r.x + radius, y: r.y,              w: r.w - 2 * radius as u16, h: r.h }, c);
        self.fill_rect(Rect { x: r.x,          y: r.y + radius,     w: r.w, h: r.h - 2 * radius as u16 }, c);
        self.fill_circle(r.x + radius,              r.y + radius,              radius, c);
        self.fill_circle(r.x + r.w as i32 - radius, r.y + radius,              radius, c);
        self.fill_circle(r.x + radius,              r.y + r.h as i32 - radius, radius, c);
        self.fill_circle(r.x + r.w as i32 - radius, r.y + r.h as i32 - radius, radius, c);
    }

    pub fn blur_rect(&mut self, r: Rect, passes: usize) {
        let x0 = r.x.max(1) as usize;
        let y0 = r.y.max(1) as usize;
        let x1 = (r.x + r.w as i32 - 1).min(self.width as i32 - 1) as usize;
        let y1 = (r.y + r.h as i32 - 1).min(self.height as i32 - 1) as usize;
        for _ in 0..passes {
            for y in y0..y1 {
                for x in x0..x1 {
                    let s = self.stride;
                    let sum_r = |dx: i32, dy: i32| -> (u32, u32, u32) {
                        let p = Color(self.pixels[(y as i32 + dy) as usize * s + (x as i32 + dx) as usize]);
                        (p.r() as u32, p.g() as u32, p.b() as u32)
                    };
                    let neighbors = [(-1,0),(1,0),(0,-1),(0,1),(0,0)];
                    let mut rr = 0u32; let mut gg = 0u32; let mut bb = 0u32;
                    for (dx, dy) in neighbors { let (r,g,b) = sum_r(dx,dy); rr+=r; gg+=g; bb+=b; }
                    self.pixels[y * s + x] = Color::rgba((rr/5) as u8, (gg/5) as u8, (bb/5) as u8, 255).0;
                }
            }
        }
    }
}
