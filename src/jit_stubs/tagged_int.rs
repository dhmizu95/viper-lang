#![allow(unused_unsafe)]

use std::os::raw::c_char;
use std::ffi::{CString, c_void};
use crate::jit_stubs::bigint::{vp_bigint_add_stub, vp_bigint_sub_stub, vp_bigint_mul_stub, vp_bigint_div_stub, vp_bigint_mod_stub, vp_bigint_pow_stub, vp_bigint_neg_stub, vp_bigint_eq_stub, vp_bigint_lt_stub, vp_bigint_gt_stub, vp_bigint_to_str_stub, vp_bigint_free_stub, vp_bigint_from_i64_stub, vp_bigint_cmp_stub, vp_bigint_to_i64_stub};

// Tagged pointer representation:
// - LSB = 0: small int (i63), value is shifted left by 1 (val << 1)
// - LSB = 1: BigInt pointer (ptr | 1)

const TAG_BIT: i64 = 1;

#[inline(always)]
fn is_bigint(val: i64) -> bool {
    (val & TAG_BIT) != 0
}

#[inline(always)]
fn get_small_int(val: i64) -> i64 {
    val >> 1
}

#[inline(always)]
fn make_small_int(val: i64) -> i64 {
    val << 1
}

#[inline(always)]
fn extract_ptr(val: i64) -> *mut c_void {
    (val & !TAG_BIT) as *mut c_void
}

#[inline(always)]
fn make_tagged_ptr(ptr: *mut c_void) -> i64 {
    (ptr as i64) | TAG_BIT
}

// Check if an i64 can fit in i63
#[inline(always)]
fn fits_in_i63(val: i64) -> bool {
    val >= -(1 << 62) && val <= ((1 << 62) - 1)
}

/**
 * Try to demote a BigInt result back to SmallInt
 * Returns the SmallInt tagged value if demotion is possible, otherwise None
 */
#[inline(always)]
fn try_demote_bigint(res_ptr: *mut c_void) -> Option<i64> {
    unsafe {
        // Get the i64 value from the BigInt
        let value = vp_bigint_to_i64_stub(res_ptr as *mut _);
        
        // Check if the value fits in i63 range
        if !fits_in_i63(value) {
            return None;
        }
        
        // Create a temporary BigInt from the i64 value
        let temp_big = vp_bigint_from_i64_stub(value);
        
        // Compare the original BigInt with the temporary one
        // If they're equal, the value can be safely demoted
        let cmp_result = vp_bigint_cmp_stub(res_ptr as *mut _, temp_big as *mut _);
        vp_bigint_free_stub(temp_big as *mut _);
        
        if cmp_result == 0 {
            return Some(make_small_int(value));
        }
    }
    None
}

// Check if adding two i63 values would overflow i63
#[inline(always)]
fn would_overflow_i63_add(a: i64, b: i64) -> bool {
    const MAX_I63: i64 = (1 << 62) - 1;
    const MIN_I63: i64 = -(1 << 62);
    if b > 0 && a > MAX_I63 - b { return true; }
    if b < 0 && a < MIN_I63 - b { return true; }
    false
}

// Check if multiplying two i63 values would overflow i63
#[inline(always)]
fn would_overflow_i63_mul(a: i64, b: i64) -> bool {
    if a == 0 || b == 0 { return false; }
    if a == 1 || b == 1 { return false; }
    const MAX_I63: i64 = (1 << 62) - 1;
    const MIN_I63: i64 = -(1 << 62);
    if a > 0 {
        if b > 0 {
            return a > MAX_I63 / b;
        } else {
            return b < MIN_I63 / a;
        }
    } else {
        if b > 0 {
            return a < MIN_I63 / b;
        } else {
            return b < MAX_I63 / a;
        }
    }
}

#[no_mangle]
pub extern "C" fn tagged_int_from_i64(val: i64) -> i64 {
    if fits_in_i63(val) {
        make_small_int(val)
    } else {
        let ptr = vp_bigint_from_i64_stub(val);
        make_tagged_ptr(ptr as *mut c_void)
    }
}

