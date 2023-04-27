.global asm_snapshot
.global asm_return_virtual_native
.global asm_call_virtual_native
.global asm_return_native_virtual
.global asm_print
.global asm_halt

asm_snapshot: // (x0: *Runner) aapcs64
    // Registers
    str x18, [x0]
    stp x19, x20, [x0, 0x8]
    stp x21, x22, [x0, 0x18]
    stp x23, x24, [x0, 0x28]
    stp x25, x26, [x0, 0x38]
    stp x27, x28, [x0, 0x48]
    stp lr, fp, [x0, 0x58]
    mov x1, sp
    str x1, [x0, 0x68]
    // Stack top
    ldp x1, x2, [sp]
    stp x1, x2, [x0, 0x70]
    ldp x1, x2, [sp, 0x10]
    stp x1, x2, [x0, 0x80]
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
    ldr x21, [x19, 0x58] // callstack
    ldr lr, [x21], 0x8 // asm_return_native_virtual
    br x2

asm_return_native_virtual: // (x20: *Runner, x19: *Context) custom
    // Save mapped registers
    ldr x0, [x21], 0x8
    str x0, [x19, 0x40] // virtual return address
    str x21, [x19, 0x58] // callstack
    // Registers
    mov x0, x20
    ldr x18, [x0]
    ldp x19, x20, [x0, 0x8]
    ldp x21, x22, [x0, 0x18]
    ldp x23, x24, [x0, 0x28]
    ldp x25, x26, [x0, 0x38]
    ldp x27, x28, [x0, 0x48]
    ldp lr, fp, [x0, 0x58]
    ldr x1, [x0, 0x68]
    mov sp, x1
    // Stack top
    ldp x1, x2, [x0, 0x70]
    stp x1, x2, [sp]
    ldp x1, x2, [x0, 0x80]
    stp x1, x2, [sp, 0x10]
    ret

asm_print: // (x20: *Runner, x19: *Context, x0: i64) custom
    // Save mapped registers
    // Save state
    stp x29, x30, [sp, -0x10]!
    // Call
    bl {print_num}
    // Restore state
    ldp x29, x30, [sp], 0x10
    // Restore mapped registers
    ret

asm_halt: // (x20: *Runner, x19: *Context) custom
    str xzr, [x20, 0x60] // running
    // Save mapped registers
    str x21, [x19, 0x58] // callstack
    // Restore snapshot
    mov x0, x20
    ldr x18, [x0]
    ldp x19, x20, [x0, 0x8]
    ldp x21, x22, [x0, 0x18]
    ldp x23, x24, [x0, 0x28]
    ldp x25, x26, [x0, 0x38]
    ldp x27, x28, [x0, 0x48]
    ldp lr, fp, [x0, 0x58]
    ldr x1, [x0, 0x68]
    mov sp, x1
    // Stack top
    ldp x1, x2, [x0, 0x70]
    stp x1, x2, [sp]
    ldp x1, x2, [x0, 0x80]
    stp x1, x2, [sp, 0x10]
    ret
