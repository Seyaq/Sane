use super::pmm;

bitflags::bitflags! {
    #[derive(Clone, Copy)]
    pub struct PteFlags: u64 {
        const PRESENT   = 1 << 0;
        const WRITABLE  = 1 << 1;
        const USER      = 1 << 2;
        const HUGE      = 1 << 7;
        const GLOBAL    = 1 << 8;
        const FROZEN    = 1 << 9; 
        const NO_EXEC   = 1 << 63;
    }
}

const ENTRIES: usize = 512;
const HUGE_SIZE: usize = 2 * 1024 * 1024;

#[repr(C, align(4096))]
pub struct PageTable([u64; ENTRIES]);

static mut KERNEL_PML4: PageTable = PageTable([0u64; ENTRIES]);

pub fn init() {
    unsafe {
        map_huge_identity(&mut KERNEL_PML4, 0, 0x1_0000_0000);
        load_pml4(&KERNEL_PML4);
    }
}

#[inline]
pub unsafe fn load_pml4(pml4: &PageTable) {
    core::arch::asm!("mov cr3, {}", in(reg) pml4 as *const PageTable as u64, options(nostack, preserves_flags));
}

pub fn map_huge_identity(pml4: &mut PageTable, phys_start: usize, size: usize) {
    let flags = PteFlags::PRESENT | PteFlags::WRITABLE | PteFlags::HUGE | PteFlags::GLOBAL;
    let mut phys = phys_start & !(HUGE_SIZE - 1);
    let end = (phys_start + size + HUGE_SIZE - 1) & !(HUGE_SIZE - 1);
    
    while phys < end {
        set_mapping(pml4, phys, phys, flags, true);
        phys += HUGE_SIZE;
    }
}

pub fn set_mapping(pml4: &mut PageTable, virt: usize, phys: usize, flags: PteFlags, huge: bool) {
    let pml4_idx = (virt >> 39) & 0x1FF;
    let pdpt_idx = (virt >> 30) & 0x1FF;
    let pd_idx   = (virt >> 21) & 0x1FF;

    let pdpt = ensure_table(&mut pml4.0[pml4_idx], flags);
    let pd   = ensure_table(&mut pdpt.0[pdpt_idx], flags);

    if huge {
        pd.0[pd_idx] = ((phys & !(HUGE_SIZE - 1)) as u64) | flags.bits();
        flush(virt);
    } else {
        let pt     = ensure_table(&mut pd.0[pd_idx], flags);
        let pt_idx = (virt >> 12) & 0x1FF;
        pt.0[pt_idx] = ((phys & !0xFFF) as u64) | flags.bits();
        flush(virt);
    }
}

