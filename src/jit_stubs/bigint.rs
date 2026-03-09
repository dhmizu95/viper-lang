//! BigInt JIT stubs for GMP integration
//!
//! These declarations link to the actual C GMP bridge functions in libviper.a
//! The registry maps LLVM function declarations to these external C functions.

use std::ffi::c_void;

// External declarations for C GMP bridge functions from libviper.a (with _c suffix)
extern "C" {
    pub fn vp_bigint_from_str_c(s: *const i8) -> *mut c_void;
    pub fn vp_bigint_from_i64_c(v: i64) -> *mut c_void;
    pub fn vp_bigint_from_i64_temp(v: i64) -> *mut c_void;
    pub fn vp_bigint_to_str_c(bigint: *mut c_void, base: i32) -> *const i8;
    pub fn vp_bigint_to_i64(bigint: *mut c_void) -> i64;
    pub fn vp_bigint_destroy(bigint: *mut c_void);

    // Arithmetic operations
    pub fn vp_bigint_add_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_sub_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_mul_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_div_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_mod_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_pow_c(result: *mut c_void, base: *mut c_void, exp: *mut c_void);
    pub fn vp_bigint_powmod_c(result: *mut c_void, base: *mut c_void, exp: *mut c_void, mod_val: *mut c_void);
    pub fn vp_bigint_divmod_c(quotient: *mut c_void, remainder: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_sqrt_c(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_abs_c(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_neg_c(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_invert_c(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_min_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_max_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_gcd_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_lcm_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_factorial_c(result: *mut c_void, n: *mut c_void);
    pub fn vp_bigint_comb_c(result: *mut c_void, n: *mut c_void, k: *mut c_void);
    pub fn vp_bigint_perm_c(result: *mut c_void, n: *mut c_void, k: *mut c_void);


    // Bitwise operations
    pub fn vp_bigint_and_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_or_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_xor_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_lshift_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_rshift_c(result: *mut c_void, a: *mut c_void, b: *mut c_void);

    // Comparison operations
    pub fn vp_bigint_eq_c(a: *mut c_void, b: *mut c_void) -> bool;
    pub fn vp_bigint_lt_c(a: *mut c_void, b: *mut c_void) -> bool;
    pub fn vp_bigint_gt_c(a: *mut c_void, b: *mut c_void) -> bool;
    pub fn vp_bigint_cmp_c(a: *mut c_void, b: *mut c_void) -> i32;

    // Boolean checks
    pub fn vp_bigint_is_zero(bigint: *mut c_void) -> bool;
    pub fn vp_bigint_is_negative(bigint: *mut c_void) -> bool;
    pub fn vp_bigint_sign(bigint: *mut c_void) -> i64;
    pub fn vp_bigint_bit_length(bigint: *mut c_void) -> i64;
}

// JIT stub wrappers - these call the C functions
pub extern "C" fn vp_bigint_from_i64_stub(v: i64) -> *mut c_void {
    let result = unsafe { vp_bigint_from_i64_c(v) };
    result
}
pub extern "C" fn vp_bigint_from_i64_temp_stub(v: i64) -> *mut c_void {
    let result = unsafe { vp_bigint_from_i64_temp(v) };
    result
}
pub extern "C" fn vp_bigint_from_str_stub(s: *const i8) -> *mut c_void {
    unsafe { vp_bigint_from_str_c(s) }
}
pub extern "C" fn vp_bigint_add_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    if result.is_null() || a.is_null() || b.is_null() {
        return;
    }
    unsafe { vp_bigint_add_c(result, a, b) }
}
pub extern "C" fn vp_bigint_sub_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_sub_c(result, a, b) }
}
pub extern "C" fn vp_bigint_mul_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_mul_c(result, a, b) }
}
pub extern "C" fn vp_bigint_div_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    if result.is_null() || a.is_null() || b.is_null() {
        return;
    }
    unsafe { vp_bigint_div_c(result, a, b) }
}
pub extern "C" fn vp_bigint_mod_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_mod_c(result, a, b) }
}
pub extern "C" fn vp_bigint_pow_stub(result: *mut c_void, base: *mut c_void, exp: *mut c_void) {
    unsafe { vp_bigint_pow_c(result, base, exp) }
}
pub extern "C" fn vp_bigint_powmod_stub(result: *mut c_void, base: *mut c_void, exp: *mut c_void, mod_val: *mut c_void) {
    unsafe { vp_bigint_powmod_c(result, base, exp, mod_val) }
}
pub extern "C" fn vp_bigint_divmod_stub(quotient: *mut c_void, remainder: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_divmod_c(quotient, remainder, a, b) }
}
pub extern "C" fn vp_bigint_sqrt_stub(result: *mut c_void, a: *mut c_void) {
    unsafe { vp_bigint_sqrt_c(result, a) }
}
pub extern "C" fn vp_bigint_neg_stub(result: *mut c_void, a: *mut c_void) {
    unsafe { vp_bigint_neg_c(result, a) }
}
pub extern "C" fn vp_bigint_abs_stub(result: *mut c_void, a: *mut c_void) {
    unsafe { vp_bigint_abs_c(result, a) }
}
pub extern "C" fn vp_bigint_min_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_min_c(result, a, b) }
}
pub extern "C" fn vp_bigint_max_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_max_c(result, a, b) }
}
pub extern "C" fn vp_bigint_gcd_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_gcd_c(result, a, b) }
}
pub extern "C" fn vp_bigint_lcm_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_lcm_c(result, a, b) }
}
pub extern "C" fn vp_bigint_factorial_stub(result: *mut c_void, n: *mut c_void) {
    unsafe { vp_bigint_factorial_c(result, n) }
}
pub extern "C" fn vp_bigint_comb_stub(result: *mut c_void, n: *mut c_void, k: *mut c_void) {
    unsafe { vp_bigint_comb_c(result, n, k) }
}
pub extern "C" fn vp_bigint_perm_stub(result: *mut c_void, n: *mut c_void, k: *mut c_void) {
    unsafe { vp_bigint_perm_c(result, n, k) }
}
pub extern "C" fn vp_bigint_cmp_stub(a: *mut c_void, b: *mut c_void) -> i32 {
    // Use the C library comparison function for accurate comparison
    // This handles arbitrary precision integers correctly
    extern "C" {
        fn vp_bigint_cmp_c(a: *mut c_void, b: *mut c_void) -> i32;
    }
    unsafe { vp_bigint_cmp_c(a, b) }
}
pub extern "C" fn vp_bigint_eq_stub(a: *mut c_void, b: *mut c_void) -> bool {
    unsafe { vp_bigint_eq_c(a, b) }
}
pub extern "C" fn vp_bigint_lt_stub(a: *mut c_void, b: *mut c_void) -> bool {
    unsafe { vp_bigint_lt_c(a, b) }
}
pub extern "C" fn vp_bigint_le_stub(_a: *mut c_void, _b: *mut c_void) -> bool {
    // Placeholder - not fully implemented
    false
}
pub extern "C" fn vp_bigint_gt_stub(a: *mut c_void, b: *mut c_void) -> bool {
    unsafe { vp_bigint_gt_c(a, b) }
}
pub extern "C" fn vp_bigint_ge_stub(_a: *mut c_void, _b: *mut c_void) -> bool {
    // Placeholder - not fully implemented
    false
}
pub extern "C" fn vp_bigint_to_str_stub(bigint: *mut c_void, base: i32) -> *const i8 {
    unsafe { vp_bigint_to_str_c(bigint, base) }
}