#[no_mangle]
pub extern "C" fn tagged_int_from_str(s: *const i8) -> i64 {
    let ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_str_stub(s) };
    make_tagged_ptr(ptr as *mut c_void)
}

pub fn convert_to_bigint_ptr(val: i64) -> *mut c_void {
    if is_bigint(val) {
        extract_ptr(val)
    } else {
        vp_bigint_from_i64_stub(get_small_int(val)) as *mut c_void
    }
}

macro_rules! binary_op {
    ($name:ident, $small_op:expr, $big_stub:ident, $overflow_check:expr) => {
        #[no_mangle]
        pub extern "C" fn $name(lhs: i64, rhs: i64) -> i64 {
            if !is_bigint(lhs) && !is_bigint(rhs) {
                let l = get_small_int(lhs);
                let r = get_small_int(rhs);
                let (res, overflow) = $small_op(l, r);
                let i63_overflow = $overflow_check(l, r);
                // Check both i64 overflow and i63 range
                if !overflow && !i63_overflow && fits_in_i63(res) {
                    return make_small_int(res);
                }
            }

            // At least one is BigInt or overflow occurred
            let l_ptr = convert_to_bigint_ptr(lhs);
            let r_ptr = convert_to_bigint_ptr(rhs);

            let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
            unsafe { $big_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };

            // Try to demote the result back to SmallInt
            if let Some(demoted) = try_demote_bigint(res_ptr as *mut c_void) {
                // Demotion successful - free the BigInt result and return SmallInt
                unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
                // Clean up temporary bigints
                if !is_bigint(lhs) {
                    unsafe { vp_bigint_free_stub(l_ptr as *mut _) };
                }
                if !is_bigint(rhs) {
                    unsafe { vp_bigint_free_stub(r_ptr as *mut _) };
                }
                return demoted;
            }

            // Clean up temporary bigints
            if !is_bigint(lhs) {
                unsafe { vp_bigint_free_stub(l_ptr as *mut _) };
            }
            if !is_bigint(rhs) {
                unsafe { vp_bigint_free_stub(r_ptr as *mut _) };
            }

            make_tagged_ptr(res_ptr as *mut c_void)
        }
    };
}

binary_op!(tagged_int_add, i64::overflowing_add, vp_bigint_add_stub, would_overflow_i63_add);
binary_op!(tagged_int_sub, i64::overflowing_sub, vp_bigint_sub_stub, would_overflow_i63_add);
binary_op!(tagged_int_mul, i64::overflowing_mul, vp_bigint_mul_stub, would_overflow_i63_mul);

#[no_mangle]
pub extern "C" fn tagged_int_div(lhs: i64, rhs: i64) -> i64 {
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let l = get_small_int(lhs);
        let r = get_small_int(rhs);
        if r != 0 {
            let res = l.wrapping_div(r);
            // i64::MIN / -1 overflows i64, but since we are bounded by i63,
            // the max magnitude is 2^62, division by -1 won't overflow the physical i64 type.
            // But it might overflow i63? Actually, no, because -(-2^62) = 2^62,
            // which doesn't fit in i63, so let's check fits_in_i63.
            if fits_in_i63(res) {
                return make_small_int(res);
            }
        } else {
            // Error handling for division by zero
            eprintln!("ZeroDivisionError: integer division or modulo by zero");
            std::process::exit(1);
        }
    }

    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);

    // Check divide by zero for bigint happens inside stub
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { vp_bigint_div_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };

    // Try to demote the result back to SmallInt
    if let Some(demoted) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
        if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
        return demoted;
    }

    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }

    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_mod(lhs: i64, rhs: i64) -> i64 {
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let l = get_small_int(lhs);
        let r = get_small_int(rhs);
        if r != 0 {
            // Python modulo behavior
            let rem = l % r;
            let res = if rem != 0 && (l ^ r) < 0 { rem + r } else { rem };
            if fits_in_i63(res) {
                return make_small_int(res);
            }
        } else {
            eprintln!("ZeroDivisionError: integer division or modulo by zero");
            std::process::exit(1);
        }
    }
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);

    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { vp_bigint_mod_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };

    // Try to demote the result back to SmallInt
    if let Some(demoted) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
        if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
        return demoted;
    }

    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }

    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_neg(operand: i64) -> i64 {
    if !is_bigint(operand) {
        let val = get_small_int(operand);
        let (res, overflow) = 0i64.overflowing_sub(val);
        if !overflow && fits_in_i63(res) {
            return make_small_int(res);
        }
    }
    let ptr = convert_to_bigint_ptr(operand);
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { vp_bigint_neg_stub(res_ptr as *mut _, ptr as *mut _) };
    
    // Try to demote the result back to SmallInt
    if let Some(demoted) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        if !is_bigint(operand) { unsafe { vp_bigint_free_stub(ptr as *mut _) }; }
        return demoted;
    }
    
    if !is_bigint(operand) { unsafe { vp_bigint_free_stub(ptr as *mut _) }; }
    make_tagged_ptr(res_ptr as *mut c_void)
}

