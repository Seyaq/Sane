use super::order::Thread;

#[naked]
pub unsafe extern "C" fn context_switch(from: *mut Thread, to: *const Thread) {
    core::arch::naked_asm!(
        "push rbx", "push rbp", "push r12", "push r13", "push r14", "push r15",
        "mov [rdi + 32], rsp",
        "mov rax, [rsi + 40]",
        "mov rcx, cr3",
        "cmp rax, rcx",
        "je 1f",
        "mov cr3, rax",
        "1:",
        "mov rsp, [rsi + 32]",
        "pop r15", "pop r14", "pop r13", "pop r12", "pop rbp", "pop rbx",
        "ret",
    )
}
