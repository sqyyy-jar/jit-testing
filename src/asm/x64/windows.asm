.global asm_snapshot
.global asm_return_virtual_native
.global asm_call_virtual_native
.global asm_return_native_virtual
.global asm_print
.global asm_halt

asm_snapshot: // (rcx: *Runner) windows
    mov [rcx], rbx
    mov [rcx + 8], rsp
    mov [rcx + 16], rbp
    mov [rcx + 24], rsi
    mov [rcx + 32], rdi
    mov [rcx + 40], r12
    mov [rcx + 48], r13
    mov [rcx + 56], r14
    mov [rcx + 64], r15
    movups xmm0, [rsp]
    movups [rdi + 72], xmm0
    movups xmm0, [rsp + 16]
    movups [rdi + 88], xmm0
    ret

asm_return_virtual_native: // (rcx: *Runner, rdx: *Context) windows
    // Change ABI
    mov rdi, rcx
    mov rsi, rdx
    // Load mapped registers
    mov rsp, [rsi + 88] // callstack
    ret

asm_call_virtual_native: // (rcx: *Runner, rdx: *Context, r8: usize) windows
    // Change ABI
    mov rdi, rcx
    mov rsi, rdx
    // Load mapped registers
    mov rsp, [rsi + 88] // callstack
    jmp r8

asm_return_native_virtual: // (rdi: *Runner, rsi: *Context) custom
    pop rax
    mov [rsi + 64], rax // return address
    mov [rsi + 88], rsp // callstack
    // Restore snapshot
    mov rcx, rdi
    mov rbx, [rcx]
    mov rsp, [rcx + 8]
    mov rbp, [rcx + 16]
    mov rsi, [rcx + 24]
    mov rdi, [rcx + 32]
    mov r12, [rcx + 40]
    mov r13, [rcx + 48]
    mov r14, [rcx + 56]
    mov r15, [rcx + 64]
    movups xmm0, [rdi + 72]
    movups [rsp], xmm0
    movups xmm0, [rdi + 88]
    movups [rsp + 16], xmm0
    ret

asm_print: // (rdi: *Runner, rsi: *Context, rax: i64) custom
    // Save mapped registers
    mov rsp, [rdi + 8] // stack snapshot
    // Save state
    sub rsp, 8
    mov rcx, rax
    // Call
    call {print_num}
    // Restore State
    add rsp, 8
    // Restore mapped registers
    mov rsp, [rsi + 88] // callstack
    ret

asm_halt: // (rdi: *Runner, rsi: *Context) custom
    mov qword ptr [rdi + 112], 0 // running
    // Save mapped registers
    mov [rsi + 88], rsp // callstack
    // Restore snapshot
    mov rcx, rdi
    mov rbx, [rcx]
    mov rsp, [rcx + 8]
    mov rbp, [rcx + 16]
    mov rsi, [rcx + 24]
    mov rdi, [rcx + 32]
    mov r12, [rcx + 40]
    mov r13, [rcx + 48]
    mov r14, [rcx + 56]
    mov r15, [rcx + 64]
    movups xmm0, [rdi + 72]
    movups [rsp], xmm0
    movups xmm0, [rdi + 88]
    movups [rsp + 16], xmm0
    ret

