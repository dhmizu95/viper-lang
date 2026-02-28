// Hashlib module stubs for JIT - Phase 4
// SHA-256, MD5, SHA-512 hash functions

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[no_mangle]
pub extern "C" fn vp_hash_sha256(data: *const i8, len: i64) -> *mut i8 {
    if data.is_null() || len <= 0 {
        return std::ffi::CString::new("").unwrap().into_raw();
    }

    unsafe {
        let bytes = std::slice::from_raw_parts(data as *const u8, len as usize);

        // Simplified SHA-256 using Rust's built-in hash (not actual SHA-256)
        // For production, use sha2 crate
        let mut hasher = DefaultHasher::new();
        hasher.write(bytes);
        let hash = hasher.finish();

        let hex = format!("{:016x}", hash);
        std::ffi::CString::new(hex).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_hash_md5(data: *const i8, len: i64) -> *mut i8 {
    if data.is_null() || len <= 0 {
        return std::ffi::CString::new("").unwrap().into_raw();
    }

    unsafe {
        let bytes = std::slice::from_raw_parts(data as *const u8, len as usize);

        // Simplified MD5 using Rust's built-in hash
        let mut hasher = DefaultHasher::new();
        hasher.write(bytes);
        let hash = hasher.finish();

        let hex = format!("{:016x}", hash);
        std::ffi::CString::new(hex).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_hash_sha512(data: *const i8, len: i64) -> *mut i8 {
    if data.is_null() || len <= 0 {
        return std::ffi::CString::new("").unwrap().into_raw();
    }

    unsafe {
        let bytes = std::slice::from_raw_parts(data as *const u8, len as usize);

        // Simplified SHA-512 using Rust's built-in hash
        let mut hasher = DefaultHasher::new();
        hasher.write(bytes);
        let hash = hasher.finish();

        let hex = format!("{:016x}", hash);
        std::ffi::CString::new(hex).unwrap().into_raw()
    }
}

// Hash object
pub struct ViperHash {
    algo: String,
    data: Vec<u8>,
}

#[no_mangle]
pub extern "C" fn vp_hashlib_new(algo: *const i8) -> *mut ViperHash {
    if algo.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(algo);
        let algo_str = match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        };

        let hash = Box::new(ViperHash { algo: algo_str, data: Vec::new() });
        Box::into_raw(hash)
    }
}

#[no_mangle]
pub extern "C" fn vp_hashlib_free(h: *mut ViperHash) {
    if !h.is_null() {
        unsafe {
            drop(Box::from_raw(h));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_hashlib_update(h: *mut ViperHash, data: *const i8, len: i64) {
    if h.is_null() || data.is_null() || len <= 0 {
        return;
    }

    unsafe {
        let hash = &mut *h;
        let bytes = std::slice::from_raw_parts(data as *const u8, len as usize);
        hash.data.extend_from_slice(bytes);
    }
}

#[no_mangle]
pub extern "C" fn vp_hashlib_digest(h: *mut ViperHash) -> *mut i8 {
    if h.is_null() {
        return std::ffi::CString::new("").unwrap().into_raw();
    }

    unsafe {
        let hash = &*h;

        // Simplified hash computation
        let mut hasher = DefaultHasher::new();
        hasher.write(&hash.data);
        let result = hasher.finish();

        let hex = format!("{:016x}", result);
        std::ffi::CString::new(hex).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_hashlib_hexdigest(h: *mut ViperHash) -> *mut i8 {
    vp_hashlib_digest(h)
}

// Constants
#[no_mangle]
pub extern "C" fn vp_hashlib_block_size_md5() -> i64 {
    64
}

#[no_mangle]
pub extern "C" fn vp_hashlib_block_size_sha256() -> i64 {
    64
}

#[no_mangle]
pub extern "C" fn vp_hashlib_block_size_sha512() -> i64 {
    128
}

#[no_mangle]
pub extern "C" fn vp_hashlib_digest_size_md5() -> i64 {
    16
}

#[no_mangle]
pub extern "C" fn vp_hashlib_digest_size_sha256() -> i64 {
    32
}

#[no_mangle]
pub extern "C" fn vp_hashlib_digest_size_sha512() -> i64 {
    64
}
