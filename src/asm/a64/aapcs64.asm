.global asm_snapshot
.global asm_return_virtual_native
.global asm_call_virtual_native
.global asm_return_native_virtual
.global asm_print
.global asm_halt

asm_snapshot: // (x0: *Runner) aapcs64
    // Registers
    str x18, [x0]
    stp x19, x20, [x0, 8]
    stp x21, x22, [x0, 24]
    stp x23, x24, [x0, 40]
    stp x25, x26, [x0, 56]
    stp x27, x28, [x0, 72]
    stp lr, fp, [x0, 88]
    mov x1, sp
    str x1, [x0, 104]
    // Stack top
    ldp x1, x2, [sp]
    stp x1, x2, [x0, 112]
    ldp x1, x2, [sp, 16]
    stp x1, x2, [x0, 128]
    ret

asm_return_virtual_native: // (x0: *Runner, x1: *Context) aapcs64
    mov x19, x1 // ctx
    mov x20, x0 // runner
    // Load mapped registers
    ldr x21, [x19, 0x58] // callstack
    ldr lr, [x21], 0x8 // return address
    ret

asm_call_virtual_native: // (x0: *Runner, x1: *Context, x2: usize) aapcs64
    mov x19, x1 // ctx
    mov x20, x0 // runner
    // Load mapped registers
    ldr x21, [x19, 0x58]
    br x2

asm_return_native_virtual: // (x20: *Runner, x19: *Context) custom
    mov x0, x20
    mov x1, x19
    // Registers
    ldr x18, [x0]
    // todo
    stp x19, x20, [x0, 8]
    stp x21, x22, [x0, 24]
    stp x23, x24, [x0, 40]
    stp x25, x26, [x0, 56]
    stp x27, x28, [x0, 72]
    stp lr, fp, [x0, 88]
    mov x1, sp
    str x1, [x0, 104]
    // Stack top
    ldp x1, x2, [sp]
    stp x1, x2, [x0, 112]
    ldp x1, x2, [sp, 16]
    stp x1, x2, [x0, 128]
    ret
asm_print:
asm_halt:
    ret
