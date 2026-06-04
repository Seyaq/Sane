use spin::Mutex;

const MAX_CAPS: usize = 4096;

#[derive(Clone, Copy, Default, Debug)]
pub struct MemCap {
    pub base:   u64,
    pub size:   u64,
    pub rights: CapRights,
    pub valid:  bool,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Debug)]
    pub struct CapRights: u8 {
        const READ    = 1 << 0;
        const WRITE   = 1 << 1;
        const EXECUTE = 1 << 2;
        const GRANT   = 1 << 3;
    }
}

static CAP_TABLE: Mutex<[MemCap; MAX_CAPS]> = Mutex::new(
    [MemCap { base: 0, size: 0, rights: CapRights::empty(), valid: false }; MAX_CAPS]
);
static mut NEXT_SLOT: u32 = 1;

pub fn mint(base: u64, size: u64, rights: CapRights) -> Option<u32> {
    let slot = unsafe {
        if NEXT_SLOT as usize >= MAX_CAPS { return None; }
        let s = NEXT_SLOT; NEXT_SLOT += 1; s
    };
    CAP_TABLE.lock()[slot as usize] = MemCap { base, size, rights, valid: true };
    Some(slot)
}

pub fn derive(parent: u32, new_rights: CapRights) -> Option<u32> {
    let parent_cap = CAP_TABLE.lock()[parent as usize];
    if !parent_cap.valid { return None; }
    if !parent_cap.rights.contains(CapRights::GRANT) { return None; }
    mint(parent_cap.base, parent_cap.size, parent_cap.rights & new_rights)
}

pub fn revoke(handle: u32) {
    CAP_TABLE.lock()[handle as usize] = MemCap::default();
}

#[inline]
pub fn check(handle: u32, addr: u64, len: u64, req: CapRights) -> bool {
    let t = CAP_TABLE.lock();
    let cap = &t[handle as usize];
    cap.valid && addr >= cap.base && addr + len <= cap.base + cap.size && cap.rights.contains(req)
}
