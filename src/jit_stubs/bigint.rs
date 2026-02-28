//! BigInt JIT stubs for GMP integration
//!
//! These declarations link to the actual C GMP bridge functions in libviper.a
//! The registry maps LLVM function declarations to these external C functions.

use std::ffi::c_void;

// External declarations for C GMP bridge functions from libviper.a
extern "C" {
    pub fn vp_bigint_from_str(s: *const i8) -> *mut c_void;
    pub fn vp_bigint_from_i64(v: i64) -> *mut c_void;
    pub fn vp_bigint_from_i64_temp(v: i64) -> *mut c_void;
    pub fn vp_bigint_to_str(bigint: *mut c_void, base: i32) -> *const i8;
    pub fn vp_bigint_to_i64(bigint: *mut c_void) -> i64;
    pub fn vp_bigint_destroy(bigint: *mut c_void);
    
    // Arithmetic operations
    pub fn vp_bigint_add(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_sub(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_mul(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_div(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_mod(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_pow(result: *mut c_void, base: *mut c_void, exp: *mut c_void);
    pub fn vp_bigint_sqrt(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_abs(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_neg(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_invert(result: *mut c_void, a: *mut c_void);
    
    // Bitwise operations
    pub fn vp_bigint_and(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_or(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_xor(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_lshift(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_rshift(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    
    // Comparison operations
    pub fn vp_bigint_eq(a: *mut c_void, b: *mut c_void) -> bool;
    pub fn vp_bigint_lt(a: *mut c_void, b: *mut c_void) -> bool;
    pub fn vp_bigint_gt(a: *mut c_void, b: *mut c_void) -> bool;
    
    // Boolean checks
    pub fn vp_bigint_is_zero(bigint: *mut c_void) -> bool;
    pub fn vp_bigint_is_negative(bigint: *mut c_void) -> bool;
    pub fn vp_bigint_sign(bigint: *mut c_void) -> i64;
    pub fn vp_bigint_bit_length(bigint: *mut c_void) -> i64;
}
