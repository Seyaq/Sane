
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

const CSPACE_SIZE: usize = 65536;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CapKind {
    Null,
    Endpoint,
    Memory,
    Irq,
    Thread,
    Notification,
}

#[derive(Clone, Copy, Debug)]
pub struct Capability {
    pub kind: CapKind,
    pub object: u64,
    pub badge: u32,
    pub rights: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum Slot {
    Free { next_free: Option<u32> },
    Occupied(Capability),
}

impl Default for Slot {
    fn default() -> Self {
        Slot::Free { next_free: None }
    }
}

struct CSpaceManager {
    slots: [Slot; CSPACE_SIZE],
    first_free: Option<u32>,
    next_unallocated: usize,
}

static CSPACE: Mutex<CSpaceManager> = Mutex::new(CSpaceManager {
    slots: [Slot::Free { next_free: None }; CSPACE_SIZE],
    first_free: None,
    next_unallocated: 1,
});

pub fn init() {}

pub fn install(slot: u32, cap: Capability) -> bool {
    if slot as usize >= CSPACE_SIZE || slot == 0 { return false; }
    if cap.kind == CapKind::Null { return false; }
    
    let mut cs = CSPACE.lock();
    match cs.slots[slot as usize] {
        Slot::Free { .. } => {
            cs.slots[slot as usize] = Slot::Occupied(cap);
            true
        }
        Slot::Occupied(_) => false,
    }
}

pub fn alloc(cap: Capability) -> Option<u32> {
    if cap.kind == CapKind::Null { return None; }
    
    let mut cs = CSPACE.lock();
    let slot_idx = if let Some(free_idx) = cs.first_free {
        if let Slot::Free { next_free } = cs.slots[free_idx as usize] {
            cs.first_free = next_free;
            free_idx
        } else {
            return None;
        }
    } else if cs.next_unallocated < CSPACE_SIZE {
        let unallocated_idx = cs.next_unallocated as u32;
        cs.next_unallocated += 1;
        unallocated_idx
    } else {
        return None;
    };

    cs.slots[slot_idx as usize] = Slot::Occupied(cap);
    Some(slot_idx)
}

#[inline]
pub fn lookup(cptr: u32) -> Option<Capability> {
    if cptr as usize >= CSPACE_SIZE || cptr == 0 { return None; }
    
    let cs = CSPACE.lock();
    match cs.slots[cptr as usize] {
        Slot::Occupied(cap) => Some(cap),
        Slot::Free { .. } => None,
    }
}

pub fn delete(cptr: u32) {
    if cptr as usize >= CSPACE_SIZE || cptr == 0 { return; }
    
    let mut cs = CSPACE.lock();
    if let Slot::Occupied(_) = cs.slots[cptr as usize] {
        cs.slots[cptr as usize] = Slot::Free { next_free: cs.first_free };
        cs.first_free = Some(cptr);
    }
}
