// BigInt JIT stubs using num-bigint

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::os::raw::c_char;

// ViperBigInt - wrapper around num_bigint::BigInt
#[repr(C)]
pub struct ViperBigInt {
    pub bigint: BigInt,
}

// Allocate a new BigInt
fn allocate_bigint() -> *mut ViperBigInt {
    let boxed = Box::new(ViperBigInt { bigint: BigInt::new(num_bigint::Sign::Plus, vec![]) });
    Box::into_raw(boxed)
}

// Destructor - called by ARC when refcount reaches 0
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_destructor(ptr: *mut std::ffi::c_void) {
    if ptr.is_null() {
        return;
    }
    // BigInt automatically cleans up when dropped
    let _ = Box::from_raw(ptr as *mut ViperBigInt);
}

// === CONSTRUCTORS ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_from_i64(val: i64) -> *mut ViperBigInt {
    let ptr = allocate_bigint();
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    (*ptr).bigint = BigInt::from(val);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_from_str(ptr: *const c_char) -> *mut ViperBigInt {
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = std::ffi::CStr::from_ptr(ptr);
    let rust_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    match rust_str.parse::<BigInt>() {
        Ok(bigint) => {
            (*result).bigint = bigint;
            result
        }
        Err(_) => {
            let _ = Box::from_raw(result);
            std::ptr::null_mut()
        }
    }
}

// === ARITHMETIC ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_add(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = &(*a).bigint + &(*b).bigint;
    result
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_sub(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = &(*a).bigint - &(*b).bigint;
    result
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_mul(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = &(*a).bigint * &(*b).bigint;
    result
}

// True division - returns f64
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_div(a: *mut ViperBigInt, b: *mut ViperBigInt) -> f64 {
    if a.is_null() || b.is_null() {
        return 0.0;
    }

    let a_val = match a.as_ref() {
        Some(a) => a.bigint.to_f64().unwrap_or(0.0),
        None => 0.0,
    };
    let b_val = match b.as_ref() {
        Some(b) => b.bigint.to_f64().unwrap_or(1.0),
        None => 1.0,
    };

    if b_val == 0.0 {
        return 0.0;
    }

    a_val / b_val
}

// Floor division - returns BigInt
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_floor_div(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    // Floor division
    (*result).bigint = &(*a).bigint / &(*b).bigint;
    result
}

// Modulo - returns BigInt
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_mod(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = &(*a).bigint % &(*b).bigint;
    result
}

// === POWER ===

// pow(base, exp) - standard power
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_pow(base: *mut ViperBigInt, exp: u64) -> *mut ViperBigInt {
    if base.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = base.as_ref().unwrap().bigint.clone().pow(exp as u32);
    result
}

// pow(base, exp, mod) - modular exponentiation
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_pow_mod(
    base: *mut ViperBigInt,
    exp: *mut ViperBigInt,
    mod_: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if base.is_null() || exp.is_null() || mod_.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    // Modular exponentiation using powm
    (*result).bigint = base
        .as_ref()
        .unwrap()
        .bigint
        .modpow(&exp.as_ref().unwrap().bigint, &mod_.as_ref().unwrap().bigint);
    result
}

// === BITWISE ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_and(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = &(*a).bigint & &(*b).bigint;
    result
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_or(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = &(*a).bigint | &(*b).bigint;
    result
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_xor(
    a: *mut ViperBigInt,
    b: *mut ViperBigInt,
) -> *mut ViperBigInt {
    if a.is_null() || b.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = &(*a).bigint ^ &(*b).bigint;
    result
}

// Bitwise NOT: ~a
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_not(a: *mut ViperBigInt) -> *mut ViperBigInt {
    if a.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    // Bitwise complement: !a = -a - 1
    (*result).bigint = !&(*a).bigint;
    result
}

// Left shift: a << n
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_shl(a: *mut ViperBigInt, shift: u64) -> *mut ViperBigInt {
    if a.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = a.as_ref().unwrap().bigint.clone() << shift as u32;
    result
}

// Right shift: a >> n
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_shr(a: *mut ViperBigInt, shift: u64) -> *mut ViperBigInt {
    if a.is_null() {
        return std::ptr::null_mut();
    }

    let result = allocate_bigint();
    if result.is_null() {
        return std::ptr::null_mut();
    }

    (*result).bigint = a.as_ref().unwrap().bigint.clone() >> shift as u32;
    result
}

// === COMPARISON ===

// Returns: -1 if a < b, 0 if a == b, 1 if a > b
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_cmp(a: *mut ViperBigInt, b: *mut ViperBigInt) -> i32 {
    if a.is_null() || b.is_null() {
        return 0;
    }

    a.as_ref().unwrap().bigint.cmp(&b.as_ref().unwrap().bigint) as i32
}

// === METHODS ===

// .bit_length()
#[no_mangle]
pub unsafe extern "C" fn vp_bigint_bit_length(a: *mut ViperBigInt) -> u64 {
    if a.is_null() {
        return 0;
    }

    // Get bit length
    let bits = a.as_ref().map(|a| a.bigint.bits() as u64).unwrap_or(0);
    bits
}

// === CONVERSION ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_to_i64(a: *mut ViperBigInt) -> i64 {
    if a.is_null() {
        return 0;
    }

    a.as_ref().map(|a| a.bigint.to_i64().unwrap_or(0)).unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_to_f64(a: *mut ViperBigInt) -> f64 {
    if a.is_null() {
        return 0.0;
    }

    a.as_ref().map(|a| a.bigint.to_f64().unwrap_or(0.0)).unwrap_or(0.0)
}

// === ARC (stub implementations - for future integration) ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_retain(_a: *mut ViperBigInt) {
    // TODO: Implement ARC reference counting
}

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_release(_a: *mut ViperBigInt) {
    // TODO: Implement ARC reference counting
}

// === PRINT ===

#[no_mangle]
pub unsafe extern "C" fn vp_bigint_print(a: *mut ViperBigInt) {
    if a.is_null() {
        println!("0");
        return;
    }

    println!("{}", a.as_ref().unwrap().bigint);
}
