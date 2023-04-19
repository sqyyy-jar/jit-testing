use std::{
    alloc::{alloc, dealloc, Layout},
    mem,
    ptr::{null, null_mut},
};

use dynasmrt::{dynasm, x64::X64Relocation, Assembler, DynasmApi, ExecutableBuffer};

use crate::opcodes::{
    ADD, CALL, DIV, HALT, IDIV, ILOAD, IMUL, IREM, JUMP, JUMPNZ, JUMPZ, LOAD, MEMLOAD, MEMSTORE,
    MOVE, MUL, NOOP, PRINT, REM, RETURN, SMALLOP, SUB,
};

#[derive(Clone, Copy)]
pub union Value {
    pub uint: u64,
    pub int: i64,
    pub size: usize,
}

#[repr(C)]
pub struct Runner {
    #[cfg(target_family = "unix")]
    pub snapshot: [usize; 7],
    #[cfg(target_family = "windows")]
    pub snapshot: [usize; 9],
    pub ctx: *mut Context,
    pub running: bool,
}

impl Runner {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            #[cfg(target_family = "unix")]
            snapshot: [0; 7],
            #[cfg(target_family = "windows")]
            snapshot: [0; 9],
            ctx,
            running: true,
        }
    }

    pub fn run(&mut self) {
        while self.running {
            unsafe { &mut *self.ctx }.step(self);
        }
    }
}

#[repr(C)]
pub struct Stack<T> {
    size: usize,
    bp: *mut T,
    sp: *mut T,
}

impl<T> Stack<T> {
    pub fn new(size: usize) -> Self {
        let bp = unsafe { (alloc(Layout::array::<T>(size).unwrap()) as *mut T).add(size) };
        Self { size, bp, sp: bp }
    }

    pub fn push(&mut self, value: T) {
        unsafe {
            self.sp = self.sp.sub(1);
            *self.sp = value;
        }
    }

    pub fn will_underflow(&self) -> bool {
        self.sp >= self.bp
    }

    pub fn will_overflow(&self) -> bool {
        self.sp <= unsafe { self.bp.sub(self.size) }
    }

    pub fn is_underflown(&self) -> bool {
        self.sp > self.bp
    }

    pub fn is_overflown(&self) -> bool {
        self.sp < unsafe { self.bp.sub(self.size) }
    }
}

impl<T: Copy> Stack<T> {
    pub fn pop(&mut self) -> T {
        unsafe {
            let value = *self.sp;
            self.sp = self.sp.add(1);
            value
        }
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.bp.sub(self.size) as *mut u8,
                Layout::array::<T>(self.size).unwrap(),
            )
        }
    }
}

#[repr(C)]
pub struct Context {
    pub regs: [Value; 8],
    pub pc: *const u16,
    pub callstack: Stack<Address>,
    pub mem: Box<[u8]>,
    pub funcs: Vec<Func>,
    pub buffers: Vec<ExecutableBuffer>,
}

