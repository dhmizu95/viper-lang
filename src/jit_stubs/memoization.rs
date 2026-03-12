// Memoization JIT stubs - C implementations linked at runtime

use std::os::raw::{c_int, c_void};

// Opaque cache types
#[repr(C)]
pub struct LRUCache {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Cache {
    _private: [u8; 0],
}

extern "C" {
    // LRU Cache functions
    fn vp_lru_cache_create(maxsize: u64) -> *mut LRUCache;
    fn vp_lru_cache_get(cache: *mut LRUCache, key: *mut c_void, found: *mut c_int, is_bigint: *mut c_int) -> i64;
    fn vp_lru_cache_set(cache: *mut LRUCache, key: *mut c_void, value: i64, key_size: i64, is_bigint: c_int);
    fn vp_lru_cache_destroy(cache: *mut LRUCache);

    // Unbounded Cache functions
    fn vp_cache_create() -> *mut Cache;
    fn vp_cache_get(cache: *mut Cache, key: *mut c_void, found: *mut c_int, is_bigint: *mut c_int) -> i64;
    fn vp_cache_set(cache: *mut Cache, key: *mut c_void, value: i64, key_size: i64, is_bigint: c_int);
    fn vp_cache_destroy(cache: *mut Cache);
    
    // Tuple creation for cache keys (implemented in C runtime)
    fn vp_tuple_create1(value: i64) -> *mut c_void;
    fn vp_tuple_create2(value1: i64, value2: i64) -> *mut c_void;
    
    // Memory management
    fn vp_free(ptr: *mut c_void);
}

// JIT stub wrappers - just call the C functions directly
#[no_mangle]
pub extern "C" fn vp_lru_cache_create_stub(maxsize: u64) -> *mut c_void {
    unsafe { vp_lru_cache_create(maxsize) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_get_stub(cache: *mut c_void, key: *mut c_void, found: *mut c_int, is_bigint: *mut c_int) -> i64 {
    unsafe { vp_lru_cache_get(cache as *mut LRUCache, key, found, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_set_stub(cache: *mut c_void, key: *mut c_void, value: i64, key_size: i64, is_bigint: c_int) {
    unsafe { vp_lru_cache_set(cache as *mut LRUCache, key, value, key_size, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_destroy_stub(cache: *mut c_void) {
    unsafe { vp_lru_cache_destroy(cache as *mut LRUCache) }
}

#[no_mangle]
pub extern "C" fn vp_cache_create_stub() -> *mut c_void {
    unsafe { vp_cache_create() as *mut c_void }
}

#[no_mangle]
pub extern "C" fn vp_cache_get_stub(cache: *mut c_void, key: *mut c_void, found: *mut c_int, is_bigint: *mut c_int) -> i64 {
    unsafe { vp_cache_get(cache as *mut Cache, key, found, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_cache_set_stub(cache: *mut c_void, key: *mut c_void, value: i64, key_size: i64, is_bigint: c_int) {
    unsafe { vp_cache_set(cache as *mut Cache, key, value, key_size, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_cache_destroy_stub(cache: *mut c_void) {
    unsafe { vp_cache_destroy(cache as *mut Cache) }
}

// Tuple creation - call C implementation
#[no_mangle]
pub extern "C" fn vp_tuple_create1_stub(value: i64) -> *mut c_void {
    unsafe { vp_tuple_create1(value) }
}

#[no_mangle]
pub extern "C" fn vp_tuple_create2_stub(value1: i64, value2: i64) -> *mut c_void {
    unsafe { vp_tuple_create2(value1, value2) }
}