pub extern "C" fn vp_bigint_and_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_and_c(result, a, b) }
}

pub extern "C" fn vp_bigint_or_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_or_c(result, a, b) }
}

pub extern "C" fn vp_bigint_xor_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_xor_c(result, a, b) }
}

pub extern "C" fn vp_bigint_lshift_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_lshift_c(result, a, b) }
}

pub extern "C" fn vp_bigint_rshift_stub(result: *mut c_void, a: *mut c_void, b: *mut c_void) {
    unsafe { vp_bigint_rshift_c(result, a, b) }
}

pub extern "C" fn vp_bigint_invert_stub(result: *mut c_void, a: *mut c_void) {
    unsafe { vp_bigint_invert_c(result, a) }
}

pub extern "C" fn vp_bigint_to_i64_stub(bigint: *mut c_void) -> i64 {
    unsafe { vp_bigint_to_i64(bigint) }
}

pub extern "C" fn vp_bigint_free_stub(_bigint: *mut c_void) {
    // Placeholder - memory managed by ARC
}

// Re-export original names (without _c suffix) for non-JIT AOT usage
extern "C" {
    pub fn vp_bigint_from_str(s: *const i8) -> *mut c_void;
    pub fn vp_bigint_from_i64(v: i64) -> *mut c_void;
    pub fn vp_bigint_to_str(bigint: *mut c_void, base: i32) -> *const i8;

    // Arithmetic operations
    pub fn vp_bigint_add(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_sub(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_mul(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_div(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_mod(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_pow(result: *mut c_void, base: *mut c_void, exp: *mut c_void);
    pub fn vp_bigint_powmod(result: *mut c_void, base: *mut c_void, exp: *mut c_void, mod_val: *mut c_void);
    pub fn vp_bigint_divmod(quotient: *mut c_void, remainder: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_sqrt(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_abs(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_neg(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_invert(result: *mut c_void, a: *mut c_void);
    pub fn vp_bigint_min(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_max(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_gcd(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_lcm(result: *mut c_void, a: *mut c_void, b: *mut c_void);
    pub fn vp_bigint_factorial(result: *mut c_void, n: *mut c_void);
    pub fn vp_bigint_comb(result: *mut c_void, n: *mut c_void, k: *mut c_void);
    pub fn vp_bigint_perm(result: *mut c_void, n: *mut c_void, k: *mut c_void);

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
}
