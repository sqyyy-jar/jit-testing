pub mod asm;

use std::{arch::asm, mem};

use asm::{asm_launch_runner, asm_return_runner};
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
    pub funcs: Vec<Func>,
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
        unsafe {
            asm_launch_runner(self, ctx);
        }
    }

    fn run(&mut self, ctx: &mut Context) {
        while self.running {
            ctx.step(self);
        }
        unsafe {
            asm_return_runner(self, ctx);
        }
    }
}

impl Context {
    #[inline(never)]
    pub fn step(&mut self, runner: &mut Runner) {
        println!("Hello world!");
        runner.running = false;
    }
}

fn main() {
    let mut ctx = Context {
        regs: [0; 32],
        funcs: Vec::with_capacity(0),
    };
    let mut runner = Runner {
        snapshot: [0; 7],
        ctx: &mut ctx,
        running: true,
    };
    runner.launch(&mut ctx);
    // store_ret(&mut runner);
    // println!("main = {:?}", main as *const c_void);
    // let new_stack = unsafe { alloc(Layout::new::<[u8; 1024 * 64]>()).add(1024 * 64) };
    // println!("new_stack = {new_stack:?}");
    // println!("Hello before!");
    // execute_vstack(new_stack, || {
    //     println!("Hello inside!");
    // });
    // println!("Hello after!");
}

pub fn execute_vstack(sp: *mut u8, exec: fn()) {
    unsafe {
        asm! {
            "push rbx",
            "sub rsp, 16",
            "mov [rsp], rbp",
            "mov [rsp + 8], rsp",
            "lea rbx, [rsp]",
            "mov rsp, {sp}",
            "mov rax, {exec}",
            "call rax",
            "mov rsp, [rbx + 8]",
            "add rsp, 16",
            "pop rbx",
            sp=in(reg) sp,
            exec=in(reg) exec
        };
    }
}

pub fn store_ret(_runner: &mut Runner) {
    unsafe {
        asm! {
            "pop rsi",
            "pop rdx",
            "mov [rdi + 24], rdx",
            "push rdx",
            "push rsi"
        };
    }
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
