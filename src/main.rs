#[cfg(not(target_pointer_width = "64"))]
compile_error!("CPU must be 64-bit");

pub mod asm;
pub mod opcodes;
pub mod runtime;

use opcodes::{__call, __iload, __imul, __jumpnz, __load, __mul, __print, __return, __sub};
use runtime::{Context, Func, Runner};

fn main() {
    let main = [
        // __load(0, 21),
        // __load(1, 2),
        // __mul(0, 1),
        // __print(0),
        __call(2),
        __return(),
    ];
    let jitted = [
        __iload(0, -21),
        __iload(1, 2),
        __imul(0, 1),
        __print(0),
        __return(),
    ];
    let prints = [
        __load(0, 100),
        __mul(0, 0),
        __load(1, 100),
        __mul(0, 1),
        // L1
        __load(1, 1),
        __sub(0, 1),
        __print(0),
        __jumpnz(0, -2),
        __return(),
    ];
    let mut ctx = Context::default();
    let mut runner = Runner::new(&mut ctx);
    let main = Func::new(main.to_vec());
    let jitted = Func::new(jitted.to_vec());
    let prints = Func::new(prints.to_vec());
    ctx.funcs.push(main);
    ctx.funcs.push(jitted);
    ctx.funcs.push(prints);
    Func::compile(&mut ctx.funcs, 2).unwrap();
    ctx.pc = ctx.funcs[0].addr.address as _;
    runner.run();
}
