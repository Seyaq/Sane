use super::{dma::DmaBuffer, reg};

pub const SQ_ENTRY_SIZE: usize = 64;
pub const CQ_ENTRY_SIZE: usize = 16;
pub const QUEUE_DEPTH:   usize = 64;

#[repr(C)]
pub struct SqEntry {
    pub cdw0: u32, pub nsid: u32, pub cdw2: u32, pub cdw3: u32,
    pub mptr: u64, pub dptr: [u64; 2],
    pub cdw10: u32, pub cdw11: u32, pub cdw12: u32,
    pub cdw13: u32, pub cdw14: u32, pub cdw15: u32,
}

#[repr(C)]
pub struct CqEntry {
    pub dw0: u32, pub dw1: u32,
    pub sqhd: u16, pub sqid: u16,
    pub cid: u16, pub sf: u16,
}

pub struct SubmissionQueue { pub buf: DmaBuffer, pub tail: u16, pub id: u16, pub db: usize }
pub struct CompletionQueue { pub buf: DmaBuffer, pub head: u16, pub phase: u8, pub id: u16, pub db: usize }

impl SubmissionQueue {
    pub fn new(id: u16, db: usize) -> Option<Self> {
        Some(Self { buf: DmaBuffer::alloc(1)?, tail: 0, id, db })
    }

    #[inline]
    pub fn submit(&mut self, cmd: SqEntry) {
        let slot = (self.buf.virt + self.tail as usize * SQ_ENTRY_SIZE) as *mut SqEntry;
        unsafe { slot.write_volatile(cmd) };
        self.tail = (self.tail + 1) % QUEUE_DEPTH as u16;
        unsafe { reg::write32(self.db, 0, self.tail as u32) };
    }
}

impl CompletionQueue {
    pub fn new(id: u16, db: usize) -> Option<Self> {
        Some(Self { buf: DmaBuffer::alloc(1)?, head: 0, phase: 1, id, db })
    }

    #[inline]
    pub fn poll(&mut self) -> Option<CqEntry> {
        let ptr = (self.buf.virt + self.head as usize * CQ_ENTRY_SIZE) as *const CqEntry;
        let e = unsafe { ptr.read_volatile() };
        if (e.sf & 1) != self.phase as u16 { return None; }
        self.head = (self.head + 1) % QUEUE_DEPTH as u16;
        if self.head == 0 { self.phase ^= 1; }
        unsafe { reg::write32(self.db, 0, self.head as u32) };
        Some(e)
    }
}
