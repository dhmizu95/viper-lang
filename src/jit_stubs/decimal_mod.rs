// Decimal module stubs for JIT - Phase 4
// Fixed-point decimal arithmetic

use std::str::FromStr;

#[no_mangle]
pub extern "C" fn vp_decimal_create() -> *mut i8 {
    // Return "0" as string representation
    std::ffi::CString::new("0").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vp_decimal_from_str(s: *const i8) -> *mut i8 {
    if s.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s);
        let s_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::ffi::CString::new("0").unwrap().into_raw(),
        };
        
        // Validate and return the string
        std::ffi::CString::new(s_str).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_from_i64(value: i64) -> *mut i8 {
    std::ffi::CString::new(value.to_string()).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vp_decimal_from_f64(value: f64) -> *mut i8 {
    std::ffi::CString::new(format!("{:.15}", value)).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vp_decimal_free(d: *mut i8) {
    if !d.is_null() {
        unsafe { drop(std::ffi::CString::from_raw(d)); }
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_to_str(d: *mut i8) -> *mut i8 {
    if d.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    d
}

#[no_mangle]
pub extern "C" fn vp_decimal_to_i64(d: *mut i8) -> i64 {
    if d.is_null() {
        return 0;
    }
    
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(d);
        let s_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };
        
        f64::from_str(s_str).unwrap_or(0.0) as i64
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_to_f64(d: *mut i8) -> f64 {
    if d.is_null() {
        return 0.0;
    }
    
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(d);
        let s_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return 0.0,
        };
        
        f64::from_str(s_str).unwrap_or(0.0)
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_add(a: *mut i8, b: *mut i8) -> *mut i8 {
    if a.is_null() || b.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let a_str = std::ffi::CStr::from_ptr(a).to_str().unwrap_or("0");
        let b_str = std::ffi::CStr::from_ptr(b).to_str().unwrap_or("0");
        
        let a_val = f64::from_str(a_str).unwrap_or(0.0);
        let b_val = f64::from_str(b_str).unwrap_or(0.0);
        
        std::ffi::CString::new(format!("{:.15}", a_val + b_val)).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_sub(a: *mut i8, b: *mut i8) -> *mut i8 {
    if a.is_null() || b.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let a_str = std::ffi::CStr::from_ptr(a).to_str().unwrap_or("0");
        let b_str = std::ffi::CStr::from_ptr(b).to_str().unwrap_or("0");
        
        let a_val = f64::from_str(a_str).unwrap_or(0.0);
        let b_val = f64::from_str(b_str).unwrap_or(0.0);
        
        std::ffi::CString::new(format!("{:.15}", a_val - b_val)).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_mul(a: *mut i8, b: *mut i8) -> *mut i8 {
    if a.is_null() || b.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let a_str = std::ffi::CStr::from_ptr(a).to_str().unwrap_or("0");
        let b_str = std::ffi::CStr::from_ptr(b).to_str().unwrap_or("0");
        
        let a_val = f64::from_str(a_str).unwrap_or(0.0);
        let b_val = f64::from_str(b_str).unwrap_or(0.0);
        
        std::ffi::CString::new(format!("{:.15}", a_val * b_val)).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_div(a: *mut i8, b: *mut i8) -> *mut i8 {
    if a.is_null() || b.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let a_str = std::ffi::CStr::from_ptr(a).to_str().unwrap_or("0");
        let b_str = std::ffi::CStr::from_ptr(b).to_str().unwrap_or("0");
        
        let a_val = f64::from_str(a_str).unwrap_or(0.0);
        let b_val = f64::from_str(b_str).unwrap_or(1.0);
        
        if b_val == 0.0 {
            return std::ffi::CString::new("NaN").unwrap().into_raw();
        }
        
        std::ffi::CString::new(format!("{:.15}", a_val / b_val)).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_neg(d: *mut i8) -> *mut i8 {
    if d.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        std::ffi::CString::new(format!("{:.15}", -d_val)).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_abs(d: *mut i8) -> *mut i8 {
    if d.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        std::ffi::CString::new(format!("{:.15}", d_val.abs())).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_cmp(a: *mut i8, b: *mut i8) -> i64 {
    if a.is_null() || b.is_null() {
        return 0;
    }
    
    unsafe {
        let a_str = std::ffi::CStr::from_ptr(a).to_str().unwrap_or("0");
        let b_str = std::ffi::CStr::from_ptr(b).to_str().unwrap_or("0");
        
        let a_val = f64::from_str(a_str).unwrap_or(0.0);
        let b_val = f64::from_str(b_str).unwrap_or(0.0);
        
        if a_val < b_val { -1 } else if a_val > b_val { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_eq(a: *mut i8, b: *mut i8) -> i64 {
    if vp_decimal_cmp(a, b) == 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn vp_decimal_lt(a: *mut i8, b: *mut i8) -> i64 {
    if vp_decimal_cmp(a, b) < 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn vp_decimal_le(a: *mut i8, b: *mut i8) -> i64 {
    if vp_decimal_cmp(a, b) <= 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn vp_decimal_gt(a: *mut i8, b: *mut i8) -> i64 {
    if vp_decimal_cmp(a, b) > 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn vp_decimal_ge(a: *mut i8, b: *mut i8) -> i64 {
    if vp_decimal_cmp(a, b) >= 0 { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn vp_decimal_quantize(d: *mut i8, places: i64) -> *mut i8 {
    if d.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        let fmt = format!("{:.1$}", d_val, places as usize);
        std::ffi::CString::new(fmt).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_round(d: *mut i8, places: i64) -> *mut i8 {
    vp_decimal_quantize(d, places)
}

#[no_mangle]
pub extern "C" fn vp_decimal_floor(d: *mut i8) -> *mut i8 {
    if d.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        std::ffi::CString::new(format!("{:.0}", d_val.floor())).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_ceil(d: *mut i8) -> *mut i8 {
    if d.is_null() {
        return std::ffi::CString::new("0").unwrap().into_raw();
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        std::ffi::CString::new(format!("{:.0}", d_val.ceil())).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_get_sign(d: *mut i8) -> i64 {
    if d.is_null() {
        return 0;
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        if d_val < 0.0 { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_get_scale(d: *mut i8) -> i64 {
    if d.is_null() {
        return 0;
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        
        if let Some(dot_pos) = d_str.find('.') {
            (d_str.len() - dot_pos - 1) as i64
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_is_zero(d: *mut i8) -> i64 {
    if d.is_null() {
        return 0;
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        if d_val == 0.0 { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_is_nan(d: *mut i8) -> i64 {
    if d.is_null() {
        return 0;
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        if d_val.is_nan() { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_is_infinite(d: *mut i8) -> i64 {
    if d.is_null() {
        return 0;
    }
    
    unsafe {
        let d_str = std::ffi::CStr::from_ptr(d).to_str().unwrap_or("0");
        let d_val = f64::from_str(d_str).unwrap_or(0.0);
        
        if d_val.is_infinite() { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn vp_decimal_is_signed(d: *mut i8) -> i64 {
    vp_decimal_get_sign(d)
}

// Constants
#[no_mangle]
pub extern "C" fn vp_decimal_zero() -> *mut i8 {
    std::ffi::CString::new("0").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vp_decimal_one() -> *mut i8 {
    std::ffi::CString::new("1").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vp_decimal_pi() -> *mut i8 {
    std::ffi::CString::new("3.141592653589793").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vp_decimal_e() -> *mut i8 {
    std::ffi::CString::new("2.718281828459045").unwrap().into_raw()
}
