use super::{CpuVendor, detect_vendor};

pub fn init() {
    let vendor = detect_vendor();
    gdt::load();
    idt::load();
    lapic::init(vendor);
    fpu::enable_extended();
    match vendor {
        CpuVendor::Amd   => { super::amd::cores::scan(); super::amd::cache::build_topology(); }
        CpuVendor::Intel => { super::intel::cores::scan(); super::intel::cache::build_topology(); }
        CpuVendor::Other => {}
    }
}

mod gdt {
    use core::arch::asm;

    #[repr(C, packed)]
    struct GdtDescriptor { limit: u16, base: u64 }

    static mut GDT: [u64; 8] = [
        0x0000_0000_0000_0000,
        0x00af_9a00_0000_ffff,
        0x00cf_9200_0000_ffff,
        0x00af_fa00_0000_ffff,
        0x00cf_f200_0000_ffff,
        0x0000_0000_0000_0000,
        0x0000_0000_0000_0000,
        0x0000_0000_0000_0000,
    ];

    pub fn load() {
        unsafe {
            let desc = GdtDescriptor {
                limit: (core::mem::size_of_val(&GDT) - 1) as u16,
                base:  GDT.as_ptr() as u64,
            };
            asm!(
                "lgdt [{0}]",
                "push 0x08",
                "lea rax, [rip + 1f]",
                "push rax",
                "retfq",
                "1:",
                "mov ax, 0x10",
                "mov ds, ax", "mov es, ax", "mov fs, ax", "mov gs, ax", "mov ss, ax",
                in(reg) &desc,
                lateout("rax") _,
            );
        }
    }
}

mod idt {
    pub fn load() {}
}

mod lapic {
    use super::super::CpuVendor;

    pub fn init(_vendor: CpuVendor) {
        unsafe {
            let (mut lo, hi): (u32, u32);
            core::arch::asm!("rdmsr", in("ecx") 0x1Bu32, out("eax") lo, out("edx") hi);
            lo |= 1 << 11;
            core::arch::asm!("wrmsr", in("ecx") 0x1Bu32, in("eax") lo, in("edx") hi);
            (0xFEE0_00F0u64 as *mut u32).write_volatile(0x1FF);
        }
    }
}

mod fpu {
    pub fn enable_extended() {
        unsafe {
            let mut cr4: u64;
            core::arch::asm!("mov {}, cr4", out(reg) cr4);
            cr4 |= (1 << 9) | (1 << 10) | (1 << 18);
            core::arch::asm!("mov cr4, {}", in(reg) cr4);
            core::arch::asm!("xsetbv", in("ecx") 0u32, in("eax") 7u32, in("edx") 0u32);
        }
    }
}
