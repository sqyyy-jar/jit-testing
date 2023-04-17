.global asm_launch_runner
.global asm_return_runner

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