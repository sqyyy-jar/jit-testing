use std::{
    alloc::{alloc, dealloc, Layout},
    collections::HashMap,
    mem,
    ptr::{null, null_mut},
};

use anyhow::anyhow;
use dynasmrt::{dynasm, Assembler, DynasmApi, DynasmLabelApi, ExecutableBuffer};

use crate::{
    asm::{
        call_virtual_native, halt, print, return_native_virtual, return_virtual_native, snapshot,
    },
    opcodes::{
        ADD, CALL, DIV, HALT, IDIV, ILOAD, IMUL, IREM, JUMP, JUMPNZ, JUMPZ, LOAD, MEMLOAD,
        MEMSTORE, MOVE, MUL, NOOP, PRINT, REM, RETURN, SMALLOP, SUB,
    },
};

macro_rules! asm {
    ($ops:ident $($t:tt)*) => {
        dynasm!($ops
            ; .arch x64
            ; .alias t0, rax
            ; .alias t1, rbx
            ; .alias t2, rcx
            ; .alias t3, rdx
            ; .alias ctx, rsi
            ; .alias runner, rdi
            $($t)*
        )
    }
}

#[derive(Clone, Copy)]
pub union Value {
    pub uint: u64,
    pub int: i64,
    pub size: usize,
}

#[repr(C)]
#[derive(Default)]
pub struct Snapshot {
    #[cfg(target_family = "unix")]
    pub regs: [usize; 7],
    #[cfg(target_family = "windows")]
    pub regs: [usize; 9],
    pub stack_top: [usize; 4],
}

#[repr(C)]
pub struct Runner {
    pub snapshot: Snapshot,
    pub ctx: *mut Context,
    pub running: bool,
}

impl Runner {
    pub fn new(ctx: &mut Context) -> Self {
        Self {
            snapshot: Snapshot::default(),
            ctx,
            running: true,
        }
    }

