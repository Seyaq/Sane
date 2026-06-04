use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

const MAX_CAPS: usize = 4096;

#[derive(Clone, Copy, Debug)]
pub enum Slot {
    Free { next_free: Option<u32> },
    Occupied(MemCap),
}

impl Default for Slot {
    fn default() -> Self {
        Slot::Free { next_free: None }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MemCap {
    pub base: u64,
    pub size: u64,
    pub rights: CapRights,
    pub parent: Option<u32>,
    pub first_child: Option<u32>,
    pub next_sibling: Option<u32>,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    pub struct CapRights: u8 {
        const READ    = 1 << 0;
        const WRITE   = 1 << 1;
        const EXECUTE = 1 << 2;
        const GRANT   = 1 << 3;
    }
}

struct CapTableManager {
    table: [Slot; MAX_CAPS],
    first_free: Option<u32>,
    next_unallocated: usize,
}

static CAP_SYSTEM: Mutex<CapTableManager> = Mutex::new(CapTableManager {
    table: [Slot::Free { next_free: None }; MAX_CAPS],
    first_free: None,
    next_unallocated: 1,
});

impl CapTableManager {
    fn alloc_slot(&mut self) -> Option<u32> {
        if let Some(slot_idx) = self.first_free {
            if let Slot::Free { next_free } = self.table[slot_idx as usize] {
                self.first_free = next_free;
                return Some(slot_idx);
            }
        }
        if self.next_unallocated < MAX_CAPS {
            let slot_idx = self.next_unallocated as u32;
            self.next_unallocated += 1;
            return Some(slot_idx);
        }
        None
    }

    fn free_slot(&mut self, slot_idx: u32) {
        self.table[slot_idx as usize] = Slot::Free { next_free: self.first_free };
        self.first_free = Some(slot_idx);
    }

    unsafe fn raw_revoke(&mut self, handle: u32) {
        if let Slot::Occupied(cap) = self.table[handle as usize] {
            let mut curr_child = cap.first_child;
            while let Some(child_idx) = curr_child {
                if let Slot::Occupied(child_cap) = self.table[child_idx as usize] {
                    curr_child = child_cap.next_sibling;
                    self.raw_revoke(child_idx);
                } else {
                    break;
                }
            }
            self.free_slot(handle);
        }
    }
}

pub fn mint(base: u64, size: u64, rights: CapRights) -> Option<u32> {
    let mut mgr = CAP_SYSTEM.lock();
    let slot = mgr.alloc_slot()?;
    
    mgr.table[slot as usize] = Slot::Occupied(MemCap {
        base,
        size,
        rights,
        parent: None,
        first_child: None,
        next_sibling: None,
    });
    
    Some(slot)
}

pub fn derive(parent: u32, offset: u64, size: u64, new_rights: CapRights) -> Option<u32> {
    if parent as usize >= MAX_CAPS || parent == 0 { return None; }
    
    let mut mgr = CAP_SYSTEM.lock();
    
    let parent_cap = match mgr.table[parent as usize] {
        Slot::Occupied(cap) => cap,
        _ => return None,
    };
    
    if !parent_cap.rights.contains(CapRights::GRANT) 
        || !parent_cap.rights.contains(new_rights) 
        || offset.checked_add(size)? > parent_cap.size 
    {
        return None;
    }

    let child_slot = mgr.alloc_slot()?;
    
    let old_first_child = parent_cap.first_child;
    
    if let Slot::Occupied(mut_parent) = &mut mgr.table[parent as usize] {
        mut_parent.first_child = Some(child_slot);
    }

    mgr.table[child_slot as usize] = Slot::Occupied(MemCap {
        base: parent_cap.base + offset,
        size,
        rights: new_rights,
        parent: Some(parent),
        first_child: None,
        next_sibling: old_first_child,
    });
    
    Some(child_slot)
}

pub fn revoke(handle: u32) {
    if handle as usize >= MAX_CAPS || handle == 0 { return; }
    
    let mut mgr = CAP_SYSTEM.lock();
    
    let parent_idx = match mgr.table[handle as usize] {
        Slot::Occupied(cap) => cap.parent,
        _ => return,
    };

    if let Some(p_idx) = parent_idx {
        if let Slot::Occupied(parent_cap) = &mut mgr.table[p_idx as usize] {
            if parent_cap.first_child == Some(handle) {
                if let Slot::Occupied(curr) = mgr.table[handle as usize] {
                    parent_cap.first_child = curr.next_sibling;
                }
            } else {
                let mut prev = parent_cap.first_child;
                while let Some(prev_idx) = prev {
                    if let Slot::Occupied(prev_cap) = mgr.table[prev_idx as usize] {
                        if prev_cap.next_sibling == Some(handle) {
                            if let Slot::Occupied(curr) = mgr.table[handle as usize] {
                                if let Slot::Occupied(mut_prev) = &mut mgr.table[prev_idx as usize] {
                                    mut_prev.next_sibling = curr.next_sibling;
                                }
                            }
                            break;
                        }
                        prev = prev_cap.next_sibling;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    unsafe { mgr.raw_revoke(handle) };
}

#[inline]
pub fn check(handle: u32, addr: u64, len: u64, req: CapRights) -> bool {
    if handle as usize >= MAX_CAPS { return false; }
    
    let mgr = CAP_SYSTEM.lock();
    if let Slot::Occupied(cap) = &mgr.table[handle as usize] {
        if let Some(requested_end) = addr.checked_add(len) {
            return addr >= cap.base && requested_end <= (cap.base + cap.size) && cap.rights.contains(req);
        }
    }
    false
}
