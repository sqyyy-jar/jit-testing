#[cfg(not(target_pointer_width = "64"))]
compile_error!("CPU must be 64-bit");

pub mod asm;
pub mod opcodes;
pub mod runtime;

use std::mem;

use dynasmrt::{dynasm, x64::X64Relocation, Assembler, DynasmApi, ExecutableBuffer};
use opcodes::{__call, __iload, __imul, __print, __return, __load, __mul};
use runtime::{Context, Func, Runner};

fn main() {
    let main = [
        // __load(0, 21),
        // __load(1, 2),
        // __mul(0, 1),
        // __print(0),
        __call(1),
        __return(),
    ];
    let jitted = [
        __load(0, 21),
        __load(1, 2),
        __mul(0, 1),
        // __load(0, 42),
        __print(0),
        __return(),
    ];
    let mut ctx = Context::default();
    let mut runner = Runner::new(&mut ctx);
    let main = Func::new(main.to_vec());
    let jitted = Func::new(jitted.to_vec());
    ctx.funcs.push(main);
    ctx.funcs.push(jitted);
    Func::compile(&mut ctx.funcs, 1).unwrap();
    ctx.pc = ctx.funcs[0].addr.address as _;
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
