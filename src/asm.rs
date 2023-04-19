use crate::{Runner, Context};
use std::arch::global_asm;

#[cfg(target_family="unix")]
global_asm! {
    include_str!("asm/system_v.asm"),
    run=sym Runner::run
}
#[cfg(target_family="windows")]
global_asm! {
    include_str!("asm/windows.asm"),
    run=sym Runner::run
}

#[allow(improper_ctypes)]
extern "C" {
    pub(crate) fn asm_launch_runner(runner: *mut Runner, ctx: *mut Context);

    pub(crate) fn asm_return_runner(runner: *mut Runner, ctx: *mut Context);
}
