.global asm_snapshot
.global asm_launch_runner
.global asm_return_runner

asm_snapshot:
    mov [rdi], r12
    mov [rdi + 8], r13
    mov [rdi + 16], r14
    mov [rdi + 24], r15
    mov [rdi + 32], rdi
    mov [rdi + 40], rsi
    mov [rdi + 48], rbx
    mov [rdi + 56], rbp
    mov [rdi + 64], rsp
    movups xmm0, [rsp]
    movups [rdi + 72], xmm0
    movups xmm0, [rsp + 16]
    movups [rdi + 88], xmm0
    ret

asm_launch_runner:
    mov [rcx], r12
    mov [rcx + 8], r13
    mov [rcx + 16], r14
    mov [rcx + 24], r15
    mov [rcx + 32], rdi
    mov [rcx + 40], rsi
    mov [rcx + 48], rbx
    mov [rcx + 56], rbp
    mov [rcx + 64], rsp
    jmp {run}

asm_return_runner:
    mov r12, [rcx]
    mov r13, [rcx + 8]
    mov r14, [rcx + 16]
    mov r15, [rcx + 24]
    mov rdi, [rcx + 32]
    mov rsi, [rcx + 40]
    mov rbx, [rcx + 48]
    mov rbp, [rcx + 56]
    mov rsp, [rcx + 64]
    ret

