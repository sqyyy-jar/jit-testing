pub mod asm;

use std::{mem, ptr::null_mut};

use dynasmrt::{dynasm, x64::X64Relocation, Assembler, DynasmApi, ExecutableBuffer};

#[repr(C)]
pub struct Runner {
    pub snapshot: [usize; 7],
    pub ctx: *mut Context,
    pub running: bool,
}

#[repr(C)]
pub struct Context {
    pub regs: [u64; 32],
    pub pc: *mut u16,
    pub funcs: Vec<Func>,
    pub callstack: Vec<*mut u16>,
}

pub struct Func {
    pub addr: Address,
    pub func: fn(),
}

pub struct Address {
    pub native: bool,
    pub address: usize,
}

impl Runner {
    pub fn launch(&mut self, ctx: &mut Context) {
        // unsafe {
        //     asm_launch_runner(self, ctx);
        // }
        self.run(ctx);
    }

    fn run(&mut self, ctx: &mut Context) {
        while self.running {
            ctx.step(self);
        }
        // unsafe {
        //     asm_return_runner(self, ctx);
        // }
    }
}

impl Context {
    pub fn step(&mut self, runner: &mut Runner) {
        println!("Hello world!");
        runner.running = false;
    }
}

fn main() {
    let mut ctx = Context {
        regs: [0; 32],
        pc: null_mut(),
        funcs: Vec::with_capacity(0),
        callstack: Vec::new(),
    };
    let mut runner = Runner {
        snapshot: [0; 7],
        ctx: &mut ctx,
        running: true,
    };
    runner.launch(&mut ctx);
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
