// Select module stubs for JIT - Phase 3
// I/O multiplexing

use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref FD_REGISTRY: Mutex<HashMap<i64, FdInfo>> = Mutex::new(HashMap::new());
}

struct FdInfo {
    readable: bool,
    writable: bool,
    error: bool,
}

#[no_mangle]
pub extern "C" fn vp_select_fdset_create() -> *mut std::ffi::c_void {
    let fdset = Box::new(Vec::<i64>::new());
    Box::into_raw(fdset) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn vp_select_fdset_free(fdset: *mut std::ffi::c_void) {
    if !fdset.is_null() {
        unsafe {
            drop(Box::from_raw(fdset as *mut Vec<i64>));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_select_fdset_add(fdset: *mut std::ffi::c_void, fd: i64) {
    if fdset.is_null() || fd < 0 {
        return;
    }
    unsafe {
        let set = &mut *(fdset as *mut Vec<i64>);
        if !set.contains(&fd) {
            set.push(fd);
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_select_fdset_remove(fdset: *mut std::ffi::c_void, fd: i64) {
    if fdset.is_null() || fd < 0 {
        return;
    }
    unsafe {
        let set = &mut *(fdset as *mut Vec<i64>);
        if let Some(pos) = set.iter().position(|&f| f == fd) {
            set.remove(pos);
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_select_fdset_contains(fdset: *mut std::ffi::c_void, fd: i64) -> i64 {
    if fdset.is_null() || fd < 0 {
        return 0;
    }
    unsafe {
        let set = &*(fdset as *mut Vec<i64>);
        if set.contains(&fd) {
            1
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_select_fdset_clear(fdset: *mut std::ffi::c_void) {
    if fdset.is_null() {
        return;
    }
    unsafe {
        let set = &mut *(fdset as *mut Vec<i64>);
        set.clear();
    }
}

#[no_mangle]
pub extern "C" fn vp_select_fdset_get_fds(fdset: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    fdset
}

#[no_mangle]
pub extern "C" fn vp_select_select(
    read_fds: *mut std::ffi::c_void,
    write_fds: *mut std::ffi::c_void,
    error_fds: *mut std::ffi::c_void,
    timeout: f64,
) -> *mut std::ffi::c_void {
    // Simplified select implementation
    let _ = read_fds;
    let _ = write_fds;
    let _ = error_fds;
    let _ = timeout;

    // Return empty result
    let result = Box::new(SelectResult {
        readable: Vec::new(),
        writable: Vec::new(),
        error: Vec::new(),
        count: 0,
    });

    Box::into_raw(result) as *mut std::ffi::c_void
}

#[no_mangle]
pub extern "C" fn vp_select_result_free(result: *mut std::ffi::c_void) {
    if !result.is_null() {
        unsafe {
            drop(Box::from_raw(result as *mut SelectResult));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_select_can_read(fd: i64, timeout: f64) -> i64 {
    // Simplified: always return not ready
    let _ = fd;
    let _ = timeout;
    0
}

#[no_mangle]
pub extern "C" fn vp_select_can_write(fd: i64, timeout: f64) -> i64 {
    // Simplified: always return ready
    let _ = fd;
    let _ = timeout;
    1
}

#[no_mangle]
pub extern "C" fn vp_select_get_error() -> i64 {
    0
}

#[no_mangle]
pub extern "C" fn vp_select_strerror(err: i64) -> *mut i8 {
    let msg = format!("Error {}", err);
    std::ffi::CString::new(msg).unwrap().into_raw()
}

// Select result structure
struct SelectResult {
    readable: Vec<i64>,
    writable: Vec<i64>,
    error: Vec<i64>,
    count: i64,
}

// Poll (simplified)
#[no_mangle]
pub extern "C" fn vp_poll_poll(fds: *mut std::ffi::c_void, timeout: f64) -> *mut std::ffi::c_void {
    let _ = fds;
    let _ = timeout;
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_poll_result_free(result: *mut std::ffi::c_void) {
    let _ = result;
}

// Epoll (Linux-specific, simplified)
#[no_mangle]
pub extern "C" fn vp_epoll_create() -> i64 {
    -1 // Not implemented
}

#[no_mangle]
pub extern "C" fn vp_epoll_free(epfd: i64) {
    let _ = epfd;
}

#[no_mangle]
pub extern "C" fn vp_epoll_ctl(epfd: i64, op: i64, fd: i64, events: u32) -> i64 {
    let _ = epfd;
    let _ = op;
    let _ = fd;
    let _ = events;
    -1
}

#[no_mangle]
pub extern "C" fn vp_epoll_wait(epfd: i64, timeout_ms: i64) -> *mut std::ffi::c_void {
    let _ = epfd;
    let _ = timeout_ms;
    // Return empty list
    let list = Box::new(Vec::<i64>::new());
    Box::into_raw(list) as *mut std::ffi::c_void
}

// Epoll constants
#[no_mangle]
pub extern "C" fn vp_epollin() -> i64 {
    0x001
}

#[no_mangle]
pub extern "C" fn vp_epollout() -> i64 {
    0x004
}

#[no_mangle]
pub extern "C" fn vp_epollerr() -> i64 {
    0x008
}

#[no_mangle]
pub extern "C" fn vp_epollhup() -> i64 {
    0x010
}

#[no_mangle]
pub extern "C" fn vp_epollet() -> i64 {
    0x80000000
}

#[no_mangle]
pub extern "C" fn vp_epoll_ctl_add() -> i64 {
    1
}

#[no_mangle]
pub extern "C" fn vp_epoll_ctl_mod() -> i64 {
    2
}

#[no_mangle]
pub extern "C" fn vp_epoll_ctl_del() -> i64 {
    3
}
