/// BigInt JIT stubs for Viper
/// Delegates to the C runtime library via dlopen/dlsym or direct linking

use std::os::raw::{c_char, c_int};

// VpBigInt structure (must match runtime/bigint.h)
#[repr(C)]
pub struct VpBigInt {
    pub sign: c_int,
    pub len: usize,
    pub cap: usize,
    pub digits: *mut u32,
}

// External C functions from libviper.a (linked at compile time)
extern "C" {
    fn vp_bigint_from_i64(v: i64) -> *mut VpBigInt;
    fn vp_bigint_from_str(s: *const c_char) -> *mut VpBigInt;
    fn vp_bigint_add(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_sub(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_mul(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_div(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_mod(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_pow(base: *mut VpBigInt, exp: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_neg(a: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_abs(a: *mut VpBigInt) -> *mut VpBigInt;
    fn vp_bigint_cmp(a: *mut VpBigInt, b: *mut VpBigInt) -> c_int;
    fn vp_bigint_eq(a: *mut VpBigInt, b: *mut VpBigInt) -> bool;
    fn vp_bigint_lt(a: *mut VpBigInt, b: *mut VpBigInt) -> bool;
    fn vp_bigint_le(a: *mut VpBigInt, b: *mut VpBigInt) -> bool;
    fn vp_bigint_gt(a: *mut VpBigInt, b: *mut VpBigInt) -> bool;
    fn vp_bigint_ge(a: *mut VpBigInt, b: *mut VpBigInt) -> bool;
    fn vp_bigint_to_str(a: *mut VpBigInt) -> *mut c_char;
    fn vp_bigint_free(a: *mut VpBigInt);
}

/// JIT stub: vp_bigint_from_i64
#[no_mangle]
pub extern "C" fn vp_bigint_from_i64_stub(v: i64) -> *mut VpBigInt {
    unsafe { vp_bigint_from_i64(v) }
}

/// JIT stub: vp_bigint_from_str
#[no_mangle]
pub extern "C" fn vp_bigint_from_str_stub(s: *const c_char) -> *mut VpBigInt {
    unsafe { vp_bigint_from_str(s) }
}

/// JIT stub: vp_bigint_add
#[no_mangle]
pub extern "C" fn vp_bigint_add_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_add(a, b) }
}

/// JIT stub: vp_bigint_sub
#[no_mangle]
pub extern "C" fn vp_bigint_sub_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_sub(a, b) }
}

/// JIT stub: vp_bigint_mul
#[no_mangle]
pub extern "C" fn vp_bigint_mul_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_mul(a, b) }
}

/// JIT stub: vp_bigint_div
#[no_mangle]
pub extern "C" fn vp_bigint_div_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_div(a, b) }
}

/// JIT stub: vp_bigint_mod
#[no_mangle]
pub extern "C" fn vp_bigint_mod_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_mod(a, b) }
}

/// JIT stub: vp_bigint_pow
#[no_mangle]
pub extern "C" fn vp_bigint_pow_stub(base: *mut VpBigInt, exp: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_pow(base, exp) }
}

/// JIT stub: vp_bigint_neg
#[no_mangle]
pub extern "C" fn vp_bigint_neg_stub(a: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_neg(a) }
}

/// JIT stub: vp_bigint_abs
#[no_mangle]
pub extern "C" fn vp_bigint_abs_stub(a: *mut VpBigInt) -> *mut VpBigInt {
    unsafe { vp_bigint_abs(a) }
}

/// JIT stub: vp_bigint_cmp
#[no_mangle]
pub extern "C" fn vp_bigint_cmp_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> c_int {
    unsafe { vp_bigint_cmp(a, b) }
}

/// JIT stub: vp_bigint_eq
#[no_mangle]
pub extern "C" fn vp_bigint_eq_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> bool {
    unsafe { vp_bigint_eq(a, b) }
}

/// JIT stub: vp_bigint_lt
#[no_mangle]
pub extern "C" fn vp_bigint_lt_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> bool {
    unsafe { vp_bigint_lt(a, b) }
}

/// JIT stub: vp_bigint_le
#[no_mangle]
pub extern "C" fn vp_bigint_le_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> bool {
    unsafe { vp_bigint_le(a, b) }
}

/// JIT stub: vp_bigint_gt
#[no_mangle]
pub extern "C" fn vp_bigint_gt_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> bool {
    unsafe { vp_bigint_gt(a, b) }
}

/// JIT stub: vp_bigint_ge
#[no_mangle]
pub extern "C" fn vp_bigint_ge_stub(a: *mut VpBigInt, b: *mut VpBigInt) -> bool {
    unsafe { vp_bigint_ge(a, b) }
}

/// JIT stub: vp_bigint_to_str
#[no_mangle]
pub extern "C" fn vp_bigint_to_str_stub(a: *mut VpBigInt) -> *const c_char {
    unsafe { vp_bigint_to_str(a) }
}

/// JIT stub: vp_bigint_free
#[no_mangle]
pub extern "C" fn vp_bigint_free_stub(a: *mut VpBigInt) {
    unsafe { vp_bigint_free(a) }
}
