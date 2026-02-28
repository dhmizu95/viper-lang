//! BigInt JIT stubs for GMP integration
//! 
//! These stubs delegate to the C bridge implementation (gmp_bridge.c)
//! which handles all BigInt operations using GMP and ARC memory management.

use std::ffi::{c_char, c_int, c_void};

// Import C bridge functions from libviper.a
extern "C" {
    // Core BigInt operations
    fn vp_bigint_from_str(str: *const c_char) -> *mut c_void;
    fn vp_bigint_from_i64(value: i64) -> *mut c_void;
    fn vp_bigint_to_str(bigint: *mut c_void, base: c_int) -> *mut c_char;
    fn vp_bigint_to_i64(bigint: *mut c_void) -> i64;
    fn vp_bigint_destroy(bigint: *mut c_void);
    
    // Arithmetic operations (result must be an initialized BigInt)
    fn vp_bigint_add(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_sub(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_mul(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_div(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_mod(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_pow(result: *mut c_void, base: *mut c_void, exp: *mut c_void);
    fn vp_bigint_sqrt(result: *mut c_void, a: *mut c_void);
    
    // Unary operations
    fn vp_bigint_abs(result: *mut c_void, operand: *mut c_void);
    fn vp_bigint_neg(result: *mut c_void, operand: *mut c_void);
    fn vp_bigint_invert(result: *mut c_void, operand: *mut c_void);
    
    // Bitwise operations
    fn vp_bigint_and(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_or(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_xor(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_lshift(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    fn vp_bigint_rshift(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    
    // Comparison operations
    fn vp_bigint_eq(a: *mut c_void, b: *mut c_void) -> bool;
    fn vp_bigint_lt(a: *mut c_void, b: *mut c_void) -> bool;
    fn vp_bigint_gt(a: *mut c_void, b: *mut c_void) -> bool;
    fn vp_bigint_is_zero(a: *mut c_void) -> bool;
    fn vp_bigint_is_negative(a: *mut c_void) -> bool;
    fn vp_bigint_sign(a: *mut c_void) -> i64;
    fn vp_bigint_bit_length(a: *mut c_void) -> i64;
}

/// Stub for vp_bigint_from_str - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_from_str_stub(s: *const c_char) -> *mut c_void {
    unsafe { vp_bigint_from_str(s) }
}

/// Stub for vp_bigint_from_i64 - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_from_i64_stub(v: i64) -> *mut c_void {
    unsafe { vp_bigint_from_i64(v) }
}

/// Stub for vp_bigint_to_str - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_to_str_stub(bigint: *mut c_void, base: i64) -> *const c_char {
    unsafe { vp_bigint_to_str(bigint, base as c_int) }
}

/// Stub for vp_bigint_to_i64 - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_to_i64_stub(bigint: *mut c_void) -> i64 {
    unsafe { vp_bigint_to_i64(bigint) }
}

/// Stub for vp_bigint_add - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_add_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_add(result, a, b); }
}

/// Stub for vp_bigint_sub - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_sub_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_sub(result, a, b); }
}

/// Stub for vp_bigint_mul - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_mul_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_mul(result, a, b); }
}

/// Stub for vp_bigint_div - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_div_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_div(result, a, b); }
}

/// Stub for vp_bigint_mod - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_mod_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_mod(result, a, b); }
}

/// Stub for vp_bigint_pow - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_pow_stub(
    result: *mut c_void,
    base: *mut c_void,
    exp: *mut c_void,
) {
    unsafe { vp_bigint_pow(result, base, exp); }
}

/// Stub for vp_bigint_sqrt - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_sqrt_stub(
    result: *mut c_void,
    a: *mut c_void,
) {
    unsafe { vp_bigint_sqrt(result, a); }
}

/// Stub for vp_bigint_abs - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_abs_stub(
    result: *mut c_void,
    a: *mut c_void,
) {
    unsafe { vp_bigint_abs(result, a); }
}

/// Stub for vp_bigint_neg - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_neg_stub(
    result: *mut c_void,
    a: *mut c_void,
) {
    unsafe { vp_bigint_neg(result, a); }
}

/// Stub for vp_bigint_invert - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_invert_stub(
    result: *mut c_void,
    a: *mut c_void,
) {
    unsafe { vp_bigint_invert(result, a); }
}

/// Stub for vp_bigint_is_zero - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_is_zero_stub(bigint: *mut c_void) -> bool {
    unsafe { vp_bigint_is_zero(bigint) }
}

/// Stub for vp_bigint_is_negative - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_is_negative_stub(bigint: *mut c_void) -> bool {
    unsafe { vp_bigint_is_negative(bigint) }
}

/// Stub for vp_bigint_sign - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_sign_stub(bigint: *mut c_void) -> i64 {
    unsafe { vp_bigint_sign(bigint) }
}

/// Stub for vp_bigint_bit_length - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_bit_length_stub(bigint: *mut c_void) -> i64 {
    unsafe { vp_bigint_bit_length(bigint) }
}

/// Stub for vp_bigint_and - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_and_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_and(result, a, b); }
}

/// Stub for vp_bigint_or - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_or_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_or(result, a, b); }
}

/// Stub for vp_bigint_xor - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_xor_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_xor(result, a, b); }
}

/// Stub for vp_bigint_lshift - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_lshift_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_lshift(result, a, b); }
}

/// Stub for vp_bigint_rshift - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_rshift_stub(
    result: *mut c_void,
    a: *mut c_void,
    b: *mut c_void,
) {
    unsafe { vp_bigint_rshift(result, a, b); }
}

/// Stub for vp_bigint_eq - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_eq_stub(a: *mut c_void, b: *mut c_void) -> bool {
    unsafe { vp_bigint_eq(a, b) }
}

/// Stub for vp_bigint_lt - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_lt_stub(a: *mut c_void, b: *mut c_void) -> bool {
    unsafe { vp_bigint_lt(a, b) }
}

/// Stub for vp_bigint_gt - delegates to C bridge
#[no_mangle]
pub extern "C" fn vp_bigint_gt_stub(a: *mut c_void, b: *mut c_void) -> bool {
    unsafe { vp_bigint_gt(a, b) }
}

/// Stub for vp_bigint_destroy - delegates to C bridge (via ARC)
#[no_mangle]
pub extern "C" fn vp_bigint_destroy_stub(bigint: *mut c_void) {
    unsafe { vp_bigint_destroy(bigint); }
}
