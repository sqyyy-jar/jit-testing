pub mod asm;
pub mod opcodes;
pub mod runtime;

use std::{mem, ptr::null_mut};

use dynasmrt::{dynasm, x64::X64Relocation, Assembler, DynasmApi, ExecutableBuffer};
use runtime::{Context, Runner, Value};

fn main() {
    let mut ctx = Context {
        regs: [Value { uint: 0 }; 8],
        pc: null_mut(),
        mem: vec![0; u16::MAX as usize].into_boxed_slice(),
        funcs: Vec::with_capacity(0),
        callstack: Vec::new(),
    };
    let mut runner = Runner::new(&mut ctx);
    runner.run();
}

pub fn create_stub() -> (ExecutableBuffer, fn(u64) -> u64) {
    let mut ops = Assembler::<X64Relocation>::new().unwrap();
    let offset = ops.offset();
    dynasm!(ops
        ; .arch x64
        ; mov [rsp - 8], rdi
        ; movups xmm0, [rsp - 16]
        ; xor rdi, rdi
        ; mov [rsp - 8], rdi
        ; movups [rsp - 16], xmm0
        ; mov rax, [rsp - 8]
        ; ret
    );
    let buf = ops.finalize().unwrap();
    let stub = unsafe { mem::transmute(buf.ptr(offset)) };
    (buf, stub)
}
