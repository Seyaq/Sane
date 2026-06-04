use spin::Mutex;

const CSPACE_SIZE: usize = 65536;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CapKind { Null, Endpoint, Memory, Irq, Thread, Notification }

#[derive(Clone, Copy, Debug)]
pub struct Capability {
    pub kind:   CapKind,
    pub object: u64,
    pub badge:  u32,
    pub rights: u8,
}

impl Default for Capability {
    fn default() -> Self { Self { kind: CapKind::Null, object: 0, badge: 0, rights: 0 } }
}

static CSPACE:    Mutex<[Capability; CSPACE_SIZE]> = Mutex::new(
    [Capability { kind: CapKind::Null, object: 0, badge: 0, rights: 0 }; CSPACE_SIZE]
);
static mut NEXT_SLOT: u32 = 1;

pub fn init() {}

pub fn install(slot: u32, cap: Capability) -> bool {
    if slot as usize >= CSPACE_SIZE { return false; }
    let mut cs = CSPACE.lock();
    if cs[slot as usize].kind != CapKind::Null { return false; }
    cs[slot as usize] = cap;
    true
}

pub fn alloc(cap: Capability) -> Option<u32> {
    let slot = unsafe {
        if NEXT_SLOT as usize >= CSPACE_SIZE { return None; }
        let s = NEXT_SLOT; NEXT_SLOT += 1; s
    };
    install(slot, cap);
    Some(slot)
}

#[inline]
pub fn lookup(cptr: u32) -> Option<Capability> {
    if cptr as usize >= CSPACE_SIZE { return None; }
    let cs = CSPACE.lock();
    let cap = cs[cptr as usize];
    if cap.kind == CapKind::Null { None } else { Some(cap) }
}

pub fn delete(cptr: u32) {
    if cptr as usize >= CSPACE_SIZE { return; }
    CSPACE.lock()[cptr as usize] = Capability::default();
}
