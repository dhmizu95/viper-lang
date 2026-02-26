//! JIT stubs for hash functions

/// Hash an i64 value (MurmurHash3 mixer)
#[no_mangle]
pub extern "C" fn vp_hash_i64(val: i64) -> i64 {
    let mut hash = val as u64;
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51afd7ed558ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
    hash ^= hash >> 33;
    hash as i64
}

/// Hash an f64 value
#[no_mangle]
pub extern "C" fn vp_hash_f64(val: f64) -> i64 {
    let bits = val.to_bits();
    vp_hash_i64(bits as i64)
}

/// Hash a bool value
#[no_mangle]
pub extern "C" fn vp_hash_bool(val: bool) -> i64 {
    if val { 1 } else { 0 }
}

/// Hash a string (FNV-1a)
#[no_mangle]
pub extern "C" fn vp_hash_str(str_ptr: *mut std::ffi::c_void) -> i64 {
    if str_ptr.is_null() {
        return 0;
    }
    
    let c_str = str_ptr as *const std::ffi::c_char;
    let bytes = unsafe { std::ffi::CStr::from_ptr(c_str).to_bytes() };
    
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i64
}

/// Hash for None (returns 0)
#[no_mangle]
pub extern "C" fn vp_hash_none() -> i64 {
    0
}
