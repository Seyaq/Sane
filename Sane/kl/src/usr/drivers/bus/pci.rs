const ECAM_BASE: usize = 0xE000_0000;

#[inline]
fn cfg_addr(bus: u8, dev: u8, fun: u8, offset: u16) -> usize {
    ECAM_BASE | ((bus as usize) << 20) | ((dev as usize) << 15) | ((fun as usize) << 12) | (offset as usize & 0xFFC)
}

#[inline] pub unsafe fn cfg_read32(bus: u8, dev: u8, fun: u8, off: u16) -> u32  { (cfg_addr(bus, dev, fun, off) as *const u32).read_volatile() }
#[inline] pub unsafe fn cfg_write32(bus: u8, dev: u8, fun: u8, off: u16, v: u32) { (cfg_addr(bus, dev, fun, off) as *mut u32).write_volatile(v) }

#[derive(Clone, Copy, Debug)]
pub struct PciDevice {
    pub bus: u8, pub dev: u8, pub fun: u8,
    pub vendor: u16, pub device: u16,
    pub class: u8, pub subclass: u8, pub prog_if: u8,
    pub bar0: u64,
}

static mut DEVICES:   [Option<PciDevice>; 256] = [None; 256];
static mut DEV_COUNT: usize = 0;

pub fn start() { scan_all(); }

fn scan_all() {
    for bus in 0u8..=255 {
        for dev in 0u8..32 {
            for fun in 0u8..8 {
                unsafe { probe(bus, dev, fun) };
            }
        }
    }
}

unsafe fn probe(bus: u8, dev: u8, fun: u8) {
    let vid_did = cfg_read32(bus, dev, fun, 0x00);
    if vid_did == 0xFFFF_FFFF { return; }
    let class_rev = cfg_read32(bus, dev, fun, 0x08);
    let bar0_lo   = cfg_read32(bus, dev, fun, 0x10) as u64;
    let bar0 = if bar0_lo & 0x4 != 0 {
        (cfg_read32(bus, dev, fun, 0x14) as u64) << 32 | (bar0_lo & !0xF)
    } else { bar0_lo & !0xF };
    if DEV_COUNT < 256 {
        DEVICES[DEV_COUNT] = Some(PciDevice {
            bus, dev, fun,
            vendor:  (vid_did & 0xFFFF) as u16,
            device:  (vid_did >> 16) as u16,
            class:    (class_rev >> 24) as u8,
            subclass: (class_rev >> 16) as u8,
            prog_if:  (class_rev >>  8) as u8,
            bar0,
        });
        DEV_COUNT += 1;
    }
}

pub fn find_nvme_bar0() -> Option<usize> {
    unsafe {
        DEVICES[..DEV_COUNT].iter().flatten()
            .find(|d| d.class == 0x01 && d.subclass == 0x08 && d.prog_if == 0x02)
            .map(|d| d.bar0 as usize)
    }
}

pub fn devices() -> &'static [Option<PciDevice>] { unsafe { &DEVICES[..DEV_COUNT] } }
