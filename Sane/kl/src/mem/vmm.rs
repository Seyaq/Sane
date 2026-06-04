use super::pmm;

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct PteFlags: u64 {
        const PRESENT  = 1 << 0;
        const WRITABLE = 1 << 1;
        const USER     = 1 << 2;
        const HUGE     = 1 << 7;
        const GLOBAL   = 1 << 8;
        const NO_EXEC  = 1 << 63;
    }
}

const ENTRIES: usize = 512;

#[repr(C, align(4096))]
struct PageTable([u64; ENTRIES]);

static mut KERNEL_PML4: PageTable = PageTable([0u64; ENTRIES]);

pub fn init() {
    unsafe {
        map_huge_identity(&mut KERNEL_PML4, 0, 0x1_0000_0000);
        load_pml4(&KERNEL_PML4);
    }
}

#[inline]
unsafe fn load_pml4(pml4: &PageTable) {
    core::arch::asm!("mov cr3, {}", in(reg) pml4 as *const PageTable as u64);
}

pub fn map_huge_identity(pml4: &mut PageTable, phys_start: usize, size: usize) {
    let flags = PteFlags::PRESENT | PteFlags::WRITABLE | PteFlags::HUGE | PteFlags::GLOBAL;
    let mut phys = phys_start;
    while phys < phys_start + size {
        set_mapping(pml4, phys, phys, flags, true);
        phys += HUGE_SIZE;
    }
}

fn set_mapping(pml4: &mut PageTable, virt: usize, phys: usize, flags: PteFlags, huge: bool) {
    let pml4_idx = (virt >> 39) & 0x1FF;
    let pdpt_idx = (virt >> 30) & 0x1FF;
    let pd_idx   = (virt >> 21) & 0x1FF;
    let pdpt = ensure_table(&mut pml4.0[pml4_idx]);
    let pd   = ensure_table(&mut pdpt.0[pdpt_idx]);
    if huge {
        pd.0[pd_idx] = (phys as u64) | flags.bits();
    } else {
        let pt    = ensure_table(&mut pd.0[pd_idx]);
        let pt_idx = (virt >> 12) & 0x1FF;
        pt.0[pt_idx] = (phys as u64) | flags.bits();
    }
}

const HUGE_SIZE: usize = 2 * 1024 * 1024;

fn ensure_table(pte: &mut u64) -> &mut PageTable {
    if *pte & PteFlags::PRESENT.bits() == 0 {
        let frame = pmm::alloc_frame().expect("VMM: OOM");
        unsafe { core::ptr::write_bytes(frame as *mut u8, 0, 4096) };
        *pte = (frame as u64) | (PteFlags::PRESENT | PteFlags::WRITABLE).bits();
    }
    unsafe { &mut *((*pte & !0xFFF) as *mut PageTable) }
}

#[inline]
pub fn flush(virt: usize) {
    unsafe { core::arch::asm!("invlpg [{0}]", in(reg) virt) };
}