    pub fn run(&mut self) {
        unsafe { snapshot(self) };
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
    pub callstack: Stack<*const ()>,
    pub mem: *mut u8,
    pub funcs: Vec<Func>,
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
                        let addr = unsafe { self.regs[src as usize].size } & 0xffff;
                        self.regs[dst as usize] = unsafe { *(self.mem.add(addr) as *const Value) };
                    }
                    MEMSTORE => {
                        let dst = insn & 0x7;
                        let src = (insn & 0x38) >> 3;
                        let addr = unsafe { self.regs[dst as usize].size } & 0xffff;
                        unsafe { *(self.mem.add(addr) as *mut Value) = self.regs[src as usize] };
                    }
                    RETURN => {
                        if self.callstack.will_underflow() {
                            runner.running = false;
                            return;
                        }
                        let ret_addr = self.callstack.pop();
                        if ret_addr.is_null() {
                            unsafe { return_virtual_native(runner, self) };
                            return;
                        }
                        self.pc = ret_addr as *const u16;
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
                return;
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
                let offset = sign_extend::<9>((insn & 0xff8) >> 3);
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
                    self.callstack.push(unsafe { self.pc.add(1) as *const () });
                    self.callstack.push(return_native_virtual as *const ());
                    let addr = func.addr.address;
                    unsafe { call_virtual_native(runner, self, addr) };
                    return;
                }
                self.callstack.push(unsafe { self.pc.add(1) as *const () });
                self.pc = func.addr.address as *const u16;
                return;
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
            callstack: Stack::new(1024 * 4),
            mem: unsafe { alloc(Layout::array::<u8>(u16::MAX as usize + 8).unwrap()) },
            funcs: Vec::with_capacity(0),
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.mem, Layout::array::<u8>(u16::MAX as usize).unwrap());
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
        let (buf, func) = generate_stub(res.addr.address);
        res.func = func;
        res.buf = buf;
        res
    }

    #[cfg(target_arch = "x86_64")]
    pub fn compile(funcs: &mut [Func], index: usize) -> anyhow::Result<()> {
        let func = &funcs[index];
        let mut ops = Assembler::<dynasmrt::x64::X64Relocation>::new().unwrap();
        let start = ops.offset();
        let mut labels = HashMap::with_capacity(0);
        for (i, insn) in func.code.iter().enumerate() {
            let opc = *insn & 0xf000;
            match opc {
                SMALLOP => {
                    let op = insn & 0xf00;
                    match op {
                        NOOP | MOVE | MEMLOAD | MEMSTORE | RETURN | ADD | SUB | MUL | IMUL
                        | DIV | IDIV | REM | IREM | PRINT | HALT => {}
                        _ => {
                            return Err(anyhow!("Invalid small instruction: 0x{insn:04x}"));
                        }
                    }
                }
                LOAD | ILOAD => {}
                JUMP => {
                    let offset = sign_extend::<12>(insn & 0xfff);
                    let target = i as isize + offset as isize;
                    if target < 0 || target >= func.code.len() as isize {
                        return Err(anyhow!("Invalid jump: 0x{insn:04x}"));
                    }
                    if labels.contains_key(&(target as usize)) {
                        continue;
                    }
                    labels.insert(target as usize, ops.new_dynamic_label());
                }
                JUMPZ => {
                    let offset = sign_extend::<9>((insn & 0xff8) >> 3);
                    let target = i as isize + offset as isize;
                    if target < 0 || target >= func.code.len() as isize {
                        return Err(anyhow!("Invalid jump: 0x{insn:04x}"));
                    }
                    if labels.contains_key(&(target as usize)) {
                        continue;
                    }
                    labels.insert(target as usize, ops.new_dynamic_label());
                }
                JUMPNZ => {
                    let offset = sign_extend::<9>((insn & 0xff8) >> 3);
                    let target = i as isize + offset as isize;
                    if target < 0 || target >= func.code.len() as isize {
                        return Err(anyhow!("Invalid jump: 0x{insn:04x}"));
                    }
                    if labels.contains_key(&(target as usize)) {
                        continue;
                    }
                    labels.insert(target as usize, ops.new_dynamic_label());
                }
                CALL => {
                    let target = i + 1;
                    if target >= func.code.len() {
                        return Err(anyhow!("Invalid call: 0x{insn:04x}"));
                    }
                    if labels.contains_key(&target) {
                        continue;
                    }
                    labels.insert(target, ops.new_dynamic_label());
                }
                _ => {
                    return Err(anyhow!("Invalid instruction: 0x{insn:04x}"));
                }
            }
        }
        for (i, insn) in func.code.iter().enumerate() {
            if let Some(target) = labels.get(&i) {
                ops.dynamic_label(*target);
            }
            let opc = *insn & 0xf000;
            match opc {
                SMALLOP => {
                    let op = insn & 0xf00;
                    match op {
                        NOOP => {}
                        MOVE => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov [BYTE ctx + dst], t0
                            );
                        }
                        MEMLOAD => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + 96]
                                ; mov t1, [BYTE ctx + src]
                                ; and t1, 0xffff
                                ; add t0, t1
                                ; mov t0, [t0]
                                ; mov [BYTE ctx + dst], t0
                            );
                        }
                        MEMSTORE => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + 96]
                                ; mov t1, [BYTE ctx + dst]
                                ; and t1, 0xffff
                                ; add t0, t1
                                ; mov t1, [BYTE ctx + src]
                                ; mov [t0], t1
                            );
                        }
                        RETURN => {
                            asm!(ops
                                ; ret
                            );
                        }
                        ADD => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; add [BYTE ctx + dst], t0
                            );
                        }
                        SUB => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; sub [BYTE ctx + dst], t0
                            );
                        }
                        MUL => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov t3, [BYTE ctx + dst]
                                ; mul t3
                                ; mov [BYTE ctx + dst], t0
                            );
                        }
                        IMUL => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov t3, [BYTE ctx + dst]
                                ; imul t3
                                ; mov [BYTE ctx + dst], t0
                            );
                        }
                        DIV => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov t3, [BYTE ctx + dst]
                                ; div t3
                                ; mov [BYTE ctx + dst], t0
                            );
                        }
                        IDIV => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov t3, [BYTE ctx + dst]
                                ; idiv t3
                                ; mov [BYTE ctx + dst], t0
                            );
                        }
                        REM => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov t3, [BYTE ctx + dst]
                                ; div t3
                                ; mov [BYTE ctx + dst], t3
                            );
                        }
                        IREM => {
                            let dst = ((insn & 0x7) * 8) as i8;
                            let src = (((insn & 0x38) >> 3) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov t3, [BYTE ctx + dst]
                                ; idiv t3
                                ; mov [BYTE ctx + dst], t3
                            );
                        }
                        PRINT => {
                            let src = ((insn & 0x7) * 8) as i8;
                            asm!(ops
                                ; mov t0, [BYTE ctx + src]
                                ; mov t1, QWORD print as usize as i64
                                ; call t1
                            );
                        }
                        HALT => {
                            asm!(ops
                                ; mov QWORD [BYTE runner + 96], 0
                                ; mov t0, QWORD halt as usize as i64
                                ; jmp t0
                            );
                        }
                        _ => {
                            return Err(anyhow!("Invalid small instruction: 0x{insn:04x}"));
                        }
                    }
                }
                LOAD => {
                    let dst = ((insn & 0x7) * 8) as i8;
                    let value = ((insn & 0xff8) >> 3) as i32;
                    asm!(ops
                        ; mov QWORD [BYTE ctx + dst], value
                    );
                }
                ILOAD => {
                    let dst = ((insn & 0x7) * 8) as i8;
                    let value = sign_extend::<9>((insn & 0xff8) >> 3) as i32;
                    asm!(ops
                        ; mov QWORD [BYTE ctx + dst], value
                    );
                }
                JUMP => {
                    let offset = sign_extend::<12>(insn & 0xfff);
                    let target = (i as isize + offset as isize) as usize;
                    let label = labels[&target];
                    asm!(ops
                        ; jmp =>label
                    );
                }
                JUMPZ => {
                    let cond = ((insn & 0x7) * 8) as i8;
                    let offset = sign_extend::<9>((insn & 0xff8) >> 3);
                    let target = (i as isize + offset as isize) as usize;
                    let label = labels[&target];
                    asm!(ops
                        ; mov t0, [BYTE ctx + cond]
                        ; test t0, t0
                        ; jz =>label
                    );
                }
                JUMPNZ => {
                    let cond = ((insn & 0x7) * 8) as i8;
                    let offset = sign_extend::<9>((insn & 0xff8) >> 3);
                    let target = (i as isize + offset as isize) as usize;
                    let label = labels[&target];
                    asm!(ops
                        ; mov t0, [BYTE ctx + cond]
                        ; test t0, t0
                        ; jnz =>label
                    );
                }
                CALL => {
                    let call_index = insn & 0xfff;
                    let Some(callee) = funcs.get(call_index as usize) else {
                        return Err(anyhow!("Invalid function: 0x{insn:04x}"));
                    };
                    let addr = callee.func;
                    asm!(ops
                        ; mov t0, QWORD addr as usize as i64
                        ; call t0
                    );
                }
                _ => return Err(anyhow!("Invalid instruction: 0x{insn:04x}")),
            }
        }
        let func = &mut funcs[index];
        let buf = ops.finalize().unwrap();
        let exec = unsafe { mem::transmute(buf.ptr(start)) };
        func.buf = buf;
        func.func = exec;
        func.addr.native = true;
        func.addr.address = func.func as *const ();
        Ok(())
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
        (value | (!0) << BITS) as i16 as _
    } else {
        value as _
    }
}

