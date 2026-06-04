#[inline] pub unsafe fn inb(port: u16) -> u8  { let v: u8;  core::arch::asm!("in al, dx",  out("al")  v, in("dx") port); v }
#[inline] pub unsafe fn inw(port: u16) -> u16 { let v: u16; core::arch::asm!("in ax, dx",  out("ax")  v, in("dx") port); v }
#[inline] pub unsafe fn inl(port: u16) -> u32 { let v: u32; core::arch::asm!("in eax, dx", out("eax") v, in("dx") port); v }
#[inline] pub unsafe fn outb(port: u16, v: u8)  { core::arch::asm!("out dx, al",  in("dx") port, in("al")  v) }
#[inline] pub unsafe fn outw(port: u16, v: u16) { core::arch::asm!("out dx, ax",  in("dx") port, in("ax")  v) }
#[inline] pub unsafe fn outl(port: u16, v: u32) { core::arch::asm!("out dx, eax", in("dx") port, in("eax") v) }

pub mod serial {
    use super::*;
    const COM1: u16 = 0x3F8;

    pub fn init() {
        unsafe {
            outb(COM1 + 1, 0x00);
            outb(COM1 + 3, 0x80);
            outb(COM1 + 0, 0x01);
            outb(COM1 + 1, 0x00);
            outb(COM1 + 3, 0x03);
            outb(COM1 + 2, 0xC7);
            outb(COM1 + 4, 0x0B);
        }
    }

    #[inline]
    pub fn write_byte(b: u8) {
        unsafe { while inb(COM1 + 5) & 0x20 == 0 {} outb(COM1, b); }
    }

    pub fn write_str(s: &str) { for b in s.bytes() { write_byte(b); } }
}
