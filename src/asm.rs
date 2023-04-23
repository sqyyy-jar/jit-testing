use crate::{runtime::print_num, Context, Runner};
use std::arch::global_asm;

#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
global_asm! {
    include_str!("asm/x64/system_v.asm"),
    print_num=sym print_num
}
#[cfg(all(target_arch = "x86_64", target_family = "windows"))]
global_asm! {
    include_str!("asm/x64/windows.asm"),
    print_num=sym print_num
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

    #[link_name = "asm_print"]
    pub(crate) fn print(runner: *mut Runner, ctx: *mut Context, num: i64);

    #[link_name = "asm_halt"]
    pub(crate) fn halt(runner: *mut Runner, ctx: *mut Context);
}
