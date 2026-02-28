//! BigInt JIT stubs for GMP integration

use std::ffi::CStr;
use std::os::raw::c_char;

// External GMP runtime functions (from libviper.a)
extern "C" {
    fn vp_bigint_from_str(s: *const c_char) -> *mut std::ffi::c_void;
    fn vp_bigint_from_i64(v: i64) -> *mut std::ffi::c_void;
    fn vp_bigint_to_str(bigint: *mut std::ffi::c_void, base: i64) -> *mut c_char;
    fn vp_bigint_to_i64(bigint: *mut std::ffi::c_void) -> i64;
    fn vp_bigint_add(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_sub(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_mul(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_div(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_mod(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_pow(result: *mut std::ffi::c_void, base: *mut std::ffi::c_void, exp: *mut std::ffi::c_void);
    fn vp_bigint_sqrt(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void);
    fn vp_bigint_abs(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void);
    fn vp_bigint_and(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_or(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_xor(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_lshift(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_rshift(result: *mut std::ffi::c_void, a: *mut std::ffi::c_void, b: *mut std::ffi::c_void);
    fn vp_bigint_eq(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool;
    fn vp_bigint_lt(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool;
    fn vp_bigint_gt(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool;
    fn vp_bigint_destroy(bigint: *mut std::ffi::c_void);
}

/// Stub for vp_bigint_from_str - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_from_str_stub(s: *const c_char) -> *mut std::ffi::c_void {
    unsafe { vp_bigint_from_str(s) }
}

/// Stub for vp_bigint_from_i64 - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_from_i64_stub(v: i64) -> *mut std::ffi::c_void {
    unsafe { vp_bigint_from_i64(v) }
}

/// Stub for vp_bigint_to_str - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_to_str_stub(bigint: *mut std::ffi::c_void, base: i64) -> *mut c_char {
    unsafe { vp_bigint_to_str(bigint, base) }
}

/// Stub for vp_bigint_to_i64 - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_to_i64_stub(bigint: *mut std::ffi::c_void) -> i64 {
    unsafe { vp_bigint_to_i64(bigint) }
}

/// Stub for vp_bigint_add - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_add_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_add(result, a, b) }
}

/// Stub for vp_bigint_sub - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_sub_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_sub(result, a, b) }
}

/// Stub for vp_bigint_mul - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_mul_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_mul(result, a, b) }
}

/// Stub for vp_bigint_div - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_div_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_div(result, a, b) }
}

/// Stub for vp_bigint_mod - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_mod_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_mod(result, a, b) }
}

/// Stub for vp_bigint_pow - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_pow_stub(
    result: *mut std::ffi::c_void,
    base: *mut std::ffi::c_void,
    exp: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_pow(result, base, exp) }
}

/// Stub for vp_bigint_sqrt - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_sqrt_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_sqrt(result, a) }
}

/// Stub for vp_bigint_abs - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_abs_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_abs(result, a) }
}

/// Stub for vp_bigint_and - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_and_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_and(result, a, b) }
}

/// Stub for vp_bigint_or - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_or_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_or(result, a, b) }
}

/// Stub for vp_bigint_xor - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_xor_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_xor(result, a, b) }
}

/// Stub for vp_bigint_lshift - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_lshift_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_lshift(result, a, b) }
}

/// Stub for vp_bigint_rshift - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_rshift_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe { vp_bigint_rshift(result, a, b) }
}

/// Stub for vp_bigint_eq - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_eq_stub(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool {
    unsafe { vp_bigint_eq(a, b) }
}

/// Stub for vp_bigint_lt - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_lt_stub(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool {
    unsafe { vp_bigint_lt(a, b) }
}

/// Stub for vp_bigint_gt - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_gt_stub(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool {
    unsafe { vp_bigint_gt(a, b) }
}

/// Stub for vp_bigint_destroy - directly calls runtime
#[no_mangle]
pub extern "C" fn vp_bigint_destroy_stub(bigint: *mut std::ffi::c_void) {
    unsafe { vp_bigint_destroy(bigint) }
}
