pub const CAPLENGTH:  usize = 0x00;
pub const HCSPARAMS1: usize = 0x04;
pub const HCSPARAMS2: usize = 0x08;
pub const HCCPARAMS1: usize = 0x10;
pub const DBOFF:      usize = 0x14;
pub const RTSOFF:     usize = 0x18;
pub const USBCMD:     usize = 0x00;
pub const USBSTS:     usize = 0x04;
pub const PAGESIZE:   usize = 0x08;
pub const DNCTRL:     usize = 0x14;
pub const CRCR:       usize = 0x18;
pub const DCBAAP:     usize = 0x30;
pub const CONFIG:     usize = 0x38;
pub const USBCMD_RUN:   u32 = 1 << 0;
pub const USBCMD_HCRST: u32 = 1 << 1;
pub const USBSTS_HCH:   u32 = 1 << 0;

pub struct XhciController {
    pub cap_base:  usize,
    pub op_base:   usize,
    pub max_slots: u8,
    pub max_ports: u8,
}

impl XhciController {
    pub fn new(bar0: usize) -> Self {
        let cap_len = unsafe { (bar0 as *const u8).read_volatile() } as usize;
        let params1 = unsafe { ((bar0 + HCSPARAMS1) as *const u32).read_volatile() };
        Self {
            cap_base:  bar0,
            op_base:   bar0 + cap_len,
            max_slots: (params1 & 0xFF) as u8,
            max_ports: ((params1 >> 24) & 0xFF) as u8,
        }
    }

    pub fn reset(&self) {
        unsafe {
            let cmd = (self.op_base + USBCMD) as *mut u32;
            cmd.write_volatile(cmd.read_volatile() & !USBCMD_RUN);
            while (self.op_base + USBSTS) as *const u32 as u32 & USBSTS_HCH == 0 {}
            cmd.write_volatile(USBCMD_HCRST);
            while cmd.read_volatile() & USBCMD_HCRST != 0 {}
        }
    }
}
