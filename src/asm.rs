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
    #[link_name = "asm_snapshot"]
    pub(crate) fn snapshot(runner: *mut Runner);

    #[link_name = "asm_return_virtual_native"]
    pub(crate) fn return_virtual_native(runner: *mut Runner, ctx: *mut Context);

    #[link_name = "asm_call_virtual_native"]
    pub(crate) fn call_virtual_native(runner: *mut Runner, ctx: *mut Context, addr: *const ());

    #[link_name = "asm_return_native_virtual"]
    pub(crate) fn return_native_virtual(runner: *mut Runner, ctx: *mut Context);

    #[link_name = "asm_halt"]
    pub(crate) fn halt(runner: *mut Runner, ctx: *mut Context);

    // pub(crate) fn asm_launch_runner(runner: *mut Runner, ctx: *mut Context);

    // pub(crate) fn asm_return_runner(runner: *mut Runner, ctx: *mut Context);

    // pub(crate) fn asm_enter_native(runner: *mut Runner, ctx: *mut Context, address: *const ());
}
