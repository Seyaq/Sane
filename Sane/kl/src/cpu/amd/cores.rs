use crate::cpu::logical_core_count;

#[derive(Clone, Copy, Default)]
pub struct LogicalCpu {
    pub apic_id: u32,
    pub core_id: u8,
    pub ccd_id:  u8,
    pub ccx_id:  u8,
    pub smt_id:  u8,
}

static mut TOPO_MAP:   [LogicalCpu; 256] = [LogicalCpu { apic_id: 0, core_id: 0, ccd_id: 0, ccx_id: 0, smt_id: 0 }; 256];
static mut CORE_COUNT: u8 = 0;

pub fn scan() {
    let n = logical_core_count();
    unsafe { CORE_COUNT = n };
    for i in 0..n as usize {
        unsafe { TOPO_MAP[i] = probe(i as u32) };
    }
}

#[inline]
fn probe(apic_id: u32) -> LogicalCpu {
    let (ebx_1e, ecx_1e): (u32, u32);
    unsafe {
        core::arch::asm!(
            "mov eax, 0x8000001E", "xor ecx, ecx", "cpuid",
            out("ebx") ebx_1e, out("ecx") ecx_1e,
            lateout("eax") _, lateout("edx") _,
        );
    }
    LogicalCpu {
        apic_id,
        core_id: (ebx_1e & 0xFF) as u8,
        ccx_id:  ((ecx_1e >> 8) & 0x7) as u8,
        ccd_id:  (ecx_1e & 0xFF) as u8,
        smt_id:  ((ebx_1e >> 8) & 0xFF) as u8,
    }
}

#[inline]
pub fn topo() -> &'static [LogicalCpu] {
    unsafe { &TOPO_MAP[..CORE_COUNT as usize] }
}
