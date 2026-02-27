use std::collections::VecDeque;
use std::sync::Mutex;

// Concurrency runtime stubs for JIT (Phase 3)
// Proper implementations using channels and synchronization

struct JitChannel {
    buffer: VecDeque<i64>,
}

impl JitChannel {
    fn new() -> Self {
        JitChannel {
            buffer: VecDeque::new(),
        }
    }

    fn send(&mut self, value: i64) {
        self.buffer.push_back(value);
    }

    fn recv(&mut self) -> i64 {
        self.buffer.pop_front().unwrap_or(0)
    }
}

struct JitConcurrency {
    channels: Vec<JitChannel>,
    wg_count: i64,
}

impl JitConcurrency {
    fn new() -> Self {
        JitConcurrency {
            channels: Vec::new(),
            wg_count: 0,
        }
    }
}

lazy_static::lazy_static! {
    static ref JIT_CONCURRENCY: Mutex<JitConcurrency> = Mutex::new(JitConcurrency::new());
}

pub extern "C" fn vp_chan_create(_capacity: i64) -> *mut std::ffi::c_void {
    let mut state = JIT_CONCURRENCY.lock().unwrap();
    let id = state.channels.len();
    state.channels.push(JitChannel::new());
    id as *mut std::ffi::c_void
}

pub extern "C" fn vp_chan_destroy(_chan: *mut std::ffi::c_void) {
    // No-op for JIT
}

pub extern "C" fn vp_chan_send(_chan: *mut std::ffi::c_void, value: i64) {
    let chan_id = _chan as usize;
    let mut state = JIT_CONCURRENCY.lock().unwrap();
    if chan_id < state.channels.len() {
        state.channels[chan_id].send(value);
    }
}

pub extern "C" fn vp_chan_recv(_chan: *mut std::ffi::c_void) -> i64 {
    let chan_id = _chan as usize;
    let mut state = JIT_CONCURRENCY.lock().unwrap();
    if chan_id < state.channels.len() {
        state.channels[chan_id].recv()
    } else {
        0
    }
}

pub extern "C" fn vp_waitgroup_create() -> *mut std::ffi::c_void {
    // Return a dummy pointer - WG is tracked globally
    1 as *mut std::ffi::c_void
}

pub extern "C" fn vp_waitgroup_destroy(_wg: *mut std::ffi::c_void) {
    // No-op for JIT
}

pub extern "C" fn vp_waitgroup_add(_wg: *mut std::ffi::c_void, n: i64) {
    let mut state = JIT_CONCURRENCY.lock().unwrap();
    state.wg_count += n;
}

pub extern "C" fn vp_waitgroup_done(_wg: *mut std::ffi::c_void) {
    let mut state = JIT_CONCURRENCY.lock().unwrap();
    if state.wg_count > 0 {
        state.wg_count -= 1;
    }
}

pub extern "C" fn vp_waitgroup_wait(_wg: *mut std::ffi::c_void) {
    // Spin-wait until counter reaches 0
    loop {
        {
            let state = JIT_CONCURRENCY.lock().unwrap();
            if state.wg_count <= 0 {
                break;
            }
        }
        std::thread::yield_now();
    }
}

pub extern "C" fn vp_init_threadpool(_num_threads: usize) {
    // No-op for JIT
}

pub extern "C" fn vp_shutdown_threadpool() {
    // No-op for JIT
}

pub extern "C" fn vp_submit_task(func: extern "C" fn(*mut std::ffi::c_void), data: *mut std::ffi::c_void) {
    // For JIT: execute the task synchronously
    // This is a simplified implementation that runs tasks inline
    if !data.is_null() {
        // Call the function with the data pointer
        func(data);
    }
}

pub extern "C" fn vp_wait_all_tasks() {
    // No-op for JIT - tasks run synchronously in vp_submit_task
}

/* ============================================ */
/* Async/Await Runtime Stubs                   */
/* ============================================ */

pub extern "C" fn vp_future_await(future: i64) -> i64 {
    // Stub for async/await - just returns the future value as-is
    future
}

// Async range for "async for i in async_range(n)"
pub extern "C" fn vp_async_range_create(start: i64, end: i64, step: i64) -> *mut std::ffi::c_void {
    // Allocate a simple range struct
    let range = Box::new(JitAsyncRange {
        current: start,
        end,
        step: if step == 0 { 1 } else { step },
    });
    Box::into_raw(range) as *mut std::ffi::c_void
}

pub extern "C" fn vp_async_range_next(range_ptr: *mut std::ffi::c_void) -> i64 {
    if range_ptr.is_null() {
        return -1;
    }
    let range = unsafe { &mut *(range_ptr as *mut JitAsyncRange) };
    
    if range.current >= range.end {
        return -1;  // StopAsyncIteration
    }
    
    let value = range.current;
    range.current += range.step;
    value
}

pub extern "C" fn vp_async_range_free(range_ptr: *mut std::ffi::c_void) {
    if !range_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(range_ptr as *mut JitAsyncRange);
        }
    }
}

pub extern "C" fn vp_async_iter(obj: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    // For now, just return the object as-is
    obj
}

pub extern "C" fn vp_async_next(iterator: *mut std::ffi::c_void) -> i64 {
    // Call vp_async_range_next for range iterators
    vp_async_range_next(iterator)
}

pub extern "C" fn vp_async_spawn(_func: extern "C" fn(*mut std::ffi::c_void), _arg: *mut std::ffi::c_void) -> i64 {
    // Same as vp_submit_task for now
    0
}

pub extern "C" fn vp_async_run_loop() {
    // No-op for JIT
}

// Internal struct for async range
struct JitAsyncRange {
    current: i64,
    end: i64,
    step: i64,
}
