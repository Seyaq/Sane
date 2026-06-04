pub const CAP:   usize = 0x00;
pub const VS:    usize = 0x08;
pub const INTMS: usize = 0x0C;
pub const INTMC: usize = 0x10;
pub const CC:    usize = 0x14;
pub const CSTS:  usize = 0x1C;
pub const NSSR:  usize = 0x20;
pub const AQA:   usize = 0x24;
pub const ASQ:   usize = 0x28;
pub const ACQ:   usize = 0x30;

pub const CC_EN:      u32 = 1 << 0;
pub const CC_CSS_NVM: u32 = 0 << 4;
pub const CC_MPS_4K:  u32 = 0 << 7;
pub const CC_AMS_RR:  u32 = 0 << 11;
pub const CC_IOSQES:  u32 = 6 << 16;
pub const CC_IOCQES:  u32 = 4 << 20;
pub const CSTS_RDY:   u32 = 1 << 0;
pub const CSTS_CFS:   u32 = 1 << 1;

#[inline] pub unsafe fn read32 (base: usize, off: usize) -> u32  { ((base + off) as *const u32).read_volatile() }
#[inline] pub unsafe fn write32(base: usize, off: usize, v: u32) { ((base + off) as *mut   u32).write_volatile(v) }
#[inline] pub unsafe fn read64 (base: usize, off: usize) -> u64  { ((base + off) as *const u64).read_volatile() }
