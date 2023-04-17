use crate::{Runner, Context};
use std::arch::global_asm;

global_asm! {
    include_str!("extern_asm/system_v.asm"),
    run=sym Runner::run
}

#[allow(improper_ctypes)]
extern "C" {
    pub(crate) fn asm_launch_runner(runner: *mut Runner, ctx: *mut Context);

    pub(crate) fn asm_return_runner(runner: *mut Runner, ctx: *mut Context);
}
