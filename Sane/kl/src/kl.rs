#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![allow(dead_code)]

extern crate alloc;

use core::panic::PanicInfo;

mod cpu;
mod mem;
mod ipc;
mod sched;
mod usr;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    cpu::boot::init();
    mem::pmm::init();
    mem::vmm::init();
    ipc::cspace::init();
    sched::bore::init();
    usr::boot();
    sched::idle()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { unsafe { core::arch::asm!("hlt") }; }
}

#[alloc_error_handler]
fn alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("OOM: {:?}", layout);
}
