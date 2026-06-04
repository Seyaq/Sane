use crate::mem::pmm;

pub struct DmaBuffer {
    pub phys: usize,
    pub virt: usize,
    pub size: usize,
}

impl DmaBuffer {
    pub fn alloc(pages: usize) -> Option<Self> {
        match pages {
            1 => {
                let phys = pmm::alloc_frame()?;
                unsafe { core::ptr::write_bytes(phys as *mut u8, 0, 4096) };
                Some(Self { phys, virt: phys, size: 4096 })
            }
            512 => {
                let phys = pmm::alloc_huge()?;
                unsafe { core::ptr::write_bytes(phys as *mut u8, 0, 2 * 1024 * 1024) };
                Some(Self { phys, virt: phys, size: 2 * 1024 * 1024 })
            }
            _ => {
                let phys = pmm::alloc_frame()?;
                Some(Self { phys, virt: phys, size: pages * 4096 })
            }
        }
    }

    pub fn free(self) { pmm::free_frame(self.phys); }
}
