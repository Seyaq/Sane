use crate::ipc::shared::SharedRing;

#[repr(C)]
pub struct KeyReport {
    pub modifiers: u8,
    pub reserved:  u8,
    pub keys:      [u8; 6],
}

pub mod Modifier {
    pub const L_CTRL:  u8 = 0x01; pub const L_SHIFT: u8 = 0x02;
    pub const L_ALT:   u8 = 0x04; pub const L_GUI:   u8 = 0x08;
    pub const R_CTRL:  u8 = 0x10; pub const R_SHIFT: u8 = 0x20;
    pub const R_ALT:   u8 = 0x40; pub const R_GUI:   u8 = 0x80;
}

#[inline]
pub fn keycode_to_ascii(code: u8) -> Option<char> {
    match code {
        0x04..=0x1D => Some((b'a' + code - 0x04) as char),
        0x1E..=0x27 => Some((b'1' + code - 0x1E) as char),
        0x28 => Some('\n'), 0x2C => Some(' '), 0x2B => Some('\t'),
        _ => None,
    }
}

static mut EVENT_RING: Option<*mut SharedRing> = None;

pub fn start() {
    if let Some(phys) = crate::mem::pmm::alloc_frame() {
        unsafe { EVENT_RING = crate::ipc::shared::create_ring(phys as u64, 64, 8) };
    }
}

pub fn on_report(report: &KeyReport) {
    unsafe {
        if let Some(r) = EVENT_RING {
            let ring = &mut *r;
            if ring.can_write() { (ring.write_ptr() as *mut KeyReport).write_volatile(*report); ring.commit(); }
        }
    }
}

pub fn poll() -> Option<KeyReport> {
    unsafe {
        EVENT_RING.and_then(|r| {
            let ring = &mut *r;
            ring.can_read().then(|| {
                let rep = (ring.read_ptr() as *const KeyReport).read_volatile();
                ring.consume();
                rep
            })
        })
    }
}
