use super::{reg, queue::{SubmissionQueue, CompletionQueue, SqEntry}, namespace::Namespace};
use crate::usr::drivers::bus::pci;

const IO_READ:  u8 = 0x02;
const IO_WRITE: u8 = 0x01;

static mut CTRL_BASE: usize = 0;
static mut IO_SQ: Option<SubmissionQueue> = None;
static mut IO_CQ: Option<CompletionQueue> = None;

pub fn init() {
    let base = match pci::find_nvme_bar0() { Some(b) => b, None => return };
    unsafe { CTRL_BASE = base };

    let cap = unsafe { reg::read64(base, reg::CAP) };
    let to  = ((cap >> 24) & 0xFF) as u32;

    unsafe { reg::write32(base, reg::CC, 0) };
    wait_ready(base, false, to);

    let admin_sq = SubmissionQueue::new(0, doorbell(base, 0, false)).unwrap();
    let admin_cq = CompletionQueue::new(0, doorbell(base, 0, true)).unwrap();

    unsafe {
        reg::write32(base, reg::AQA, ((63) << 16) | 63);
        write64_pair(base, reg::ASQ, admin_sq.buf.phys as u64);
        write64_pair(base, reg::ACQ, admin_cq.buf.phys as u64);
    }

    let cc = reg::CC_EN | reg::CC_CSS_NVM | reg::CC_MPS_4K | reg::CC_IOSQES | reg::CC_IOCQES;
    unsafe { reg::write32(base, reg::CC, cc) };
    wait_ready(base, true, to);

    unsafe {
        IO_CQ = Some(CompletionQueue::new(1, doorbell(base, 1, true)).unwrap());
        IO_SQ = Some(SubmissionQueue::new(1, doorbell(base, 1, false)).unwrap());
    }
}

pub fn read_sectors(nsid: u32, lba: u64, count: u16, buf_phys: u64) -> bool {
    let cmd = SqEntry {
        cdw0:  (IO_READ as u32) | (1u32 << 16),
        nsid,  cdw2: 0, cdw3: 0, mptr: 0,
        dptr:  [buf_phys, 0],
        cdw10: lba as u32, cdw11: (lba >> 32) as u32,
        cdw12: (count as u32) - 1,
        cdw13: 0, cdw14: 0, cdw15: 0,
    };
    unsafe { IO_SQ.as_mut().map_or(false, |sq| { sq.submit(cmd); true }) }
}

fn doorbell(base: usize, qid: usize, completion: bool) -> usize {
    base + 0x1000 + (2 * qid + completion as usize) * 4
}

fn wait_ready(base: usize, want: bool, timeout_500ms: u32) {
    for _ in 0..timeout_500ms * 1000 {
        if (unsafe { reg::read32(base, reg::CSTS) } & reg::CSTS_RDY != 0) == want { return; }
        for _ in 0..10000 { unsafe { core::arch::asm!("pause") }; }
    }
}

unsafe fn write64_pair(base: usize, offset: usize, val: u64) {
    reg::write32(base, offset,     (val & 0xFFFF_FFFF) as u32);
    reg::write32(base, offset + 4, (val >> 32) as u32);
}