pub extern "C" fn print_num(num: i64) {
    println!("{num}");
}

#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
fn generate_stub(addr: *const ()) -> (ExecutableBuffer, NativeAccessFunc) {
    let mut ops = Assembler::<dynasmrt::x64::X64Relocation>::new().unwrap();
    let offset = ops.offset();
    asm!(ops // -> native (*Runner, *Context)
        // Save mapped registers
        ; push 0
        ; mov t0, QWORD addr as i64
        ; mov [BYTE ctx + 64], t0 // virtual address
        ; mov [BYTE ctx + 88], rsp // callstack
        // Restore snapshot
        ; mov rbx, [runner]
        ; mov rsp, [BYTE runner + 8]
        ; mov rbp, [BYTE runner + 16]
        ; mov r12, [BYTE runner + 24]
        ; mov r13, [BYTE runner + 32]
        ; mov r14, [BYTE runner + 40]
        ; mov r15, [BYTE runner + 48]
        ; movups xmm0, [BYTE runner + 56]
        ; movups [rsp], xmm0
        ; movups xmm0, [BYTE runner + 72]
        ; movups [BYTE rsp + 16], xmm0
        ; ret
    );
    let buf = ops.finalize().unwrap();
    let stub = unsafe { mem::transmute(buf.ptr(offset)) };
    (buf, stub)
}

#[cfg(all(target_arch = "x86_64", target_family = "windows"))]
fn generate_stub(addr: *const ()) -> (ExecutableBuffer, NativeAccessFunc) {
    let mut ops = Assembler::<dynasmrt::x64::X64Relocation>::new().unwrap();
    let offset = ops.offset();
    asm!(ops // -> native (*Runner, *Context)
        // Save mapped registers
        ; push 0
        ; mov t0, QWORD addr as i64
        ; mov [BYTE ctx + 64], t0 // virtual address
        ; mov [BYTE ctx + 88], rsp // callstack
        // Restore snapshot
        ; mov rcx, runner
        ; mov rbx, [rcx]
        ; mov rsp, [BYTE rcx + 8]
        ; mov rbp, [BYTE rcx + 16]
        ; mov rsi, [BYTE rcx + 24]
        ; mov rdi, [BYTE rcx + 32]
        ; mov r12, [BYTE rcx + 40]
        ; mov r13, [BYTE rcx + 48]
        ; mov r14, [BYTE rcx + 56]
        ; mov r15, [BYTE rcx + 64]
        ; movups xmm0, [BYTE rcx + 72]
        ; movups [rsp], xmm0
        ; movups xmm0, [BYTE rcx + 88]
        ; movups [BYTE rsp + 16], xmm0
        ; ret
    );
    let buf = ops.finalize().unwrap();
    let stub = unsafe { mem::transmute(buf.ptr(offset)) };
    (buf, stub)
}
