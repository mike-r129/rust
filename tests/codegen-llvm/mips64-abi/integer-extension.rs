//! The MIPS64 calling conventions (n64 and n32) pass integers narrower than 64 bits extended to
//! the full register: signed types and `u32` are sign-extended, the other unsigned types are
//! zero-extended. Check that `signext`/`zeroext` stay on such arguments, in definitions and at
//! call sites, also when other arguments of the same function need alignment padding.

// ignore-tidy-linelength
//@ add-minicore
//@ compile-flags: -Copt-level=3 -C no-prepopulate-passes
//
//@ revisions: mips64 mips64el n32
//@[mips64] compile-flags: --target mips64-unknown-linux-gnuabi64
//@[mips64] needs-llvm-components: mips
//@[mips64el] compile-flags: --target mips64el-unknown-linux-gnuabi64
//@[mips64el] needs-llvm-components: mips
// There is no in-tree n32 target, but custom targets can use that ABI.
//@[n32] compile-flags: --target {{src-base}}/mips64-abi/mips64-n32.json -Z unstable-options
//@[n32] needs-llvm-components: mips

#![feature(no_core, lang_items)]
#![crate_type = "lib"]
#![no_core]

extern crate minicore;
use minicore::*;

// CHECK: define{{.*}} void @narrow_args(i8 noundef signext %a, i8 noundef zeroext %b, i16 noundef signext %c, i16 noundef zeroext %d, i32 noundef signext %e, i32 noundef signext %f, i1 noundef zeroext %g)
#[no_mangle]
pub extern "C" fn narrow_args(a: i8, b: u8, c: i16, d: u16, e: i32, f: u32, g: bool) {}

// CHECK: define{{.*}} void @padded_args(i8 noundef zeroext %a, i32 %0, i128 %1, i16 noundef signext %c)
#[no_mangle]
pub extern "C" fn padded_args(a: u8, b: i128, c: i16) {}

// CHECK: define{{.*}} noundef signext i32 @ret_u32(i32 noundef signext %x)
#[no_mangle]
pub extern "C" fn ret_u32(x: u32) -> u32 {
    x
}

extern "C" {
    fn ext_narrow(a: i8, b: u8, c: i16, d: u16, e: i32, f: u32, g: bool);
}

// CHECK-LABEL: @call_narrow
// CHECK: call void @ext_narrow(i8 noundef signext -16, i8 noundef zeroext -1, i16 noundef signext -2, i16 noundef zeroext -1, i32 noundef signext -7, i32 noundef signext -2147483647, i1 noundef zeroext true)
#[no_mangle]
pub unsafe fn call_narrow() {
    ext_narrow(
        0xf0u8 as i8,
        0xff,
        0xfffeu16 as i16,
        0xffff,
        0xffff_fff9u32 as i32,
        0x8000_0001,
        true,
    );
}

// CHECK: declare void @ext_narrow(i8 noundef signext, i8 noundef zeroext, i16 noundef signext, i16 noundef zeroext, i32 noundef signext, i32 noundef signext, i1 noundef zeroext)
