#[cfg(not(target_pointer_width = "64"))]
compile_error!("CPU must be 64-bit");

pub mod asm;
pub mod opcodes;
pub mod runtime;

use opcodes::{__call, __iload, __imul, __print, __return};
use runtime::{Context, Func, Runner};

fn main() {
    let main = [__call(1), __return()];
    let code = [
        __iload(0, -21),
        __iload(1, 2),
        __imul(0, 1),
        __print(0),
        __return(),
    ];
    let mut ctx = Context::default();
    let mut runner = Runner::default();
    let main = Func::new(main.to_vec());
    let jitted = Func::new(code.to_vec());
    ctx.funcs.push(main);
    ctx.funcs.push(jitted);
    Func::compile(&mut ctx.funcs, 1).unwrap();
    ctx.pc = ctx.funcs[0].addr.address as _;
    runner.run(&mut ctx);
}

#[cfg(test)]
mod tests {
    use crate::{
        opcodes::{__add, __call, __div, __idiv, __iload, __load, __mul, __return, __sub},
        runtime::{Context, Func, Runner},
    };

    fn ctx(code: &[u16]) -> (Context, Runner) {
        let mut ctx = Context::default();
        let runner = Runner::default();
        let main = Func::new([__call(1), __return()].to_vec());
        let func = Func::new(code.to_vec());
        ctx.funcs.push(main);
        ctx.funcs.push(func);
        ctx.pc = ctx.funcs[0].addr.address as *const u16;
        (ctx, runner)
    }

    fn run(code: &[u16]) -> Context {
        let (mut ctx, mut runner) = ctx(code);
        runner.run(&mut ctx);
        ctx
    }

    fn run_jitted(code: &[u16]) -> Context {
        let (mut ctx, mut runner) = ctx(code);
        Func::compile(&mut ctx.funcs, 1).unwrap();
        runner.run(&mut ctx);
        ctx
    }

    #[test]
    fn test_addition() {
        let code = [__load(0, 3), __load(1, 5), __add(0, 1), __return()];
        let ctx = run(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, 8);
        let ctx = run_jitted(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, 8);
    }

    #[test]
    fn test_subtraction() {
        let code = [__load(0, 3), __load(1, 5), __sub(0, 1), __return()];
        let ctx = run(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, -2);
        let ctx = run_jitted(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, -2);
    }

    #[test]
    fn test_multiplication() {
        let code = [__load(0, 3), __load(1, 5), __mul(0, 1), __return()];
        let ctx = run(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, 15);
        let ctx = run_jitted(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, 15);
    }

    #[test]
    fn test_division() {
        let code = [__load(0, 15), __load(1, 5), __div(0, 1), __return()];
        let ctx = run(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, 3);
        let ctx = run_jitted(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, 3);
    }

    #[test]
    fn test_signed_division() {
        let code = [__iload(0, 15), __iload(1, -5), __idiv(0, 1), __return()];
        let ctx = run(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, -3);
        let ctx = run_jitted(&code);
        assert_eq!(unsafe { ctx.regs[0].int }, -3);
    }
}
