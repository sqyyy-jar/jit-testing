use crate::{Context, Runner};
use std::arch::global_asm;

#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
global_asm! {
    include_str!("asm/system_v.asm"),
    run=sym Runner::run
}
#[cfg(all(target_arch = "x86_64", target_family = "windows"))]
global_asm! {
    include_str!("asm/windows.asm"),
    run=sym Runner::run
}

#[allow(improper_ctypes)]
extern "C" {
    pub(crate) fn asm_snapshot(runner: *mut Runner);

    pub(crate) fn asm_launch_runner(runner: *mut Runner, ctx: *mut Context);

    pub(crate) fn asm_return_runner(runner: *mut Runner, ctx: *mut Context);

    pub(crate) fn asm_enter_native(runner: *mut Runner, ctx: *mut Context, address: *const ());
}
