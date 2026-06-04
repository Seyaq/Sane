const MAX_THREADS: usize = 256;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Priority {
    Idle        = 0,
    Background  = 64,
    Normal      = 100,
    Interactive = 130,
    RealTime    = 200,
}

#[derive(Clone, Copy, Debug)]
pub struct Thread {
    pub tid:           u32,
    pub base_prio:     i32,
    pub dynamic_prio:  i32,
    pub burst_ns:      u64,
    pub latency_class: Priority,
    pub stack_ptr:     u64,
    pub cr3:           u64,
    pub name:          [u8; 16],
}

impl Thread {
    pub fn new_idle() -> Self {
        Self {
            tid: 0,
            base_prio: Priority::Idle as i32,
            dynamic_prio: Priority::Idle as i32,
            burst_ns: 0,
            latency_class: Priority::Idle,
            stack_ptr: 0,
            cr3: 0,
            name: *b"idle\0\0\0\0\0\0\0\0\0\0\0\0",
        }
    }
}

pub struct RunQueue {
    threads: [Option<Thread>; MAX_THREADS],
    count:   usize,
}

impl RunQueue {
    pub const fn new() -> Self {
        Self { threads: [None; MAX_THREADS], count: 0 }
    }

    pub fn push(&mut self, t: Thread) {
        if self.count >= MAX_THREADS { return; }
        self.threads[self.count] = Some(t);
        self.count += 1;
        let mut i = self.count - 1;
        while i > 0 {
            let a = self.threads[i - 1].map_or(0, |t| t.dynamic_prio);
            let b = self.threads[i    ].map_or(0, |t| t.dynamic_prio);
            if b > a { self.threads.swap(i - 1, i); i -= 1; } else { break; }
        }
    }

    pub fn pop_best(&mut self) -> Option<Thread> {
        if self.count == 0 { return None; }
        let t = self.threads[0];
        self.threads.copy_within(1..self.count, 0);
        self.threads[self.count - 1] = None;
        self.count -= 1;
        t
    }

    pub fn peek_best(&self) -> Option<&Thread> { self.threads[0].as_ref() }

    pub fn set_latency_class(&mut self, tid: u32, prio: Priority) {
        for slot in self.threads[..self.count].iter_mut().flatten() {
            if slot.tid == tid {
                slot.latency_class = prio;
                slot.base_prio     = prio as i32;
                slot.dynamic_prio  = prio as i32;
            }
        }
    }
}
