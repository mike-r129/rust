//! On MIPS64 every argument slot is 64 bits wide, also with the n32 ABI where pointers are only
//! 32 bits. Check that rustc counts the hidden return pointer and `Complex` arguments in such
//! slots, like GCC does: the n32 return pointer must not cause padding of the next argument, and
//! a `Complex<f64>` that still has two free slots goes in floating-point registers.

// ignore-tidy-linelength
//@ add-minicore
//@ compile-flags: -Copt-level=3 -C no-prepopulate-passes
//
//@ revisions: n32 mips64
// There is no in-tree n32 target, but custom targets can use that ABI.
//@[n32] compile-flags: --target {{src-base}}/mips64-abi/mips64-n32.json -Z unstable-options
//@[n32] needs-llvm-components: mips
//@[mips64] compile-flags: --target mips64-unknown-linux-gnuabi64
//@[mips64] needs-llvm-components: mips

#![feature(no_core, lang_items)]
#![crate_type = "lib"]
#![no_core]

extern crate minicore;
use minicore::num::Complex;
use minicore::*;

// Larger than 16 bytes, so returned through a hidden pointer.
#[repr(C)]
pub struct Big {
    w: [u32; 6],
}

#[repr(C)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

// CHECK: define{{.*}} void @sret_then_i32(ptr {{.*}}sret([24 x i8]){{.*}} %_0, i32 noundef signext %k)
#[no_mangle]
pub extern "C" fn sret_then_i32(k: i32) -> Big {
    Big { w: [0; 6] }
}

// CHECK: define{{.*}} void @sret_then_aggregate(ptr {{.*}}sret([24 x i8]){{.*}} %_0, i64 %0, i32 noundef signext %k)
#[no_mangle]
pub extern "C" fn sret_then_aggregate(c: Color, k: i32) -> Big {
    Big { w: [0; 6] }
}

// Six integer arguments take slots 0-5, so `z` gets the FPRs of slots 6 and 7.
// CHECK: define{{.*}} void @complex_in_fprs(i64 noundef %a, i64 noundef %b, i64 noundef %c, i64 noundef %d, i64 noundef %e, i64 noundef %f, [2 x double] %0)
#[no_mangle]
pub extern "C" fn complex_in_fprs(a: i64, b: i64, c: i64, d: i64, e: i64, f: i64, z: Complex<f64>) {
}

// Only slot 7 is left, so `z` is passed like an integer, in a GPR and on the stack.
// CHECK: define{{.*}} void @complex_in_gprs(i64 noundef %a, i64 noundef %b, i64 noundef %c, i64 noundef %d, i64 noundef %e, i64 noundef %f, i64 noundef %g, [2 x i64] %0)
#[no_mangle]
pub extern "C" fn complex_in_gprs(
    a: i64,
    b: i64,
    c: i64,
    d: i64,
    e: i64,
    f: i64,
    g: i64,
    z: Complex<f64>,
) {
}

extern "C" {
    fn ext_big(k: i32) -> Big;
}

// CHECK-LABEL: @call_big
// CHECK: call void @ext_big(ptr {{.*}}sret([24 x i8]){{.*}}, i32 noundef signext %k)
#[no_mangle]
pub unsafe fn call_big(k: i32) -> Big {
    ext_big(k)
}
