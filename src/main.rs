use std::{
    alloc::{alloc, Layout},
    arch::asm,
    ffi::c_void,
    mem,
};

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
    pub fn launch(&mut self, _ctx: &mut Context) {
        unsafe {
            asm! {
                "add rsp, 8",
                "mov [rdi], rbx",
                "mov [rdi + 8], rsp",
                "mov [rdi + 16], rbp",
                "mov [rdi + 24], r12",
                "mov [rdi + 32], r13",
                "mov [rdi + 40], r14",
                "mov [rdi + 48], r15",
                "jmp rax",
                in("rax") Runner::run
            };
        }
    }

    fn run(&mut self, _ctx: &mut Context) {
        unsafe {
            asm! {
                "push r14",
                "mov r14, rdi",
                "push rbx",
                "mov rbx, rsi",
                "cmp byte ptr [rdi + 64], 0",
                "je 2f",
                "1:",
                "mov rdi, rbx",
                "mov rsi, r14",
                "call r15",
                "cmp byte ptr [r14 + 64], 0",
                "jne 1f",
                "2:",
                "mov rsi, rbx",
                "mov rdi, r14",
                "mov rbx, [rdi]",
                "mov rsp, [rdi + 8]",
                "mov rbp, [rdi + 16]",
                "mov r12, [rdi + 24]",
                "mov r13, [rdi + 32]",
                "mov r14, [rdi + 40]",
                "mov r15, [rdi + 48]",
                "ret",
                in("r15") Context::step
            };
        }
    }
}

impl Context {
    pub fn step(&mut self, _runner: &mut Runner) {
        todo!()
    }
}

fn main() {
    let mut context = Context {
        regs: [0; 32],
        funcs: Vec::with_capacity(0),
    };
    let mut runner = Runner {
        snapshot: [0; 7],
        ctx: &mut context,
        running: true,
    };
    store_ret(&mut runner);
    println!("main = {:?}", main as *const c_void);
    let new_stack = unsafe { alloc(Layout::new::<[u8; 1024 * 64]>()).add(1024 * 64) };
    println!("new_stack = {new_stack:?}");
    println!("Hello before!");
    execute_vstack(new_stack, || {
        println!("Hello inside!");
    });
    println!("Hello after!");
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
