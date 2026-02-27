// Socket module stubs for JIT - Phase 3
// POSIX socket wrappers

use std::net::{TcpListener, TcpStream, UdpSocket};
use std::io::{Read, Write};
use std::collections::HashMap;
use std::sync::{Mutex, Arc};

lazy_static::lazy_static! {
    static ref SOCKET_REGISTRY: Mutex<HashMap<i64, Arc<Mutex<SocketHandle>>>> = Mutex::new(HashMap::new());
    static ref SOCKET_COUNTER: Mutex<i64> = Mutex::new(0);
}

enum SocketHandle {
    TcpListener(TcpListener),
    TcpStream(TcpStream),
    Udp(UdpSocket),
    None,  // Placeholder before connect
}

fn get_next_socket_id() -> i64 {
    let mut counter = SOCKET_COUNTER.lock().unwrap();
    *counter += 1;
    *counter
}

#[no_mangle]
pub extern "C" fn vp_socket_create(family: i64, sock_type: i64, protocol: i64) -> i64 {
    // Simplified: only support TCP for now
    let _ = family;
    let _ = protocol;
    
    if sock_type == 1 {  // SOCK_STREAM
        let id = get_next_socket_id();
        // Placeholder - actual socket created on connect
        SOCKET_REGISTRY.lock().unwrap().insert(id, Arc::new(Mutex::new(SocketHandle::None)));
        id
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn vp_socket_connect(sock_id: i64, host: *const i8, port: i64) -> i64 {
    if host.is_null() {
        return -1;
    }
    
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(host);
        let host_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        let addr = format!("{}:{}", host_str, port);
        match TcpStream::connect(&addr) {
            Ok(stream) => {
                let mut registry = SOCKET_REGISTRY.lock().unwrap();
                registry.insert(sock_id, Arc::new(Mutex::new(SocketHandle::TcpStream(stream))));
                0
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_socket_send(sock_id: i64, data: *const i8, len: i64) -> i64 {
    if data.is_null() || len <= 0 {
        return -1;
    }
    
    let registry = SOCKET_REGISTRY.lock().unwrap();
    if let Some(handle_arc) = registry.get(&sock_id) {
        let mut handle = handle_arc.lock().unwrap();
        if let SocketHandle::TcpStream(stream) = &mut *handle {
            unsafe {
                let bytes = std::slice::from_raw_parts(data as *const u8, len as usize);
                match stream.write(bytes) {
                    Ok(n) => n as i64,
                    Err(_) => -1,
                }
            }
        } else {
            -1  // Socket not connected
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn vp_socket_recv(sock_id: i64, buffer: *mut i8, maxlen: i64) -> i64 {
    if buffer.is_null() || maxlen <= 0 {
        return -1;
    }
    
    let registry = SOCKET_REGISTRY.lock().unwrap();
    if let Some(handle_arc) = registry.get(&sock_id) {
        let mut handle = handle_arc.lock().unwrap();
        if let SocketHandle::TcpStream(stream) = &mut *handle {
            unsafe {
                let bytes = std::slice::from_raw_parts_mut(buffer as *mut u8, maxlen as usize);
                match stream.read(bytes) {
                    Ok(n) => n as i64,
                    Err(_) => -1,
                }
            }
        } else {
            -1  // Socket not connected
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn vp_socket_close(sock_id: i64) -> i64 {
    let mut registry = SOCKET_REGISTRY.lock().unwrap();
    registry.remove(&sock_id);
    0
}

#[no_mangle]
pub extern "C" fn vp_socket_bind(sock_id: i64, host: *const i8, port: i64) -> i64 {
    if host.is_null() {
        return -1;
    }
    
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(host);
        let host_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        
        let addr = format!("{}:{}", host_str, port);
        match TcpListener::bind(&addr) {
            Ok(listener) => {
                let mut registry = SOCKET_REGISTRY.lock().unwrap();
                registry.insert(sock_id, Arc::new(Mutex::new(SocketHandle::TcpListener(listener))));
                0
            }
            Err(_) => -1,
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_socket_listen(sock_id: i64, backlog: i64) -> i64 {
    // TCP listener is already listening after bind
    let _ = backlog;
    let registry = SOCKET_REGISTRY.lock().unwrap();
    if registry.contains_key(&sock_id) {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn vp_socket_accept(sock_id: i64) -> i64 {
    let registry = SOCKET_REGISTRY.lock().unwrap();
    if let Some(handle_arc) = registry.get(&sock_id) {
        let handle = handle_arc.lock().unwrap();
        if let SocketHandle::TcpListener(listener) = &*handle {
            match listener.accept() {
                Ok((stream, _)) => {
                    let new_id = get_next_socket_id();
                    drop(handle);
                    drop(registry);
                    SOCKET_REGISTRY.lock().unwrap().insert(
                        new_id, 
                        Arc::new(Mutex::new(SocketHandle::TcpStream(stream)))
                    );
                    new_id
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn vp_socket_setblocking(sock_id: i64, blocking: i64) -> i64 {
    // Simplified: non-blocking not fully implemented
    let _ = sock_id;
    let _ = blocking;
    0
}

#[no_mangle]
pub extern "C" fn vp_socket_getsockopt(
    sock_id: i64,
    level: i64,
    optname: i64,
    value: *mut i8,
    len: *mut i64,
) -> i64 {
    // Simplified: return dummy values
    let _ = sock_id;
    let _ = level;
    let _ = optname;
    if !value.is_null() && !len.is_null() {
        unsafe {
            *value = 0;
            *len = 1;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn vp_socket_setsockopt(
    sock_id: i64,
    level: i64,
    optname: i64,
    value: *const i8,
    len: i64,
) -> i64 {
    // Simplified: accept all options
    let _ = sock_id;
    let _ = level;
    let _ = optname;
    let _ = value;
    let _ = len;
    0
}

#[no_mangle]
pub extern "C" fn vp_socket_fileno(sock_id: i64) -> i64 {
    // Return socket ID as file descriptor placeholder
    sock_id
}

// Constants
#[no_mangle]
pub extern "C" fn vp_socket_af_inet() -> i64 { 2 }

#[no_mangle]
pub extern "C" fn vp_socket_af_inet6() -> i64 { 10 }

#[no_mangle]
pub extern "C" fn vp_socket_sock_stream() -> i64 { 1 }

#[no_mangle]
pub extern "C" fn vp_socket_sock_dgram() -> i64 { 2 }

#[no_mangle]
pub extern "C" fn vp_socket_sol_socket() -> i64 { 1 }

#[no_mangle]
pub extern "C" fn vp_socket_so_reuseaddr() -> i64 { 2 }

#[no_mangle]
pub extern "C" fn vp_socket_tcp_nodelay() -> i64 { 1 }

#[no_mangle]
pub extern "C" fn vp_socket_shut_rd() -> i64 { 0 }

#[no_mangle]
pub extern "C" fn vp_socket_shut_wr() -> i64 { 1 }

#[no_mangle]
pub extern "C" fn vp_socket_shut_rdwr() -> i64 { 2 }
