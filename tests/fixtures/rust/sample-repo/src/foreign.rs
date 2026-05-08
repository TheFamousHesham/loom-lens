// Foreign-effect patterns: unsafe, FFI, raw pointer manipulation.

use std::ffi::{c_char, c_int};

extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn abs(value: c_int) -> c_int;
}

pub fn ffi_strlen(s: *const c_char) -> usize {
    // expect: Foreign=definite
    unsafe { strlen(s) }
}

pub fn ffi_abs(v: c_int) -> c_int {
    // expect: Foreign=definite
    unsafe { abs(v) }
}

pub unsafe fn marked_unsafe_fn(p: *const u8) -> u8 {
    // expect: Foreign=definite
    unsafe { *p }
}

pub fn raw_ptr_read(p: *const u8) -> u8 {
    // expect: Foreign=definite
    unsafe { *p }
}

pub fn transmute_things(x: i32) -> u32 {
    // expect: Foreign=definite
    unsafe { std::mem::transmute(x) }
}