macro_rules! cmp_op {
    ($name:ident, $small_op:expr, $big_stub:ident) => {
        #[no_mangle]
        pub extern "C" fn $name(lhs: i64, rhs: i64) -> bool {
            if !is_bigint(lhs) && !is_bigint(rhs) {
                let l = get_small_int(lhs);
                let r = get_small_int(rhs);
                return $small_op(l, r);
            }
            
            let l_ptr = convert_to_bigint_ptr(lhs);
            let r_ptr = convert_to_bigint_ptr(rhs);
            
            let res = unsafe { $big_stub(l_ptr as *mut _, r_ptr as *mut _) };
            
            if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
            if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
            
            res
        }
    };
}

cmp_op!(tagged_int_eq, |l, r| l == r, vp_bigint_eq_stub);
cmp_op!(tagged_int_lt, |l, r| l < r, vp_bigint_lt_stub);
cmp_op!(tagged_int_gt, |l, r| l > r, vp_bigint_gt_stub);

#[no_mangle]
pub extern "C" fn tagged_int_cmp(lhs: i64, rhs: i64) -> i64 {
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let l = get_small_int(lhs);
        let r = get_small_int(rhs);
        return if l < r { -1 } else if l > r { 1 } else { 0 };
    }
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);
    
    let res = unsafe { vp_bigint_cmp_stub(l_ptr as *mut _, r_ptr as *mut _) } as i64;
    
    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
    
    res
}

#[no_mangle]
pub extern "C" fn tagged_int_pow(lhs: i64, rhs: i64) -> i64 {
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);
    
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { vp_bigint_pow_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };
    
    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
    
    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_to_str(val: i64) -> *mut c_char {
    if !is_bigint(val) {
        let v = get_small_int(val);
        let s = v.to_string();
        CString::new(s).unwrap().into_raw()
    } else {
        unsafe { vp_bigint_to_str_stub(extract_ptr(val) as *mut _, 10) as *mut c_char }
    }
}

#[no_mangle]
pub extern "C" fn tagged_int_print(val: i64) {
    let s_ptr = tagged_int_to_str(val);
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s_ptr);
        print!("{}", c_str.to_str().unwrap());
        // Since we created the string, we might need to free it. 
        // vp_bigint_to_str_stub returns a malloc'd string, so does CString::into_raw.
        libc::free(s_ptr as *mut libc::c_void);
    }
}

#[no_mangle]
pub extern "C" fn tagged_int_free(val: i64) {
    if is_bigint(val) {
        unsafe { vp_bigint_free_stub(extract_ptr(val) as *mut _) };
    }
}

