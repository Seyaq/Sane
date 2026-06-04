
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

pub const PAGE_SIZE: usize = 4096;
pub const MAX_ORDERS: usize = 11;

struct FreeBlock {
    next: *mut FreeBlock,
}

pub struct BuddyAllocator {
    buckets: [*mut FreeBlock; MAX_ORDERS],
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self { buckets: [core::ptr::null_mut(); MAX_ORDERS] }
    }

    pub unsafe fn add_mem(&mut self, mut base: usize, mut size: usize) {
        if base % PAGE_SIZE != 0 {
            let diff = PAGE_SIZE - (base % PAGE_SIZE);
            base += diff;
            size = size.saturating_sub(diff);
        }
        size &= !(PAGE_SIZE - 1);

        let mut current_addr = base;
        let end_addr = base + size;

        while current_addr < end_addr {
            let mut order = MAX_ORDERS - 1;
            while order > 0 {
                let block_size = PAGE_SIZE << order;
                if current_addr % block_size == 0 && current_addr + block_size <= end_addr {
                    break;
                }
                order -= 1;
            }

            self.free_block(current_addr, order);
            current_addr += PAGE_SIZE << order;
            FREE_PAGES.fetch_add(1 << order, Ordering::Relaxed);
            TOTAL_PAGES.fetch_add(1 << order, Ordering::Relaxed);
        }
    }

    pub fn alloc_pages(&mut self, order: usize) -> Option<usize> {
        if order >= MAX_ORDERS { return None; }

        for current_order in order..MAX_ORDERS {
            let head = self.buckets[current_order];
            if !head.is_null() {
                unsafe {
                    self.buckets[current_order] = (*head).next;
                    
                    let mut addr = head as usize;
                    for split_order in (order..current_order).rev() {
                        let buddy = addr + (PAGE_SIZE << split_order);
                        self.free_block(buddy, split_order);
                    }
                    
                    FREE_PAGES.fetch_sub(1 << order, Ordering::Relaxed);
                    return Some(addr);
                }
            }
        }
        None
    }

    unsafe fn free_block(&mut self, addr: usize, order: usize) {
        if order >= MAX_ORDERS { return; }

        let block_size = PAGE_SIZE << order;
        let buddy_addr = addr ^ block_size;

        let mut prev: *mut *mut FreeBlock = &mut self.buckets[order];
        let mut curr = self.buckets[order];
        let mut buddy_found = false;

        while !curr.is_null() {
            if curr as usize == buddy_addr {
                *prev = (*curr).next;
                buddy_found = true;
                break;
            }
            prev = &mut (*curr).next;
            curr = (*curr).next;
        }

        if buddy_found {
            let merged_addr = core::cmp::min(addr, buddy_addr);
            self.free_block(merged_addr, order + 1);
        } else {
            let new_node = addr as *mut FreeBlock;
            (*new_node).next = self.buckets[order];
            self.buckets[order] = new_node;
        }
    }
}

static BUDDY: Mutex<BuddyAllocator> = Mutex::new(BuddyAllocator::new());
static TOTAL_PAGES: AtomicUsize = AtomicUsize::new(0);
static FREE_PAGES: AtomicUsize = AtomicUsize::new(0);

pub fn init() {
    let mut allocator = BUDDY.lock();
    unsafe {
        allocator.add_mem(0x10_0000, 0x2000_0000 - 0x10_0000);
    }
}

#[inline] pub fn alloc_pages(order: usize) -> Option<usize> { BUDDY.lock().alloc_pages(order) }
#[inline] pub fn free_pages(phys: usize, order: usize) { unsafe { BUDDY.lock().free_block(phys, order); } FREE_PAGES.fetch_add(1 << order, Ordering::Relaxed); }
#[inline] pub fn free_pages_count() -> usize { FREE_PAGES.load(Ordering::Relaxed) }

pub struct SlabCache {
    object_size: usize,
    current_page: usize,
    next_offset: usize,
    free_list: *mut FreeBlock,
}

impl SlabCache {
    pub const fn new(object_size: usize) -> Self {
        Self {
            object_size: if object_size < 8 { 8 } else { object_size },
            current_page: 0,
            next_offset: 0,
            free_list: core::ptr::null_mut(),
        }
    }

    pub fn alloc(&mut self) -> Option<*mut u8> {
        if !self.free_list.is_null() {
            let obj = self.free_list as *mut u8;
            unsafe { self.free_list = (*self.free_list).next; }
            return Some(obj);
        }

        if self.current_page == 0 || self.next_offset + self.object_size > PAGE_SIZE {
            self.current_page = alloc_pages(0)?;
            self.next_offset = 0;
        }

        let obj_addr = self.current_page + self.next_offset;
        self.next_offset += self.object_size;
        Some(obj_addr as *mut u8)
    }

    pub unsafe fn free(&mut self, ptr: *mut u8) {
        if ptr.is_null() { return; }
        let node = ptr as *mut FreeBlock;
        (*node).next = self.free_list;
        self.free_list = node;
    }
}
