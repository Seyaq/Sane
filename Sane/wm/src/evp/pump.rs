use super::event::Event;
use crate::wmc::compositor::Compositor;
use arrayvec::ArrayVec;

pub struct EventPump {
    queue: ArrayVec<Event, 256>,
}

impl EventPump {
    pub fn new() -> Self { Self { queue: ArrayVec::new() } }

    pub fn push(&mut self, e: Event) { let _ = self.queue.try_push(e); }

    pub fn drain(&mut self, comp: &mut Compositor) {
        for e in self.queue.drain(..) { comp.dispatch(e); }
    }
}
