use std::collections::VecDeque;
use std::sync::Mutex;
use crate::jit_stubs::bigint::vp_bigint_to_i64_stub;
use crate::jit_stubs::tagged_int::tagged_int_from_i64;

// Concurrency runtime stubs for JIT (Phase 3)
// Proper implementations using channels and synchronization

struct JitChannel {
    buffer: VecDeque<i64>,
}

impl JitChannel {
    fn new() -> Self {
        JitChannel { buffer: VecDeque::new() }
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
        JitConcurrency { channels: Vec::new(), wg_count: 0 }
    }
}

lazy_static::lazy_static! {
    static ref JIT_CONCURRENCY: Mutex<JitConcurrency> = Mutex::new(JitConcurrency::new());
}

#[inline(always)]
fn tagged_to_i64(val: i64) -> i64 {
    if (val & 1) == 0 {
        val >> 1
    } else {
        // BigInt pointer with tag bit set
        let ptr = (val & !1) as *mut std::ffi::c_void;
        unsafe { vp_bigint_to_i64_stub(ptr as *mut _) }
    }
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

pub extern "C" fn vp_submit_task(
    func: extern "C" fn(*mut std::ffi::c_void),
    data: *mut std::ffi::c_void,
) {
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

// Minimal Future struct for JIT mode (must match runtime/src/async.c)
#[repr(C)]
struct JitFuture {
    ref_count: i64,
    state: i32,        // 0=PENDING, 1=READY, 2=RUNNING, 3=COMPLETED, 4=ERROR
    _pad: i32,
    result: i64,
    callback: *mut std::ffi::c_void,
    user_data: *mut std::ffi::c_void,
    waiting_fiber: *mut std::ffi::c_void,
}

pub extern "C" fn vp_future_await(future: *mut std::ffi::c_void) -> i64 {
    // In JIT mode, use a simplified spin-wait implementation
    if future.is_null() {
        return 0;
    }
    
    unsafe {
        let fut = &*(future as *mut JitFuture);
        
        // Spin-wait until future is completed (state == 3) or error (state == 4)
        // In a real implementation, this would yield the fiber
        while fut.state != 3 && fut.state != 4 {
            std::hint::spin_loop();
        }
        
        fut.result
    }
}

pub extern "C" fn vp_future_set_result(future: *mut std::ffi::c_void, result: i64) {
    if future.is_null() {
        return;
    }
    
    unsafe {
        let fut = &mut *(future as *mut JitFuture);
        fut.result = result;
        fut.state = 3; // COMPLETED
    }
}

pub extern "C" fn vp_future_create() -> *mut std::ffi::c_void {
    let future = Box::new(JitFuture {
        ref_count: 1,
        state: 0,  // PENDING
        _pad: 0,
        result: 0,
        callback: std::ptr::null_mut(),
        user_data: std::ptr::null_mut(),
        waiting_fiber: std::ptr::null_mut(),
    });
    Box::into_raw(future) as *mut std::ffi::c_void
}

// Async range for "async for i in async_range(n)"
pub extern "C" fn vp_async_range_create(start: i64, end: i64, step: i64) -> *mut std::ffi::c_void {
    // Allocate a simple range struct
    let raw_start = tagged_to_i64(start);
    let raw_end = tagged_to_i64(end);
    let raw_step = {
        let s = tagged_to_i64(step);
        if s == 0 { 1 } else { s }
    };
    let range =
        Box::new(JitAsyncRange { 
            magic: JIT_ASYNC_RANGE_MAGIC,
            current: raw_start, 
            end: raw_end, 
            step: raw_step, 
        });
    Box::into_raw(range) as *mut std::ffi::c_void
}

pub extern "C" fn vp_async_range_next(range_ptr: *mut std::ffi::c_void) -> i64 {
    if range_ptr.is_null() {
        return -1;
    }
    let range = unsafe { &mut *(range_ptr as *mut JitAsyncRange) };

    if range.magic != JIT_ASYNC_RANGE_MAGIC {
        return -1;
    }

    if range.current >= range.end {
        return -1; // StopAsyncIteration
    }

    let value = range.current;
    range.current += range.step;
    tagged_int_from_i64(value)
}

pub extern "C" fn vp_async_range_free(range_ptr: *mut std::ffi::c_void) {
    if !range_ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(range_ptr as *mut JitAsyncRange);
        }
    }
}