impl Context {
    pub fn step(&mut self, runner: &mut Runner) {
        let insn = unsafe { *self.pc };
        let opc = insn & 0xf000;
        match opc {
            SMALLOP => {
                let op = insn & 0xf00;
                match op {
                    NOOP => {}
                    MOVE => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        self.regs[dst as usize] = self.regs[src as usize];
                    }
                    MEMLOAD => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        let addr = unsafe { self.regs[src as usize].size };
                        if self.mem.len() <= addr {
                            runner.running = false;
                            eprintln!("Invalid memory access: 0x{insn:04x}");
                            return;
                        }
                        self.regs[dst as usize] =
                            unsafe { *(self.mem.as_ptr().add(addr) as *const Value) };
                    }
                    MEMSTORE => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        let addr = unsafe { self.regs[dst as usize].size };
                        if self.mem.len() <= addr {
                            runner.running = false;
                            eprintln!("Invalid memory access: 0x{insn:04x}");
                            return;
                        }
                        unsafe {
                            *(self.mem.as_ptr().add(addr) as *mut Value) = self.regs[src as usize]
                        };
                    }
                    RETURN => {
                        if self.callstack.will_underflow() {
                            runner.running = false;
                            return;
                        }
                        let ret_addr = self.callstack.pop();
                        if ret_addr.native {
                            todo!("native");
                        }
                        self.pc = ret_addr.address as *const u16;
                        return;
                    }
                    ADD => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].uint += self.regs[src as usize].uint };
                    }
                    SUB => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].uint -= self.regs[src as usize].uint };
                    }
                    MUL => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].uint *= self.regs[src as usize].uint };
                    }
                    IMUL => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].int *= self.regs[src as usize].int };
                    }
                    DIV => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].uint /= self.regs[src as usize].uint };
                    }
                    IDIV => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].int /= self.regs[src as usize].int };
                    }
                    REM => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].uint %= self.regs[src as usize].uint };
                    }
                    IREM => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        unsafe { self.regs[dst as usize].int %= self.regs[src as usize].int };
                    }
                    PRINT => {
                        let src = insn & 0x7;
                        println!("{}", unsafe { self.regs[src as usize].int });
                    }
                    HALT => {
                        runner.running = false;
                        return;
                    }
                    _ => {
                        runner.running = false;
                        eprintln!("Invalid small instruction: 0x{insn:04x}");
                        return;
                    }
                }
            }
            LOAD => {
                let dst = insn & 0x7;
                let value = (insn & 0xff8) >> 3;
                self.regs[dst as usize].uint = value as u64;
            }
            ILOAD => {
                let dst = insn & 0x7;
                let value = sign_extend::<9>((insn & 0xff8) >> 3);
                self.regs[dst as usize].int = value;
            }
            JUMP => {
                let offset = sign_extend::<12>(insn & 0xfff);
                self.pc = unsafe { self.pc.offset(offset as isize) };
            }
            JUMPZ => {
                let cond = insn & 0x7;
                let offset = sign_extend::<9>((insn & 0xff8) >> 3);
                if unsafe { self.regs[cond as usize].uint } == 0 {
                    self.pc = unsafe { self.pc.offset(offset as isize) };
                    return;
                }
            }
            JUMPNZ => {
                let cond = insn & 0x7;
                let offset = sign_extend::<9>((insn & 0xFF8) >> 3);
                if unsafe { self.regs[cond as usize].uint } != 0 {
                    self.pc = unsafe { self.pc.offset(offset as isize) };
                    return;
                }
            }
            CALL => {
                let index = insn & 0xfff;
                let Some(func) = self.funcs.get(index as usize) else {
                    runner.running = false;
                    eprintln!("Invalid function: 0x{insn:04x}");
                    return;
                };
                if self.callstack.will_overflow() {
                    runner.running = false;
                    eprintln!("Callstack overflow: 0x{insn:04x}");
                    return;
                }
                if func.addr.native {
                    todo!("native")
                }
                self.callstack.push(Address {
                    native: false,
                    address: unsafe { self.pc.add(1) as *const () },
                });
                self.pc = unsafe { self.pc.offset(index as isize) };
            }
            _ => {
                runner.running = false;
                eprintln!("Invalid instruction: 0x{insn:04x}");
                return;
            }
        }
        self.pc = unsafe { self.pc.add(1) };
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            regs: [Value { uint: 0 }; 8],
            pc: null_mut(),
            callstack: Stack::new(1024 * 8),
            mem: vec![0; u16::MAX as usize].into_boxed_slice(),
            funcs: Vec::with_capacity(0),
            buffers: Vec::with_capacity(0),
        }
    }
}

pub type NativeAccessFunc = fn(*mut Runner, *mut Context);

pub struct Func {
    pub code: Vec<u16>,
    pub addr: Address,
    pub func: NativeAccessFunc,
    pub buf: ExecutableBuffer,
}

impl Func {
    pub fn new(code: Vec<u16>) -> Self {
        let mut res = Self {
            code,
            addr: Address {
                native: false,
                address: null(),
            },
            func: |_, _| {},
            buf: ExecutableBuffer::default(),
        };
        res.addr.address = res.code.as_ptr() as *const ();
        res
    }
}

#[derive(Clone, Copy)]
pub struct Address {
    pub native: bool,
    pub address: *const (),
}

#[inline(always)]
pub const fn sign_extend<const BITS: usize>(value: u16) -> i64 {
    if ((value >> (BITS - 1)) & 1) != 0 {
        (value | (!0) << BITS) as i32 as _
    } else {
        value as _
    }
}

fn generate_stub(addr: *const ()) -> (ExecutableBuffer, NativeAccessFunc) {
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
