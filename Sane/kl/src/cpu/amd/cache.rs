#[derive(Clone, Copy, Debug)]
pub struct CacheLevel {
    pub level:        u8,
    pub size_kb:      u32,
    pub associativity: u8,
    pub shared_cpus:  u8,
}

static mut CACHE_TABLE: [CacheLevel; 16] = [CacheLevel { level: 0, size_kb: 0, associativity: 0, shared_cpus: 0 }; 16];
static mut CACHE_COUNT: usize = 0;

pub fn build_topology() {
    let mut idx = 0usize;
    for sub in 0u32..16 {
        let (eax, ebx, ecx): (u32, u32, u32);
        unsafe {
            core::arch::asm!(
                "mov eax, 0x8000001D", "cpuid",
                in("ecx") sub,
                out("eax") eax, out("ebx") ebx, out("ecx") ecx,
                lateout("edx") _,
            );
        }
        if eax & 0x1F == 0 { break; }
        let ways       = ((ebx >> 22) & 0x3FF) + 1;
        let partitions = ((ebx >> 12) & 0x3FF) + 1;
        let line_size  = (ebx & 0xFFF) + 1;
        let sets       = ecx + 1;
        unsafe {
            CACHE_TABLE[idx] = CacheLevel {
                level:        ((eax >> 5) & 0x7) as u8,
                size_kb:      (ways * partitions * line_size * sets) / 1024,
                associativity: ways as u8,
                shared_cpus:  ((eax >> 14) & 0xFFF) as u8 + 1,
            };
        }
        idx += 1;
    }
    unsafe { CACHE_COUNT = idx };
}

#[inline]
pub fn caches() -> &'static [CacheLevel] {
    unsafe { &CACHE_TABLE[..CACHE_COUNT] }
}
