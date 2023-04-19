.global asm_launch_runner
.global asm_return_runner

asm_launch_runner:
    mov [rcx], r12
    mov [rcx + 8], r13
    mov [rcx + 16], r14
    mov [rcx + 24], r15
    mov [rcx + 32], rdi
    mov [rcx + 40], rsi
    mov [rcx + 48], rbx
    mov [rcx + 56], rbp
    mov [rcx + 56], rsp
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
    mov rsp, [rcx + 56]
    ret
