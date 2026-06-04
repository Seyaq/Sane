pub mod boot;
pub mod amd;
pub mod intel;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CpuVendor { Amd, Intel, Other }

#[inline]
pub fn detect_vendor() -> CpuVendor {
    let (ebx, ecx, edx): (u32, u32, u32);
    unsafe {
        core::arch::asm!(
            "xor eax, eax", "cpuid",
            out("ebx") ebx, out("ecx") ecx, out("edx") edx,
            lateout("eax") _,
        );
    }
    match (ebx, edx, ecx) {
        (0x6874_7541, 0x6974_6E65, 0x444D_4163) => CpuVendor::Amd,
        (0x756E_6547, 0x4965_6E69, 0x6C65_746E) => CpuVendor::Intel,
        _ => CpuVendor::Other,
    }
}

#[inline]
pub fn logical_core_count() -> u8 {
    let ebx: u32;
    unsafe { core::arch::asm!("mov eax, 1", "cpuid", out("ebx") ebx, lateout("eax") _, lateout("ecx") _, lateout("edx") _) };
    ((ebx >> 16) & 0xFF) as u8
}

#[inline(always)]
pub fn rdtsc() -> u64 {
    let lo: u32; let hi: u32;
    unsafe { core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi) };
    ((hi as u64) << 32) | lo as u64
}
