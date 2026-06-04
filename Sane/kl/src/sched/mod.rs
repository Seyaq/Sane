pub mod bore;
pub mod order;
pub mod arch;

pub fn idle() -> ! {
    loop { unsafe { core::arch::asm!("hlt") }; }
}
