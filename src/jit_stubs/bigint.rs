//! BigInt JIT stubs for GMP integration

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// External GMP library functions (with __gmpz_ prefix)
extern "C" {
    // GMP mpz functions
    fn __gmpz_init_set_str(rop: *mut MpzStruct, str: *const c_char, base: i32) -> i32;
    fn __gmpz_init_set_si(rop: *mut MpzStruct, val: i64);
    fn __gmpz_init(rop: *mut MpzStruct);
    fn __gmpz_clear(rop: *mut MpzStruct);
    fn __gmpz_get_str(str: *mut c_char, base: i32, op: *const MpzStruct) -> *mut c_char;
    fn __gmpz_get_si(op: *const MpzStruct) -> i64;
    fn __gmpz_add(rop: *mut MpzStruct, op1: *const MpzStruct, op2: *const MpzStruct);
    fn __gmpz_sub(rop: *mut MpzStruct, op1: *const MpzStruct, op2: *const MpzStruct);
    fn __gmpz_mul(rop: *mut MpzStruct, op1: *const MpzStruct, op2: *const MpzStruct);
    fn __gmpz_tdiv_q(rop: *mut MpzStruct, n: *const MpzStruct, d: *const MpzStruct);
    fn __gmpz_tdiv_r(rop: *mut MpzStruct, n: *const MpzStruct, d: *const MpzStruct);
    fn __gmpz_pow_ui(rop: *mut MpzStruct, base: *const MpzStruct, exp: u64);
    fn __gmpz_sqrt(rop: *mut MpzStruct, op: *const MpzStruct);
    fn __gmpz_abs(rop: *mut MpzStruct, op: *const MpzStruct);
    fn __gmpz_and(rop: *mut MpzStruct, op1: *const MpzStruct, op2: *const MpzStruct);
    fn __gmpz_ior(rop: *mut MpzStruct, op1: *const MpzStruct, op2: *const MpzStruct);
    fn __gmpz_xor(rop: *mut MpzStruct, op1: *const MpzStruct, op2: *const MpzStruct);
    fn __gmpz_mul_2exp(rop: *mut MpzStruct, op1: *const MpzStruct, cnt: u64);
    fn __gmpz_tdiv_q_2exp(rop: *mut MpzStruct, op1: *const MpzStruct, cnt: u64);
    fn __gmpz_cmp(op1: *const MpzStruct, op2: *const MpzStruct) -> i32;
    fn __gmpz_cmp_si(op1: *const MpzStruct, op2: i64) -> i32;
}

// GMP mpz_t structure (simplified - actual layout may vary)
#[repr(C)]
struct MpzStruct {
    _mp_alloc: i32,
    _mp_size: i32,
    _mp_d: *mut u64,
}

// ViperBigInt structure matching the C runtime
#[repr(C)]
struct ViperBigInt {
    ref_count: i64,
    destructor: *const (),
    flags: u8,
    reserved: [u8; 7],
    mpz: MpzStruct,
}

