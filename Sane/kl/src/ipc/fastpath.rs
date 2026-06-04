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
pub enum IpcResult { Ok(MsgRegs), Error(IpcError) }

#[derive(Clone, Copy, Debug)]
pub enum IpcError { InvalidCap, WrongCapKind, ReceiverNotReady, MessageTooLong }

#[inline]
pub fn call(ep_cptr: u32, msg: MsgRegs) -> IpcResult {
    let cap = match lookup(ep_cptr) {
        Some(c) if c.kind == CapKind::Endpoint => c,
        Some(_) => return IpcResult::Error(IpcError::WrongCapKind),
        None    => return IpcResult::Error(IpcError::InvalidCap),
    };
    if cap.rights & 0x02 == 0 { return IpcResult::Error(IpcError::InvalidCap); }
    deliver(cap.object, msg)
}

#[inline]
pub fn send(ep_cptr: u32, msg: MsgRegs) -> bool {
    match lookup(ep_cptr) {
        Some(c) if c.kind == CapKind::Endpoint && c.rights & 0x02 != 0 => {
            deliver(c.object, msg); true
        }
        _ => false,
    }
}

#[inline(always)]
fn deliver(_endpoint_obj: u64, _msg: MsgRegs) -> IpcResult {
    IpcResult::Ok(MsgRegs::default())
}
