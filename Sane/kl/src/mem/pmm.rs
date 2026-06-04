use spin::Mutex;

pub const PAGE_SIZE:  usize = 4096;
pub const HUGE_SIZE:  usize = 2 * 1024 * 1024;
const MAX_FRAMES: usize = 1 << 20;

static BITMAP:         Mutex<[u64; MAX_FRAMES / 64]> = Mutex::new([0u64; MAX_FRAMES / 64]);
static mut TOTAL_FRAMES: usize = 0;
static mut FREE_FRAMES:  usize = 0;

pub fn init() {
    let start_frame = 0x10_0000 / PAGE_SIZE;
    let frames      = (0x2000_0000 - 0x10_0000) / PAGE_SIZE;
    let mut bm      = BITMAP.lock();
    for i in start_frame..start_frame + frames {
        bm[i / 64] |= 1u64 << (i % 64);
    }
    unsafe { TOTAL_FRAMES = frames; FREE_FRAMES = frames; }
}

#[inline]
pub fn alloc_frame() -> Option<usize> {
    let mut bm = BITMAP.lock();
    for (wi, word) in bm.iter_mut().enumerate() {
        if *word != 0 {
            let bit = word.trailing_zeros() as usize;
            *word &= !(1u64 << bit);
            unsafe { FREE_FRAMES -= 1 };
            return Some((wi * 64 + bit) * PAGE_SIZE);
        }
    }
    None
}

pub fn alloc_huge() -> Option<usize> {
    let mut bm = BITMAP.lock();
    const WPH: usize = HUGE_SIZE / PAGE_SIZE / 64;
    let mut i = 0;
    while i + WPH <= bm.len() {
        if bm[i..i + WPH].iter().all(|&w| w == u64::MAX) {
            bm[i..i + WPH].fill(0);
            unsafe { FREE_FRAMES -= 512 };
            return Some(i * 64 * PAGE_SIZE);
        }
        i += WPH;
    }
    None
}

#[inline]
pub fn free_frame(phys: usize) {
    let frame = phys / PAGE_SIZE;
    BITMAP.lock()[frame / 64] |= 1u64 << (frame % 64);
    unsafe { FREE_FRAMES += 1 };
}

#[inline] pub fn free_frames()  -> usize { unsafe { FREE_FRAMES  } }
#[inline] pub fn total_frames() -> usize { unsafe { TOTAL_FRAMES } }
