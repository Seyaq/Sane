use crate::mem::caps::{CapRights, check};
use core::sync::atomic::{AtomicU64, Ordering};

#[repr(C, align(64))]
pub struct SharedRing {
    pub head:      AtomicU64,
    pub tail:      AtomicU64,
    pub capacity:  u64,
    pub slot_size: u32,
    _pad: [u8; 36],
}

impl SharedRing {
    #[inline] pub fn can_write(&self) -> bool {
        self.head.load(Ordering::Acquire) - self.tail.load(Ordering::Acquire) < self.capacity
    }

    #[inline] pub fn can_read(&self) -> bool {
        self.head.load(Ordering::Acquire) != self.tail.load(Ordering::Acquire)
    }

    #[inline] pub fn commit(&self)  { self.head.fetch_add(1, Ordering::Release); }
    #[inline] pub fn consume(&self) { self.tail.fetch_add(1, Ordering::Release); }

    #[inline] pub fn write_ptr(&self) -> *mut u8 {
        let offset = ((self.head.load(Ordering::Relaxed) % self.capacity) * self.slot_size as u64) as usize;
        unsafe { (self as *const _ as *mut u8).add(64 + offset) }
    }

    #[inline] pub fn read_ptr(&self) -> *const u8 {
        let offset = ((self.tail.load(Ordering::Relaxed) % self.capacity) * self.slot_size as u64) as usize;
        unsafe { (self as *const _ as *const u8).add(64 + offset) }
    }
}

pub fn create_ring(base_phys: u64, capacity: u64, slot_size: u32) -> Option<*mut SharedRing> {
    let ring = base_phys as *mut SharedRing;
    unsafe {
        (*ring).head      = AtomicU64::new(0);
        (*ring).tail      = AtomicU64::new(0);
        (*ring).capacity  = capacity;
        (*ring).slot_size = slot_size;
    }
    Some(ring)
}
