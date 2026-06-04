use super::cspace::{lookup, CapKind};

#[derive(Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct MsgRegs {
    pub label: u32,
    pub w0:    u64,
    pub w1:    u64,
    pub w2:    u64,
}

#[derive(Clone, Copy, Debug)]
pub enum IpcResult {
    Ok(MsgRegs),
    Error(IpcError),
}

#[derive(Clone, Copy, Debug)]
pub enum IpcError {
    InvalidCap,
    WrongCapKind,
    ReceiverNotReady,
    MessageTooLong,
}

#[inline]
pub fn call(ep_cptr: u32, msg: MsgRegs) -> IpcResult {
    let cap = match lookup(ep_cptr) {
        Some(c) if c.kind == CapKind::Endpoint => c,
        Some(_) => return IpcResult::Error(IpcError::WrongCapKind),
        None    => return IpcResult::Error(IpcError::InvalidCap),
    };
    
    if cap.rights & 0x02 == 0 { 
        return IpcResult::Error(IpcError::InvalidCap); 
    }
    
    deliver(cap.object, msg)
}

#[inline]
pub fn send(ep_cptr: u32, msg: MsgRegs) -> bool {
    match lookup(ep_cptr) {
        Some(c) if c.kind == CapKind::Endpoint && c.rights & 0x02 != 0 => {
            let _ = deliver(c.object, msg); 
            true
        }
        _ => false,
    }
}

#[inline(always)]
fn deliver(endpoint_obj: u64, msg: MsgRegs) -> IpcResult {
    if endpoint_obj == 0 {
        return IpcResult::Error(IpcError::ReceiverNotReady);
    }

    let mut out_label: u32;
    let mut out_w0: u64;
    let mut out_w1: u64;
    let mut out_w2: u64;

    unsafe {
        core::arch::asm!(
            "mov r11, rcx",
            "syscall",
            "mov rcx, r11",
            in("rdi") endpoint_obj,
            in("rsi") msg.label,
            in("rdx") msg.w0,
            in("r10") msg.w1,
            in("r8")  msg.w2,
            lateout("rax") out_label,
            lateout("rdi") out_w0,
            lateout("rsi") out_w1,
            lateout("rdx") out_w2,
            clobber_mem,
            options(nostack)
        );
    }

    if out_label == u32::MAX {
        return IpcResult::Error(IpcError::ReceiverNotReady);
    }

    IpcResult::Ok(MsgRegs {
        label: out_label,
        w0: out_w0,
        w1: out_w1,
        w2: out_w2,
    })
}