#[no_mangle]
pub extern "C" fn tagged_int_bitand(lhs: i64, rhs: i64) -> i64 {
    // Both small integers - direct bitwise AND
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let a = get_small_int(lhs);
        let b = get_small_int(rhs);
        return make_small_int(a & b);
    }
    
    // At least one BigInt - convert both to BigInt and use GMP
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);
    
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { crate::jit_stubs::bigint::vp_bigint_and_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };
    
    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
    
    // Try to demote back to small int
    if let Some(small) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        return small;
    }
    
    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_bitor(lhs: i64, rhs: i64) -> i64 {
    // Both small integers - direct bitwise OR
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let a = get_small_int(lhs);
        let b = get_small_int(rhs);
        return make_small_int(a | b);
    }
    
    // At least one BigInt - convert both to BigInt and use GMP
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);
    
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { crate::jit_stubs::bigint::vp_bigint_or_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };
    
    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
    
    // Try to demote back to small int
    if let Some(small) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        return small;
    }
    
    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_bitxor(lhs: i64, rhs: i64) -> i64 {
    // Both small integers - direct bitwise XOR
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let a = get_small_int(lhs);
        let b = get_small_int(rhs);
        return make_small_int(a ^ b);
    }
    
    // At least one BigInt - convert both to BigInt and use GMP
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);
    
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { crate::jit_stubs::bigint::vp_bigint_xor_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };
    
    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
    
    // Try to demote back to small int
    if let Some(small) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        return small;
    }
    
    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_lshift(lhs: i64, rhs: i64) -> i64 {
    // Both small integers - direct left shift
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let a = get_small_int(lhs);
        let b = get_small_int(rhs);
        // Check for overflow
        if b < 0 || b >= 63 {
            // Result would overflow, use BigInt
            let l_ptr = convert_to_bigint_ptr(lhs);
            let r_ptr = convert_to_bigint_ptr(rhs);
            let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
            unsafe { crate::jit_stubs::bigint::vp_bigint_lshift_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };
            unsafe { vp_bigint_free_stub(l_ptr as *mut _) };
            unsafe { vp_bigint_free_stub(r_ptr as *mut _) };
            if let Some(small) = try_demote_bigint(res_ptr as *mut c_void) {
                unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
                return small;
            }
            return make_tagged_ptr(res_ptr as *mut c_void);
        }
        return make_small_int(a << b);
    }
    
    // At least one BigInt - use GMP
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);
    
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { crate::jit_stubs::bigint::vp_bigint_lshift_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };
    
    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
    
    if let Some(small) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        return small;
    }
    
    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_rshift(lhs: i64, rhs: i64) -> i64 {
    // Both small integers - direct right shift
    if !is_bigint(lhs) && !is_bigint(rhs) {
        let a = get_small_int(lhs);
        let b = get_small_int(rhs);
        if b < 0 {
            return make_small_int(0);
        }
        if b >= 63 {
            return make_small_int(if a >= 0 { 0 } else { -1 });
        }
        return make_small_int(a >> b);
    }
    
    // At least one BigInt - use GMP
    let l_ptr = convert_to_bigint_ptr(lhs);
    let r_ptr = convert_to_bigint_ptr(rhs);
    
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { crate::jit_stubs::bigint::vp_bigint_rshift_stub(res_ptr as *mut _, l_ptr as *mut _, r_ptr as *mut _) };
    
    if !is_bigint(lhs) { unsafe { vp_bigint_free_stub(l_ptr as *mut _) }; }
    if !is_bigint(rhs) { unsafe { vp_bigint_free_stub(r_ptr as *mut _) }; }
    
    if let Some(small) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        return small;
    }
    
    make_tagged_ptr(res_ptr as *mut c_void)
}

#[no_mangle]
pub extern "C" fn tagged_int_invert(val: i64) -> i64 {
    // Small integer - direct bitwise NOT
    if !is_bigint(val) {
        let a = get_small_int(val);
        return make_small_int(!a);
    }
    
    // BigInt - use GMP
    let ptr = convert_to_bigint_ptr(val);
    let res_ptr = unsafe { crate::jit_stubs::bigint::vp_bigint_from_i64_temp_stub(0) };
    unsafe { crate::jit_stubs::bigint::vp_bigint_invert_stub(res_ptr as *mut _, ptr as *mut _) };
    unsafe { vp_bigint_free_stub(ptr as *mut _) };
    
    if let Some(small) = try_demote_bigint(res_ptr as *mut c_void) {
        unsafe { vp_bigint_free_stub(res_ptr as *mut _) };
        return small;
    }
    
    make_tagged_ptr(res_ptr as *mut c_void)
}