pub extern "C" fn vp_async_iter(obj: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if obj.is_null() {
        return std::ptr::null_mut();
    }

    let range = unsafe { &mut *(obj as *mut JitAsyncRange) };
    if range.magic == JIT_ASYNC_RANGE_MAGIC {
        return obj;
    }

    std::ptr::null_mut()
}

pub extern "C" fn vp_async_next(iterator: *mut std::ffi::c_void) -> i64 {
    // Call vp_async_range_next for range iterators
    if iterator.is_null() {
        return -1;  // StopAsyncIteration
    }
    
    let range = unsafe { &mut *(iterator as *mut JitAsyncRange) };

    if range.magic != JIT_ASYNC_RANGE_MAGIC {
        return -1;
    }
    
    if range.current >= range.end {
        return -1;  // StopAsyncIteration
    }
    
    let value = range.current;
    range.current += range.step;
    tagged_int_from_i64(value)
}

pub extern "C" fn vp_async_spawn(
    func: extern "C" fn(*mut std::ffi::c_void),
    arg: *mut std::ffi::c_void,
) -> i64 {
    // For JIT mode, run the function synchronously
    // This allows the future result to be set before await
    if !arg.is_null() {
        func(arg);
    }
    0
}

pub extern "C" fn vp_async_run_loop() {
    // No-op for JIT
}

pub extern "C" fn vp_async_sleep(milliseconds: i64) -> i64 {
    // For JIT mode, do a simple blocking sleep and return a completed future
    use std::thread;
    use std::time::Duration;
    
    thread::sleep(Duration::from_millis(milliseconds as u64));
    
    // Return a completed future pointer
    let future = Box::new(JitFuture {
        ref_count: 1,
        state: 3,  // COMPLETED
        _pad: 0,
        result: 0,
        callback: std::ptr::null_mut(),
        user_data: std::ptr::null_mut(),
        waiting_fiber: std::ptr::null_mut(),
    });
    Box::into_raw(future) as i64
}

pub extern "C" fn vp_future_gather(futures_ptr: i64, count: i64) -> i64 {
    // Gather multiple futures and return array of results
    if futures_ptr == 0 || count <= 0 {
        return 0;
    }

    unsafe {
        let futures = std::slice::from_raw_parts(futures_ptr as *const i64, count as usize);
        let mut results = Vec::with_capacity(count as usize);

        for &f in futures {
            let future = f as *mut JitFuture;
            if !future.is_null() {
                // Wait for future to complete
                while (*future).state != 3 && (*future).state != 4 {
                    std::hint::spin_loop();
                }
                results.push((*future).result);
            } else {
                results.push(0);
            }
        }

        // Return pointer to results (convert Vec to raw pointer)
        let results_ptr = results.as_mut_ptr();
        std::mem::forget(results);  // Don't drop, we're transferring ownership
        results_ptr as i64
    }
}

pub extern "C" fn vp_future_gather_free(results_ptr: i64, count: i64) {
    if results_ptr != 0 && count > 0 {
        unsafe {
            let raw = results_ptr as *mut i64;
            // Reconstruct Vec to free correctly
            let _ = Vec::from_raw_parts(raw, count as usize, count as usize);
        }
    }
}

// Internal struct for async range
struct JitAsyncRange {
    magic: u64,
    current: i64,
    end: i64,
    step: i64,
}

const JIT_ASYNC_RANGE_MAGIC: u64 = 0x5650525F41524E47u64; // "VPR_ARNG"