/// Create a Viper string from a C string (minimal implementation for BigInt)
/// This allocates memory that will be managed by the Viper ARC runtime
#[no_mangle]
pub extern "C" fn vp_str_create(s: *const c_char) -> *mut std::ffi::c_void {
    unsafe {
        if s.is_null() {
            return std::ptr::null_mut();
        }
        let c_str = CStr::from_ptr(s);
        let bytes = c_str.to_bytes_with_nul();
        let ptr = libc::malloc(bytes.len()) as *mut c_char;
        if ptr.is_null() {
            return std::ptr::null_mut();
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
        ptr as *mut std::ffi::c_void
    }
}

/// Allocate a new BigInt
fn alloc_bigint() -> *mut ViperBigInt {
    unsafe {
        let bigint = libc::malloc(std::mem::size_of::<ViperBigInt>()) as *mut ViperBigInt;
        if bigint.is_null() {
            panic!("Failed to allocate BigInt");
        }
        (*bigint).ref_count = 1;
        (*bigint).destructor = vp_bigint_destroy_stub as *const ();
        (*bigint).flags = 0;
        (*bigint).reserved = [0; 7];
        __gmpz_init(&mut (*bigint).mpz);
        bigint
    }
}

/// Stub for vp_bigint_from_str
#[no_mangle]
pub extern "C" fn vp_bigint_from_str_stub(s: *const c_char) -> *mut std::ffi::c_void {
    unsafe {
        let bigint = alloc_bigint();
        if __gmpz_init_set_str(&mut (*bigint).mpz, s, 0) != 0 {
            libc::free(bigint as *mut libc::c_void);
            return std::ptr::null_mut();
        }
        bigint as *mut std::ffi::c_void
    }
}

/// Stub for vp_bigint_from_i64
#[no_mangle]
pub extern "C" fn vp_bigint_from_i64_stub(v: i64) -> *mut std::ffi::c_void {
    unsafe {
        let bigint = alloc_bigint();
        __gmpz_init_set_si(&mut (*bigint).mpz, v);
        bigint as *mut std::ffi::c_void
    }
}

/// Stub for vp_bigint_to_str - converts BigInt to ViperString
#[no_mangle]
pub extern "C" fn vp_bigint_to_str_stub(bigint: *mut std::ffi::c_void, base: i64) -> *mut std::ffi::c_void {
    unsafe {
        if bigint.is_null() {
            return vp_str_create(CString::new("").unwrap().as_ptr());
        }
        let c_str = __gmpz_get_str(std::ptr::null_mut(), base as i32, &(*bigint.cast::<ViperBigInt>()).mpz);
        if c_str.is_null() {
            return vp_str_create(CString::new("").unwrap().as_ptr());
        }
        let viper_str = vp_str_create(c_str);
        libc::free(c_str as *mut libc::c_void);
        viper_str
    }
}

/// Stub for vp_bigint_to_i64
#[no_mangle]
pub extern "C" fn vp_bigint_to_i64_stub(bigint: *mut std::ffi::c_void) -> i64 {
    unsafe {
        if bigint.is_null() {
            return 0;
        }
        __gmpz_get_si(&(*bigint.cast::<ViperBigInt>()).mpz)
    }
}

/// Stub for vp_bigint_add
#[no_mangle]
pub extern "C" fn vp_bigint_add_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        __gmpz_add(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_sub
#[no_mangle]
pub extern "C" fn vp_bigint_sub_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        __gmpz_sub(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_mul
#[no_mangle]
pub extern "C" fn vp_bigint_mul_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        __gmpz_mul(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_div
#[no_mangle]
pub extern "C" fn vp_bigint_div_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        if __gmpz_cmp_si(&(*b.cast::<ViperBigInt>()).mpz, 0) == 0 {
            eprintln!("Error: Division by zero in BigInt division");
            return;
        }
        __gmpz_tdiv_q(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_mod
#[no_mangle]
pub extern "C" fn vp_bigint_mod_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        if __gmpz_cmp_si(&(*b.cast::<ViperBigInt>()).mpz, 0) == 0 {
            eprintln!("Error: Modulo by zero in BigInt operation");
            return;
        }
        __gmpz_tdiv_r(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_pow
#[no_mangle]
pub extern "C" fn vp_bigint_pow_stub(
    result: *mut std::ffi::c_void,
    base: *mut std::ffi::c_void,
    exp: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || base.is_null() || exp.is_null() {
            return;
        }
        if __gmpz_cmp_si(&(*exp.cast::<ViperBigInt>()).mpz, 0) < 0 {
            eprintln!("Error: Negative exponent in BigInt power");
            return;
        }
        // For simplicity, only handle small exponents
        let exp_val = __gmpz_get_si(&(*exp.cast::<ViperBigInt>()).mpz);
        if exp_val < 0 || exp_val > 1000000 {
            eprintln!("Error: Exponent too large");
            return;
        }
        __gmpz_pow_ui(&mut (*result.cast::<ViperBigInt>()).mpz, &(*base.cast::<ViperBigInt>()).mpz, exp_val as u64);
    }
}

/// Stub for vp_bigint_sqrt
#[no_mangle]
pub extern "C" fn vp_bigint_sqrt_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() {
            return;
        }
        if __gmpz_cmp_si(&(*a.cast::<ViperBigInt>()).mpz, 0) < 0 {
            eprintln!("Error: Square root of negative number");
            return;
        }
        __gmpz_sqrt(&mut (*result.cast::<ViperBigInt>()).mpz, &(*a.cast::<ViperBigInt>()).mpz);
    }
}

/// Stub for vp_bigint_abs
#[no_mangle]
pub extern "C" fn vp_bigint_abs_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() {
            return;
        }
        __gmpz_abs(&mut (*result.cast::<ViperBigInt>()).mpz, &(*a.cast::<ViperBigInt>()).mpz);
    }
}

/// Stub for vp_bigint_and
#[no_mangle]
pub extern "C" fn vp_bigint_and_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        __gmpz_and(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_or
#[no_mangle]
pub extern "C" fn vp_bigint_or_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        __gmpz_ior(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_xor
#[no_mangle]
pub extern "C" fn vp_bigint_xor_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        __gmpz_xor(
            &mut (*result.cast::<ViperBigInt>()).mpz,
            &(*a.cast::<ViperBigInt>()).mpz,
            &(*b.cast::<ViperBigInt>()).mpz,
        );
    }
}

/// Stub for vp_bigint_lshift
#[no_mangle]
pub extern "C" fn vp_bigint_lshift_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        let shift = __gmpz_get_si(&(*b.cast::<ViperBigInt>()).mpz);
        if shift < 0 {
            eprintln!("Error: Negative shift in BigInt lshift");
            return;
        }
        __gmpz_mul_2exp(&mut (*result.cast::<ViperBigInt>()).mpz, &(*a.cast::<ViperBigInt>()).mpz, shift as u64);
    }
}

