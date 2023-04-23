#[cfg(not(target_pointer_width = "64"))]
compile_error!("CPU must be 64-bit");

pub mod asm;
pub mod opcodes;
pub mod runtime;

use opcodes::{__call, __iload, __imul, __print, __return};
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
        __iload(0, -21),
        __iload(1, 2),
        __imul(0, 1),
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
