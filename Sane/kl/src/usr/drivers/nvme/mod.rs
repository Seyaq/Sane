pub mod dma;
pub mod main;
pub mod namespace;
pub mod queue;
pub mod reg;

pub fn start() { main::init(); }
