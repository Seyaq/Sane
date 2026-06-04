use crate::cpu::logical_core_count;

#[derive(Clone, Copy, Default)]
pub struct LogicalCpu {
    pub apic_id: u32,
    pub smt_id:  u8,
    pub core_id: u16,
    pub pkg_id:  u8,
}

static mut TOPO_MAP:   [LogicalCpu; 256] = [LogicalCpu { apic_id: 0, smt_id: 0, core_id: 0, pkg_id: 0 }; 256];
static mut CORE_COUNT: u8 = 0;

pub fn scan() {
    let n = logical_core_count();
    unsafe { CORE_COUNT = n };
    for i in 0..n as usize {
        unsafe { TOPO_MAP[i] = probe() };
    }
}

#[inline]
fn probe() -> LogicalCpu {
    let (eax0, ecx0, eax1, ecx1, edx1): (u32, u32, u32, u32, u32);
    unsafe {
        core::arch::asm!(
            "mov eax, 0xB", "xor ecx, ecx", "cpuid",
            out("eax") eax0, lateout("ebx") _, out("ecx") ecx0, lateout("edx") _
        );
        core::arch::asm!(
            "mov eax, 0xB", "mov ecx, 1", "cpuid",
            out("eax") eax1, lateout("ebx") _, out("ecx") ecx1, out("edx") edx1
        );
    }

    let smt_w  = if (ecx0 >> 8) & 0xFF == 1 { eax0 & 0x1F } else { 0 };
    let core_w = if (ecx1 >> 8) & 0xFF == 2 { eax1 & 0x1F } else { 0 };
    let apic   = edx1;

    let smt_mask  = (1u32 << smt_w).wrapping_sub(1);
    let core_mask = ((1u32 << core_w).wrapping_sub(1)) ^ smt_mask;
    let pkg_mask  = !((1u32 << core_w).wrapping_sub(1));

    LogicalCpu {
        apic_id: apic,
        smt_id:  (apic & smt_mask) as u8,
        core_id: ((apic & core_mask) >> smt_w) as u16,
        pkg_id:  ((apic & pkg_mask) >> core_w) as u8,
    }
}

#[inline]
pub fn topo() -> &'static [LogicalCpu] {
    unsafe { &TOPO_MAP[..CORE_COUNT as usize] }
}