/// Stub for vp_bigint_rshift
#[no_mangle]
pub extern "C" fn vp_bigint_rshift_stub(
    result: *mut std::ffi::c_void,
    a: *mut std::ffi::c_void,
    b: *mut std::ffi::c_void,
) {
    unsafe {
        if result.is_null() || a.is_null() || b.is_null() {
            return;
        }
        let shift = __gmpz_get_si(&(*b.cast::<ViperBigInt>()).mpz);
        if shift < 0 {
            eprintln!("Error: Negative shift in BigInt rshift");
            return;
        }
        __gmpz_tdiv_q_2exp(&mut (*result.cast::<ViperBigInt>()).mpz, &(*a.cast::<ViperBigInt>()).mpz, shift as u64);
    }
}

/// Stub for vp_bigint_eq
#[no_mangle]
pub extern "C" fn vp_bigint_eq_stub(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool {
    unsafe {
        if a.is_null() || b.is_null() {
            return false;
        }
        __gmpz_cmp(&(*a.cast::<ViperBigInt>()).mpz, &(*b.cast::<ViperBigInt>()).mpz) == 0
    }
}

/// Stub for vp_bigint_lt
#[no_mangle]
pub extern "C" fn vp_bigint_lt_stub(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool {
    unsafe {
        if a.is_null() || b.is_null() {
            return false;
        }
        __gmpz_cmp(&(*a.cast::<ViperBigInt>()).mpz, &(*b.cast::<ViperBigInt>()).mpz) < 0
    }
}

/// Stub for vp_bigint_gt
#[no_mangle]
pub extern "C" fn vp_bigint_gt_stub(a: *mut std::ffi::c_void, b: *mut std::ffi::c_void) -> bool {
    unsafe {
        if a.is_null() || b.is_null() {
            return false;
        }
        __gmpz_cmp(&(*a.cast::<ViperBigInt>()).mpz, &(*b.cast::<ViperBigInt>()).mpz) > 0
    }
}

/// Stub for vp_bigint_destroy
#[no_mangle]
pub extern "C" fn vp_bigint_destroy_stub(bigint: *mut std::ffi::c_void) {
    unsafe {
        if bigint.is_null() {
            return;
        }
        __gmpz_clear(&mut (*bigint.cast::<ViperBigInt>()).mpz);
        libc::free(bigint as *mut libc::c_void);
    }
}
