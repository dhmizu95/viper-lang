use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};

// Concurrency runtime stubs for JIT (Phase 3)
// Simplified implementations using atomics for safety

static JIT_CHANNEL_COUNTER: AtomicUsize = AtomicUsize::new(0);
static JIT_CHANNEL_VALUE: AtomicI64 = AtomicI64::new(0);
static JIT_WG_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub extern "C" fn vp_chan_create(_capacity: i64) -> *mut std::ffi::c_void {
    let id = JIT_CHANNEL_COUNTER.fetch_add(1, Ordering::SeqCst);
    id as *mut std::ffi::c_void
}

pub extern "C" fn vp_chan_destroy(_chan: *mut std::ffi::c_void) {
    // No-op for JIT
}

pub extern "C" fn vp_chan_send(_chan: *mut std::ffi::c_void, value: i64) {
    JIT_CHANNEL_VALUE.store(value, Ordering::SeqCst);
}

pub extern "C" fn vp_chan_recv(_chan: *mut std::ffi::c_void) -> i64 {
    JIT_CHANNEL_VALUE.load(Ordering::SeqCst)
}

pub extern "C" fn vp_waitgroup_create() -> *mut std::ffi::c_void {
    let id = JIT_WG_COUNTER.fetch_add(1, Ordering::SeqCst);
    id as *mut std::ffi::c_void
}

pub extern "C" fn vp_waitgroup_destroy(_wg: *mut std::ffi::c_void) {
    // No-op for JIT
}

pub extern "C" fn vp_waitgroup_add(_wg: *mut std::ffi::c_void, _n: i64) {
    // No-op for JIT stub
}

pub extern "C" fn vp_waitgroup_done(_wg: *mut std::ffi::c_void) {
    // No-op for JIT stub
}

pub extern "C" fn vp_waitgroup_wait(_wg: *mut std::ffi::c_void) {
    // No-op for JIT stub
}

pub extern "C" fn vp_init_threadpool(_num_threads: usize) {
    // No-op for JIT
}

pub extern "C" fn vp_shutdown_threadpool() {
    // No-op for JIT
}

pub extern "C" fn vp_future_await(future: i64) -> i64 {
    // Stub for async/await - just returns the future value as-is
    // A full implementation would suspend and resume the coroutine
    future
}
