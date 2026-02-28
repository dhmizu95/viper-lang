// Asyncio module stubs for JIT - Phase 3
// Async I/O adapter

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

lazy_static::lazy_static! {
    static ref TASK_REGISTRY: Mutex<HashMap<i64, Arc<Mutex<TaskState>>>> = Mutex::new(HashMap::new());
    static ref TASK_COUNTER: Mutex<i64> = Mutex::new(0);
}

#[derive(Clone, Copy, PartialEq)]
enum TaskState {
    Pending,
    Running,
    Done,
    Cancelled,
}

fn get_next_task_id() -> i64 {
    let mut counter = TASK_COUNTER.lock().unwrap();
    *counter += 1;
    *counter
}

#[no_mangle]
pub extern "C" fn vp_asyncio_init() {
    // Initialize event loop
}

#[no_mangle]
pub extern "C" fn vp_asyncio_cleanup() {
    // Cleanup event loop
    TASK_REGISTRY.lock().unwrap().clear();
}

#[no_mangle]
pub extern "C" fn vp_asyncio_sleep(seconds: f64) {
    if seconds > 0.0 {
        thread::sleep(Duration::from_secs_f64(seconds));
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_create_task(coro: *mut std::ffi::c_void) -> i64 {
    let _ = coro;
    let id = get_next_task_id();
    TASK_REGISTRY.lock().unwrap().insert(id, Arc::new(Mutex::new(TaskState::Pending)));
    id
}

#[no_mangle]
pub extern "C" fn vp_asyncio_task_done(task_id: i64) -> i64 {
    let registry = TASK_REGISTRY.lock().unwrap();
    if let Some(state_arc) = registry.get(&task_id) {
        let state = state_arc.lock().unwrap();
        if *state == TaskState::Done {
            1
        } else {
            0
        }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_task_cancelled(task_id: i64) -> i64 {
    let registry = TASK_REGISTRY.lock().unwrap();
    if let Some(state_arc) = registry.get(&task_id) {
        let state = state_arc.lock().unwrap();
        if *state == TaskState::Cancelled {
            1
        } else {
            0
        }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_task_cancel(task_id: i64) -> i64 {
    let registry = TASK_REGISTRY.lock().unwrap();
    if let Some(state_arc) = registry.get(&task_id) {
        let mut state = state_arc.lock().unwrap();
        *state = TaskState::Cancelled;
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_gather(tasks: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    // Simplified: return empty list
    let _ = tasks;
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_asyncio_wait(tasks: *mut std::ffi::c_void, timeout: f64) -> i64 {
    // Simplified: wait for first task
    let _ = tasks;
    let _ = timeout;
    0
}

#[no_mangle]
pub extern "C" fn vp_asyncio_run(main_coro: *mut std::ffi::c_void) -> i64 {
    // Simplified: just run the coroutine
    let _ = main_coro;
    0
}

#[no_mangle]
pub extern "C" fn vp_asyncio_stop() {
    // Stop event loop
}

// Lock
#[no_mangle]
pub extern "C" fn vp_asyncio_lock_create() -> *mut std::ffi::c_void {
    let lock = Box::new(Mutex::new(false));
    Box::into_raw(lock) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn vp_asyncio_lock_free(lock: *mut std::ffi::c_void) {
    if !lock.is_null() {
        unsafe {
            drop(Box::from_raw(lock as *mut Mutex<bool>));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_lock_acquire(lock: *mut std::ffi::c_void) -> i64 {
    if lock.is_null() {
        return -1;
    }
    unsafe {
        let mutex = &*(lock as *mut Mutex<bool>);
        let mut locked = mutex.lock().unwrap();
        if !*locked {
            *locked = true;
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_lock_release(lock: *mut std::ffi::c_void) {
    if lock.is_null() {
        return;
    }
    unsafe {
        let mutex = &*(lock as *mut Mutex<bool>);
        let mut locked = mutex.lock().unwrap();
        *locked = false;
    }
}

// Event
#[no_mangle]
pub extern "C" fn vp_asyncio_event_create() -> *mut std::ffi::c_void {
    let event = Box::new(Mutex::new(false));
    Box::into_raw(event) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn vp_asyncio_event_free(event: *mut std::ffi::c_void) {
    if !event.is_null() {
        unsafe {
            drop(Box::from_raw(event as *mut Mutex<bool>));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_event_is_set(event: *mut std::ffi::c_void) -> i64 {
    if event.is_null() {
        return 0;
    }
    unsafe {
        let mutex = &*(event as *mut Mutex<bool>);
        let set = mutex.lock().unwrap();
        if *set {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_event_set(event: *mut std::ffi::c_void) {
    if event.is_null() {
        return;
    }
    unsafe {
        let mutex = &*(event as *mut Mutex<bool>);
        let mut set = mutex.lock().unwrap();
        *set = true;
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_event_clear(event: *mut std::ffi::c_void) {
    if event.is_null() {
        return;
    }
    unsafe {
        let mutex = &*(event as *mut Mutex<bool>);
        let mut set = mutex.lock().unwrap();
        *set = false;
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_event_wait(event: *mut std::ffi::c_void, timeout: f64) -> i64 {
    if event.is_null() {
        return 0;
    }
    unsafe {
        let mutex = &*(event as *mut Mutex<bool>);
        let set = mutex.lock().unwrap();
        if *set {
            1
        } else if timeout > 0.0 {
            drop(set);
            thread::sleep(Duration::from_secs_f64(timeout));
            0
        } else {
            0
        }
    }
}

// Queue
#[no_mangle]
pub extern "C" fn vp_asyncio_queue_create(maxsize: i64) -> *mut std::ffi::c_void {
    let _ = maxsize;
    let queue = Box::new(Mutex::new(Vec::<i64>::new()));
    Box::into_raw(queue) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn vp_asyncio_queue_free(queue: *mut std::ffi::c_void) {
    if !queue.is_null() {
        unsafe {
            drop(Box::from_raw(queue as *mut Mutex<Vec<i64>>));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_queue_size(queue: *mut std::ffi::c_void) -> i64 {
    if queue.is_null() {
        return 0;
    }
    unsafe {
        let q = &*(queue as *mut Mutex<Vec<i64>>);
        q.lock().unwrap().len() as i64
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_queue_empty(queue: *mut std::ffi::c_void) -> i64 {
    if vp_asyncio_queue_size(queue) == 0 {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_queue_full(queue: *mut std::ffi::c_void) -> i64 {
    // Simplified: never full
    let _ = queue;
    0
}

#[no_mangle]
pub extern "C" fn vp_asyncio_queue_put(queue: *mut std::ffi::c_void, item: i64) {
    if queue.is_null() {
        return;
    }
    unsafe {
        let q = &*(queue as *mut Mutex<Vec<i64>>);
        q.lock().unwrap().push(item);
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_queue_get(queue: *mut std::ffi::c_void) -> i64 {
    if queue.is_null() {
        return 0;
    }
    unsafe {
        let q = &*(queue as *mut Mutex<Vec<i64>>);
        let mut vec = q.lock().unwrap();
        if vec.is_empty() {
            0
        } else {
            vec.remove(0)
        }
    }
}

// Semaphore
#[no_mangle]
pub extern "C" fn vp_asyncio_semaphore_create(value: i64) -> *mut std::ffi::c_void {
    let sem = Box::new(Mutex::new(value));
    Box::into_raw(sem) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn vp_asyncio_semaphore_free(sem: *mut std::ffi::c_void) {
    if !sem.is_null() {
        unsafe {
            drop(Box::from_raw(sem as *mut Mutex<i64>));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_semaphore_acquire(sem: *mut std::ffi::c_void) -> i64 {
    if sem.is_null() {
        return -1;
    }
    unsafe {
        let s = &*(sem as *mut Mutex<i64>);
        let mut val = s.lock().unwrap();
        if *val > 0 {
            *val -= 1;
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_semaphore_release(sem: *mut std::ffi::c_void) {
    if sem.is_null() {
        return;
    }
    unsafe {
        let s = &*(sem as *mut Mutex<i64>);
        let mut val = s.lock().unwrap();
        *val += 1;
    }
}

// Timeout
#[no_mangle]
pub extern "C" fn vp_asyncio_timeout_create(seconds: f64) -> *mut std::ffi::c_void {
    let deadline = Instant::now() + Duration::from_secs_f64(seconds);
    let timeout = Box::new(Mutex::new(deadline));
    Box::into_raw(timeout) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn vp_asyncio_timeout_free(timeout: *mut std::ffi::c_void) {
    if !timeout.is_null() {
        unsafe {
            drop(Box::from_raw(timeout as *mut Mutex<Instant>));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_asyncio_timeout_expired(timeout: *mut std::ffi::c_void) -> i64 {
    if timeout.is_null() {
        return 0;
    }
    unsafe {
        let t = &*(timeout as *mut Mutex<Instant>);
        let deadline = t.lock().unwrap();
        if Instant::now() >= *deadline {
            1
        } else {
            0
        }
    }
}
