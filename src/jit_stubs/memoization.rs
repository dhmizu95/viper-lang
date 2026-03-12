// Memoization JIT stubs - C implementations linked at runtime
// Uses ARC (Automatic Reference Counting) for memory management

use std::os::raw::{c_int, c_void};

// Opaque ARC key type
#[repr(C)]
pub struct ARCCacheKey {
    _private: [u8; 0],
}

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
    // ARC key creation functions (1-8 parameters)
    fn arc_key_create1(value: i64) -> *mut ARCCacheKey;
    fn arc_key_create2(v1: i64, v2: i64) -> *mut ARCCacheKey;
    fn arc_key_create3(v1: i64, v2: i64, v3: i64) -> *mut ARCCacheKey;
    fn arc_key_create4(v1: i64, v2: i64, v3: i64, v4: i64) -> *mut ARCCacheKey;
    fn arc_key_create5(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64) -> *mut ARCCacheKey;
    fn arc_key_create6(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64) -> *mut ARCCacheKey;
    fn arc_key_create7(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64) -> *mut ARCCacheKey;
    fn arc_key_create8(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64) -> *mut ARCCacheKey;
    
    // LRU Cache functions (updated signatures for ARCCacheKey*)
    fn vp_lru_cache_create(maxsize: u64) -> *mut LRUCache;
    fn vp_lru_cache_get(cache: *mut LRUCache, key: *mut ARCCacheKey, found: *mut c_int, is_bigint: *mut c_int) -> i64;
    fn vp_lru_cache_set(cache: *mut LRUCache, key: *mut ARCCacheKey, value: i64, is_bigint: c_int);
    fn vp_lru_cache_destroy(cache: *mut LRUCache);

    // Unbounded Cache functions (updated signatures for ARCCacheKey*)
    fn vp_cache_create() -> *mut Cache;
    fn vp_cache_get(cache: *mut Cache, key: *mut ARCCacheKey, found: *mut c_int, is_bigint: *mut c_int) -> i64;
    fn vp_cache_set(cache: *mut Cache, key: *mut ARCCacheKey, value: i64, is_bigint: c_int);
    fn vp_cache_destroy(cache: *mut Cache);

    // Backward compatibility wrappers
    fn vp_tuple_create1(value: i64) -> *mut c_void;
    fn vp_tuple_create2(value1: i64, value2: i64) -> *mut c_void;

    // Memory management
    fn vp_free(ptr: *mut c_void);
}

// ARC key creation stubs
#[no_mangle]
pub extern "C" fn arc_key_create1_stub(value: i64) -> *mut c_void {
    unsafe { arc_key_create1(value) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create2_stub(v1: i64, v2: i64) -> *mut c_void {
    unsafe { arc_key_create2(v1, v2) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create3_stub(v1: i64, v2: i64, v3: i64) -> *mut c_void {
    unsafe { arc_key_create3(v1, v2, v3) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create4_stub(v1: i64, v2: i64, v3: i64, v4: i64) -> *mut c_void {
    unsafe { arc_key_create4(v1, v2, v3, v4) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create5_stub(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64) -> *mut c_void {
    unsafe { arc_key_create5(v1, v2, v3, v4, v5) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create6_stub(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64) -> *mut c_void {
    unsafe { arc_key_create6(v1, v2, v3, v4, v5, v6) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create7_stub(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64) -> *mut c_void {
    unsafe { arc_key_create7(v1, v2, v3, v4, v5, v6, v7) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create8_stub(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64) -> *mut c_void {
    unsafe { arc_key_create8(v1, v2, v3, v4, v5, v6, v7, v8) as *mut c_void }
}

// LRU Cache stubs (updated signatures)
#[no_mangle]
pub extern "C" fn vp_lru_cache_create_stub(maxsize: u64) -> *mut c_void {
    unsafe { vp_lru_cache_create(maxsize) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_get_stub(cache: *mut c_void, key: *mut c_void, found: *mut c_int, is_bigint: *mut c_int) -> i64 {
    unsafe { vp_lru_cache_get(cache as *mut LRUCache, key as *mut ARCCacheKey, found, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_set_stub(cache: *mut c_void, key: *mut c_void, value: i64, is_bigint: c_int) {
    unsafe { vp_lru_cache_set(cache as *mut LRUCache, key as *mut ARCCacheKey, value, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_destroy_stub(cache: *mut c_void) {
    unsafe { vp_lru_cache_destroy(cache as *mut LRUCache) }
}

// Unbounded Cache stubs (updated signatures)
#[no_mangle]
pub extern "C" fn vp_cache_create_stub() -> *mut c_void {
    unsafe { vp_cache_create() as *mut c_void }
}

#[no_mangle]
pub extern "C" fn vp_cache_get_stub(cache: *mut c_void, key: *mut c_void, found: *mut c_int, is_bigint: *mut c_int) -> i64 {
    unsafe { vp_cache_get(cache as *mut Cache, key as *mut ARCCacheKey, found, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_cache_set_stub(cache: *mut c_void, key: *mut c_void, value: i64, is_bigint: c_int) {
    unsafe { vp_cache_set(cache as *mut Cache, key as *mut ARCCacheKey, value, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_cache_destroy_stub(cache: *mut c_void) {
    unsafe { vp_cache_destroy(cache as *mut Cache) }
}

// Backward compatibility stubs
#[no_mangle]
pub extern "C" fn vp_tuple_create1_stub(value: i64) -> *mut c_void {
    unsafe { vp_tuple_create1(value) }
}

#[no_mangle]
pub extern "C" fn vp_tuple_create2_stub(value1: i64, value2: i64) -> *mut c_void {
    unsafe { vp_tuple_create2(value1, value2) }
}
