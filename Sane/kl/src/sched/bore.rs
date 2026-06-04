use super::order::{RunQueue, Thread, Priority};
use spin::Mutex;

const DECAY_SHIFT:      u32 = 3;
const BURST_PENALTY_NS: u64 = 2_000_000;

static RUN_QUEUE: Mutex<RunQueue> = Mutex::new(RunQueue::new());

pub fn init() {
    RUN_QUEUE.lock().push(Thread::new_idle());
}

pub fn tick(t: &mut Thread) {
    t.burst_ns = t.burst_ns.saturating_add(1_000_000) >> DECAY_SHIFT;
    let penalty = if t.burst_ns > BURST_PENALTY_NS {
        ((t.burst_ns - BURST_PENALTY_NS) / 1_000_000) as i8
    } else { 0 };
    t.dynamic_prio = t.base_prio.saturating_add(penalty as i32);

    let mut rq = RUN_QUEUE.lock();
    if rq.peek_best().map_or(false, |next| next.dynamic_prio < t.dynamic_prio) {
        rq.push(*t);
    }
}

pub fn wake(mut t: Thread) {
    t.burst_ns    = 0;
    t.dynamic_prio = t.base_prio;
    RUN_QUEUE.lock().push(t);
}

pub fn schedule() -> Option<Thread> { RUN_QUEUE.lock().pop_best() }

pub fn set_gaming_class(tid: u32) {
    RUN_QUEUE.lock().set_latency_class(tid, Priority::RealTime);
}