pub fn freeze_region(pml4: &mut PageTable, virt_start: usize, size: usize, huge: bool) {
    let mut virt = virt_start;
    let end = virt_start + size;
    let step = if huge { HUGE_SIZE } else { 4096 };

    while virt < end {
        let pml4_idx = (virt >> 39) & 0x1FF;
        let pdpt_idx = (virt >> 30) & 0x1FF;
        let pd_idx   = (virt >> 21) & 0x1FF;

        if pml4.0[pml4_idx] & PteFlags::PRESENT.bits() != 0 {
            let pdpt = unsafe { &mut *(((pml4.0[pml4_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable)) };
            if pdpt.0[pdpt_idx] & PteFlags::PRESENT.bits() != 0 {
                let pd = unsafe { &mut *(((pdpt.0[pdpt_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable)) };
                
                if huge {
                    if pd.0[pd_idx] & PteFlags::PRESENT.bits() != 0 {
                        pd.0[pd_idx] &= !PteFlags::WRITABLE.bits();
                        pd.0[pd_idx] |= PteFlags::FROZEN.bits();
                        flush(virt);
                    }
                } else if pd.0[pd_idx] & PteFlags::PRESENT.bits() != 0 {
                    let pt = unsafe { &mut *(((pd.0[pd_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable)) };
                    let pt_idx = (virt >> 12) & 0x1FF;
                    if pt.0[pt_idx] & PteFlags::PRESENT.bits() != 0 {
                        pt.0[pt_idx] &= !PteFlags::WRITABLE.bits();
                        pt.0[pt_idx] |= PteFlags::FROZEN.bits();
                        flush(virt);
                    }
                }
            }
        }
        virt += step;
    }
}

pub unsafe fn handle_page_fault(pml4: &mut PageTable, fault_address: usize, error_code: u64) -> bool {
    let write_bit = (error_code & (1 << 1)) != 0;
    if !write_bit {
        return false;
    }

    let pml4_idx = (fault_address >> 39) & 0x1FF;
    let pdpt_idx = (fault_address >> 30) & 0x1FF;
    let pd_idx   = (fault_address >> 21) & 0x1FF;

    if pml4.0[pml4_idx] & PteFlags::PRESENT.bits() == 0 { return false; }
    let pdpt = &mut *(((pml4.0[pml4_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable));
    
    if pdpt.0[pdpt_idx] & PteFlags::PRESENT.bits() == 0 { return false; }
    let pd = &mut *(((pdpt.0[pdpt_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable));

    if pd.0[pd_idx] & PteFlags::PRESENT.bits() == 0 { return false; }

    if pd.0[pd_idx] & PteFlags::HUGE.bits() != 0 {
        if pd.0[pd_idx] & PteFlags::FROZEN.bits() != 0 {
            let old_phys = (pd.0[pd_idx] & 0x000F_FFFF_FFE0_0000) as usize;
            let new_phys = pmm::alloc_pages(9).expect("VMM: COW OOM");
            
            core::ptr::copy_nonoverlapping(old_phys as *const u8, new_phys as *mut u8, HUGE_SIZE);
            
            let mut flags = PteFlags::from_bits_truncate(pd.0[pd_idx]);
            flags.insert(PteFlags::WRITABLE);
            flags.remove(PteFlags::FROZEN);
            
            pd.0[pd_idx] = (new_phys as u64) | flags.bits();
            flush(fault_address);
            return true;
        }
        return false;
    }

    let pt = &mut *(((pd.0[pd_idx] & 0x000F_FFFF_FFFF_F000) as *mut PageTable));
    let pt_idx = (fault_address >> 12) & 0x1FF;

    if pt.0[pt_idx] & PteFlags::PRESENT.bits() == 0 { return false; }

    if pt.0[pt_idx] & PteFlags::FROZEN.bits() != 0 {
        let old_phys = (pt.0[pt_idx] & 0x000F_FFFF_FFFF_F000) as usize;
        let new_phys = pmm::alloc_pages(0).expect("VMM: COW OOM");
        
        core::ptr::copy_nonoverlapping(old_phys as *const u8, new_phys as *mut u8, 4096);
        
        let mut flags = PteFlags::from_bits_truncate(pt.0[pt_idx]);
        flags.insert(PteFlags::WRITABLE);
        flags.remove(PteFlags::FROZEN);
        
        pt.0[pt_idx] = (new_phys as u64) | flags.bits();
        flush(fault_address);
        return true;
    }

    false
}

fn ensure_table(pte: &mut u64, child_flags: PteFlags) -> &mut PageTable {
    if *pte & PteFlags::PRESENT.bits() == 0 {
        let frame = pmm::alloc_pages(0).expect("VMM: OOM");
        unsafe { core::ptr::write_bytes(frame as *mut u8, 0, 4096) };
        
        let mut table_flags = PteFlags::PRESENT | PteFlags::WRITABLE;
        if child_flags.contains(PteFlags::USER) {
            table_flags |= PteFlags::USER;
        }
        
        *pte = (frame as u64) | table_flags.bits();
    }
    unsafe { &mut *((*pte & 0x000F_FFFF_FFFF_F000) as *mut PageTable) }
}

#[inline]
pub fn flush(virt: usize) {
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags)) };
}


