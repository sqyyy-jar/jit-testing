.global asm_snapshot
.global asm_return_virtual_native
.global asm_call_virtual_native
.global asm_return_native_virtual
//.global asm_launch_runner
//.global asm_return_runner
//.global asm_enter_native

asm_snapshot: // (*Runner)
    mov [rdi], rbx
    mov [rdi + 8], rsp
    mov [rdi + 16], rbp
    mov [rdi + 24], r12
    mov [rdi + 32], r13
    mov [rdi + 40], r14
    mov [rdi + 48], r15
    movups xmm0, [rsp]
    movups [rdi + 56], xmm0
    movups xmm0, [rsp + 16]
    movups [rdi + 72], xmm0
    ret

asm_return_virtual_native: // (rdi: *Runner, rsi: *Context)
    // Load mapped registers
    mov rsp, [rsi + 88] // callstack
    ret

asm_call_virtual_native: // (rdi: *Runner, rsi: *Context, rdx: usize)
    // Load mapped registers
    mov rsp, [rsi + 88] // callstack
    jmp rdx

asm_return_native_virtual: // (rdi: *Runner, rsi: *Context)
    // Save mapped registers
    pop rax
    mov [rsi + 64], rax // return address
    mov [rsi + 88], rsp // callstack
    // Restore snapshot
    mov rbx, [rdi]
    mov rsp, [rdi + 8]
    mov rbp, [rdi + 16]
    mov r12, [rdi + 24]
    mov r13, [rdi + 32]
    mov r14, [rdi + 40]
    mov r15, [rdi + 48]
    movups xmm0, [rdi + 56]
    movups [rsp], xmm0
    movups xmm0, [rdi + 72]
    movups [rsp + 16], xmm0
    ret

// Deprecated
asm_launch_runner:
    mov [rdi], rbx
    mov [rdi + 8], rsp
    mov [rdi + 16], rbp
    mov [rdi + 24], r12
    mov [rdi + 32], r13
    mov [rdi + 40], r14
    mov [rdi + 48], r15
    jmp {run}

asm_return_runner:
    mov rbx, [rax]
    mov rsp, [rax + 8]
    mov rbp, [rax + 16]
    mov r12, [rax + 24]
    mov r13, [rax + 32]
    mov r14, [rax + 40]
    mov r15, [rax + 48]
    ret

asm_enter_native: // (rdi: *mut Runner, rsi: *mut Context, rdx: *const ())
    mov [rdi], rbx
    mov [rdi + 8], rsp
    mov [rdi + 16], rbp
    mov [rdi + 24], r12
    mov [rdi + 32], r13
    mov [rdi + 40], r14
    mov [rdi + 48], r15
    // load mapped registers
    mov rsp, [rsi + 88]
    jmp rdx

