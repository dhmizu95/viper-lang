use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};
use std::cell::Cell;
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

struct JitTask {
    func: extern "C" fn(*mut std::ffi::c_void),
    arg: usize,
}

struct SleepTask {
    future: *mut std::ffi::c_void,
    ms: u64,
}

extern "C" fn run_sleep_task(arg: *mut std::ffi::c_void) {
    if arg.is_null() {
        return;
    }
    let task = unsafe { Box::from_raw(arg as *mut SleepTask) };
    std::thread::sleep(std::time::Duration::from_millis(task.ms));
    vp_future_set_result(task.future, 0);
}

struct JitConcurrency {
    channels: Vec<JitChannel>,
    wg_count: i64,
    tasks: VecDeque<JitTask>,
    worker_running: bool,
}

impl JitConcurrency {
    fn new() -> Self {
        JitConcurrency {
            channels: Vec::new(),
            wg_count: 0,
            tasks: VecDeque::new(),
            worker_running: false,
        }
    }
}

lazy_static::lazy_static! {
    static ref JIT_CONCURRENCY: Mutex<JitConcurrency> = Mutex::new(JitConcurrency::new());
}

thread_local! {
    static IN_ASYNC_EXEC: Cell<bool> = Cell::new(false);
}

fn run_task(func: extern "C" fn(*mut std::ffi::c_void), arg: *mut std::ffi::c_void) {
    IN_ASYNC_EXEC.with(|flag| {
        let prev = flag.get();
        flag.set(true);
        func(arg);
        flag.set(prev);
    });
}

fn ensure_worker_running() {
    let should_spawn = {
        let mut state = JIT_CONCURRENCY.lock().unwrap();
        if state.worker_running {
            false
        } else {
            state.worker_running = true;
            true
        }
    };

    if should_spawn {
        std::thread::spawn(|| {
            loop {
                let task = {
                    let mut state = JIT_CONCURRENCY.lock().unwrap();
                    state.tasks.pop_front()
                };

                match task {
                    Some(task) => run_task(task.func, task.arg as *mut std::ffi::c_void),
                    None => {
                        let mut state = JIT_CONCURRENCY.lock().unwrap();
                        state.worker_running = false;
                        break;
                    }
                }
            }
        });
    }
}

#[inline(always)]
fn tagged_to_i64(val: i64) -> i64 {
    if (val & 1) == 0 {
        val >> 1
    } else {
        // BigInt pointer with tag bit set
        let ptr = (val & !1) as *mut std::ffi::c_void;
        vp_bigint_to_i64_stub(ptr as *mut _)
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
    id: i64,
    state: AtomicI64,        // 0=PENDING, 1=READY, 2=RUNNING, 3=COMPLETED, 4=ERROR
    result: AtomicI64,
    callback: *mut std::ffi::c_void,
    callback_arg: *mut std::ffi::c_void,
    waiting_fiber: *mut std::ffi::c_void,
}

pub extern "C" fn vp_future_await(future: *mut std::ffi::c_void) -> i64 {
    // In JIT mode, cooperatively run queued tasks while waiting.
    if future.is_null() {
        return 0;
    }
    
    unsafe {
        let fut = &*(future as *mut JitFuture);
        
        // Wait until future is completed (state == 3) or error (state == 4)
        while {
            let state = fut.state.load(Ordering::Acquire);
            state != 3 && state != 4
        } {
            let task = {
                let mut state = JIT_CONCURRENCY.lock().unwrap();
                state.tasks.pop_front()
            };
            if let Some(task) = task {
                run_task(task.func, task.arg as *mut std::ffi::c_void);
            } else {
                std::thread::yield_now();
            }
        }
        
        fut.result.load(Ordering::Acquire)
    }
}

pub extern "C" fn vp_future_set_result(future: *mut std::ffi::c_void, result: i64) {
    if future.is_null() {
        return;
    }
    
    unsafe {
        let fut = &mut *(future as *mut JitFuture);
        fut.result.store(result, Ordering::Release);
        fut.state.store(3, Ordering::Release); // COMPLETED
    }
}

pub extern "C" fn vp_future_create() -> *mut std::ffi::c_void {
    extern "C" {
        fn vp_future_create() -> *mut std::ffi::c_void;
    }
    unsafe { vp_future_create() }
}

pub extern "C" fn vp_future_retain(future: *mut std::ffi::c_void) {
    extern "C" {
        fn vp_future_retain(f: *mut std::ffi::c_void);
    }
    unsafe { vp_future_retain(future) }
}

pub extern "C" fn vp_future_release(future: *mut std::ffi::c_void) {
    extern "C" {
        fn vp_future_release(f: *mut std::ffi::c_void);
    }
    unsafe { vp_future_release(future) }
}

pub extern "C" fn vp_future_await_and_release(future: *mut std::ffi::c_void) -> i64 {
    let result = vp_future_await(future);
    vp_future_release(future);
    result
}

// Async range for "async for i in async_range(n)"
pub extern "C" fn vp_async_range_create(start: i64, end: i64, step: i64) -> *mut std::ffi::c_void {
    extern "C" {
        fn vp_async_range_create(start: i64, end: i64, step: i64) -> *mut std::ffi::c_void;
    }
    unsafe { vp_async_range_create(start, end, step) }
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
    extern "C" {
        fn vp_async_range_free(ptr: *mut std::ffi::c_void);
    }
    unsafe { vp_async_range_free(range_ptr) }
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
    // If we're already executing an async task, enqueue for concurrent execution.
    // Otherwise, run immediately to preserve top-level semantics.
    let in_async = IN_ASYNC_EXEC.with(|flag| flag.get());
    if in_async {
        {
            let mut state = JIT_CONCURRENCY.lock().unwrap();
            state.tasks.push_back(JitTask { func, arg: arg as usize });
        }
        ensure_worker_running();
    } else {
        run_task(func, arg);
    }
    0
}

pub extern "C" fn vp_async_run_loop() {
    loop {
        let task = {
            let mut state = JIT_CONCURRENCY.lock().unwrap();
            state.tasks.pop_front()
        };
        match task {
            Some(task) => run_task(task.func, task.arg as *mut std::ffi::c_void),
            None => break,
        }
    }
}

pub extern "C" fn vp_async_sleep(milliseconds: i64) -> i64 {
    // For JIT mode, create a future and enqueue a sleep task.
    let future_ptr = vp_future_create();
    let ms = if milliseconds < 0 { 0 } else { milliseconds } as u64;
    let sleep_task = Box::new(SleepTask { future: future_ptr, ms });
    let task = JitTask {
        func: run_sleep_task,
        arg: Box::into_raw(sleep_task) as usize,
    };
    {
        let mut state = JIT_CONCURRENCY.lock().unwrap();
        state.tasks.push_back(task);
    }
    ensure_worker_running();
    future_ptr as i64
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
            let future = f as *mut std::ffi::c_void;
            if !future.is_null() {
                results.push(vp_future_await_and_release(future));
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

// Internal struct for async range (must match runtime/src/async.c)
#[repr(C)]
struct JitAsyncRange {
    magic: u64,
    start: i64,
    end: i64,
    step: i64,
    current: i64,
}

const JIT_ASYNC_RANGE_MAGIC: u64 = 0x5650525F41524E47u64; // "VPR_ARNG"
