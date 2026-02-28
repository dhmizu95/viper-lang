//! BigInt JIT stubs for GMP integration
//!
//! These stubs delegate to the C bridge implementation (gmp_bridge.c)
//! which handles all BigInt operations using GMP and ARC memory management.

use std::ffi::{c_char, c_int, c_void};

// TEMPORARY STUBS - Replace with actual C bridge calls when linking is fixed
// These stubs provide minimal functionality to allow compilation

#[no_mangle]
pub extern "C" fn vp_bigint_from_str_stub(_s: *const c_char) -> *mut c_void {
    eprintln!("BigInt not available - stub called");
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_bigint_from_i64_stub(_v: i64) -> *mut c_void {
    eprintln!("BigInt not available - stub called");
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_bigint_to_str_stub(_bigint: *mut c_void, _base: i64) -> *const c_char {
    eprintln!("BigInt not available - stub called");
    std::ptr::null()
}

#[no_mangle]
pub extern "C" fn vp_bigint_to_i64_stub(_bigint: *mut c_void) -> i64 {
    eprintln!("BigInt not available - stub called");
    0
}

#[no_mangle]
pub extern "C" fn vp_bigint_add_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_sub_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_mul_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_div_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_mod_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_pow_stub(
    _result: *mut c_void,
    _base: *mut c_void,
    _exp: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_sqrt_stub(
    _result: *mut c_void,
    _a: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_abs_stub(
    _result: *mut c_void,
    _a: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_neg_stub(
    _result: *mut c_void,
    _a: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_invert_stub(
    _result: *mut c_void,
    _a: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_is_zero_stub(_bigint: *mut c_void) -> bool {
    eprintln!("BigInt not available - stub called");
    false
}

#[no_mangle]
pub extern "C" fn vp_bigint_is_negative_stub(_bigint: *mut c_void) -> bool {
    eprintln!("BigInt not available - stub called");
    false
}

#[no_mangle]
pub extern "C" fn vp_bigint_sign_stub(_bigint: *mut c_void) -> i64 {
    eprintln!("BigInt not available - stub called");
    0
}

#[no_mangle]
pub extern "C" fn vp_bigint_bit_length_stub(_bigint: *mut c_void) -> i64 {
    eprintln!("BigInt not available - stub called");
    0
}

#[no_mangle]
pub extern "C" fn vp_bigint_and_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_or_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_xor_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_lshift_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_rshift_stub(
    _result: *mut c_void,
    _a: *mut c_void,
    _b: *mut c_void,
) {
    eprintln!("BigInt not available - stub called");
}

#[no_mangle]
pub extern "C" fn vp_bigint_eq_stub(_a: *mut c_void, _b: *mut c_void) -> bool {
    eprintln!("BigInt not available - stub called");
    false
}

#[no_mangle]
pub extern "C" fn vp_bigint_lt_stub(_a: *mut c_void, _b: *mut c_void) -> bool {
    eprintln!("BigInt not available - stub called");
    false
}

#[no_mangle]
pub extern "C" fn vp_bigint_gt_stub(_a: *mut c_void, _b: *mut c_void) -> bool {
    eprintln!("BigInt not available - stub called");
    false
}

#[no_mangle]
pub extern "C" fn vp_bigint_destroy_stub(_bigint: *mut c_void) {
    eprintln!("BigInt not available - stub called");
}
