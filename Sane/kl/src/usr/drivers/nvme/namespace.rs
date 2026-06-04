#[derive(Debug, Clone, Copy)]
pub struct Namespace {
    pub nsid:      u32,
    pub lba_size:  u32,
    pub lba_count: u64,
    pub size_mb:   u64,
}

impl Namespace {
    pub fn from_identify(nsid: u32, data: &[u8; 4096]) -> Self {
        let nsze      = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let lba_idx   = (data[26] & 0xF) as usize;
        let lba_fmt   = u32::from_le_bytes(data[128 + lba_idx * 4..132 + lba_idx * 4].try_into().unwrap());
        let lba_size  = 1u32 << ((lba_fmt >> 16) & 0xFF);
        Namespace { nsid, lba_size, lba_count: nsze, size_mb: (nsze * lba_size as u64) / (1024 * 1024) }
    }
}
