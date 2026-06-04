use super::surface::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutMode { Floating, TileH, TileV, Monocle, Spiral }

#[derive(Clone, Copy, Debug)]
pub struct WindowSlot {
    pub wid:    u32,
    pub rect:   Rect,
    pub z:      u8,
    pub pinned: bool,
}

const MAX_WINDOWS: usize = 32;

pub struct Layout {
    pub mode:    LayoutMode,
    pub slots:   [Option<WindowSlot>; MAX_WINDOWS],
    pub count:   usize,
    pub focused: u32,
    pub screen:  Rect,
}

impl Layout {
    pub fn new(screen_w: u16, screen_h: u16) -> Self {
        Self {
            mode:    LayoutMode::Floating,
            slots:   [None; MAX_WINDOWS],
            count:   0,
            focused: 0,
            screen:  Rect { x: 0, y: 30, w: screen_w, h: screen_h - 30 },
        }
    }

    pub fn insert(&mut self, wid: u32, rect: Rect) {
        if self.count >= MAX_WINDOWS { return; }
        let z = self.count as u8;
        self.slots[self.count] = Some(WindowSlot { wid, rect, z, pinned: false });
        self.count += 1;
        self.focused = wid;
        if self.mode != LayoutMode::Floating { self.retile(); }
    }

    pub fn remove(&mut self, wid: u32) {
        let pos = self.slots[..self.count].iter().position(|s| s.map_or(false, |s| s.wid == wid));
        if let Some(i) = pos {
            self.slots.copy_within(i + 1..self.count, i);
            self.slots[self.count - 1] = None;
            self.count -= 1;
            if self.mode != LayoutMode::Floating { self.retile(); }
        }
    }

    pub fn focus(&mut self, wid: u32) {
        self.focused = wid;
        let max_z = self.slots[..self.count].iter().flatten().map(|s| s.z).max().unwrap_or(0);
        for s in self.slots[..self.count].iter_mut().flatten() {
            if s.wid == wid { s.z = max_z + 1; }
        }
    }

    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            LayoutMode::Floating => LayoutMode::TileH,
            LayoutMode::TileH    => LayoutMode::TileV,
            LayoutMode::TileV    => LayoutMode::Monocle,
            LayoutMode::Monocle  => LayoutMode::Spiral,
            LayoutMode::Spiral   => LayoutMode::Floating,
        };
        self.retile();
    }

    pub fn retile(&mut self) {
        let s = self.screen;
        let n = self.count;
        if n == 0 { return; }
        match self.mode {
            LayoutMode::Floating => {}
            LayoutMode::TileH => {
                let w = s.w as i32 / n as i32;
                for (i, slot) in self.slots[..n].iter_mut().flatten().enumerate() {
                    slot.rect = Rect { x: s.x + i as i32 * w, y: s.y, w: w as u16, h: s.h };
                }
            }
            LayoutMode::TileV => {
                let h = s.h as i32 / n as i32;
                for (i, slot) in self.slots[..n].iter_mut().flatten().enumerate() {
                    slot.rect = Rect { x: s.x, y: s.y + i as i32 * h, w: s.w, h: h as u16 };
                }
            }
            LayoutMode::Monocle => {
                for slot in self.slots[..n].iter_mut().flatten() {
                    slot.rect = s;
                }
            }
            LayoutMode::Spiral => {
                let mut r = s;
                for (i, slot) in self.slots[..n].iter_mut().flatten().enumerate() {
                    if i == n - 1 { slot.rect = r; break; }
                    if i % 2 == 0 {
                        let half_w = r.w / 2;
                        slot.rect = Rect { x: r.x, y: r.y, w: half_w, h: r.h };
                        r = Rect { x: r.x + half_w as i32, y: r.y, w: r.w - half_w, h: r.h };
                    } else {
                        let half_h = r.h / 2;
                        slot.rect = Rect { x: r.x, y: r.y, w: r.w, h: half_h };
                        r = Rect { x: r.x, y: r.y + half_h as i32, w: r.w, h: r.h - half_h };
                    }
                }
            }
        }
    }

    pub fn slot_at(&self, px: i32, py: i32) -> Option<u32> {
        let mut best_z = 0u8;
        let mut best_wid = None;
        for s in self.slots[..self.count].iter().flatten() {
            if s.rect.contains(px, py) && s.z >= best_z { best_z = s.z; best_wid = Some(s.wid); }
        }
        best_wid
    }

    pub fn sorted_slots(&self) -> arrayvec::ArrayVec<WindowSlot, MAX_WINDOWS> {
        let mut out: arrayvec::ArrayVec<WindowSlot, MAX_WINDOWS> = self.slots[..self.count].iter().flatten().copied().collect();
        out.sort_unstable_by_key(|s| s.z);
        out
    }
}
