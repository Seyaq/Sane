use crate::mem::caps::{CapRights, check};
use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RingError {
    QueueFull,
    QueueEmpty,
}

#[repr(C, align(64))]
pub struct SharedRing {
    pub head: AtomicU64,
    _pad0: [u8; 56], 
    pub tail: AtomicU64,
    _pad1: [u8; 56], 
    pub capacity: u64,
    pub slot_size: u32,
}

impl SharedRing {
    #[inline]
    pub fn enqueue(&self, src: *const u8) -> Result<(), RingError> {
        let h = self.head.load(Ordering::Relaxed);
        let t = self.tail.load(Ordering::Acquire);

        if h - t >= self.capacity {
            return Err(RingError::QueueFull);
        }

        let offset = (h & (self.capacity - 1)) * self.slot_size as u64;
        unsafe {
            let dest_ptr = (self as *const _ as *mut u8).add(192 + offset as usize);
            core::ptr::copy_nonoverlapping(src, dest_ptr, self.slot_size as usize);
        }

        self.head.fetch_add(1, Ordering::Release);
        Ok(())
    }

    #[inline]
    pub fn dequeue(&self, dest: *mut u8) -> Result<(), RingError> {
        let h = self.head.load(Ordering::Acquire);
        let t = self.tail.load(Ordering::Relaxed);

        if h == t {
            return Err(RingError::QueueEmpty);
        }

        let offset = (t & (self.capacity - 1)) * self.slot_size as u64;
        unsafe {
            let src_ptr = (self as *const _ as *const u8).add(192 + offset as usize);
            core::ptr::copy_nonoverlapping(src_ptr, dest, self.slot_size as usize);
        }

        self.tail.fetch_add(1, Ordering::Release);
        Ok(())
    }
}

pub fn create_ring(cap_handle: u32, base_phys: u64, capacity: u64, slot_size: u32) -> Option<*mut SharedRing> {
    if !capacity.is_power_of_two() {
        return None;
    }

    let ring_size = 192 + (capacity * slot_size as u64);
    if !check(cap_handle, base_phys, ring_size, CapRights::READ | CapRights::WRITE) {
        return None;
    }

    let ring = base_phys as *mut SharedRing;
    unsafe {
        core::ptr::write(&mut (*ring).head, AtomicU64::new(0));
        core::ptr::write(&mut (*ring).tail, AtomicU64::new(0));
        core::ptr::write(&mut (*ring).capacity, capacity);
        core::ptr::write(&mut (*ring).slot_size, slot_size);
    }
    Some(ring)
}
