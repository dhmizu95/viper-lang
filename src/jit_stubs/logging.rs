// Logging module stubs for JIT - Phase 4
// Thread-safe logger

use std::collections::HashMap;
use std::io::{stderr, Write};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref LOGGERS: Mutex<HashMap<String, LoggerConfig>> = Mutex::new(HashMap::new());
}

struct LoggerConfig {
    level: i64,
}

#[no_mangle]
pub extern "C" fn vp_logging_create_logger(name: *const i8, level: i64) -> *mut std::ffi::c_void {
    if name.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(name);
        let name_str = match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        };

        let mut loggers = LOGGERS.lock().unwrap();
        loggers.insert(name_str.clone(), LoggerConfig { level });

        // Return the name as identifier
        std::ffi::CString::new(name_str).unwrap().into_raw() as *mut std::ffi::c_void
    }
}

#[no_mangle]
pub extern "C" fn vp_logging_logger_free(logger: *mut std::ffi::c_void) {
    if !logger.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(logger as *mut i8));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_logging_set_level(logger: *mut std::ffi::c_void, level: i64) {
    if logger.is_null() {
        return;
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(logger as *mut i8);
        let name_str = match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return,
        };

        let mut loggers = LOGGERS.lock().unwrap();
        if let Some(config) = loggers.get_mut(&name_str) {
            config.level = level;
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_logging_get_level(logger: *mut std::ffi::c_void) -> i64 {
    if logger.is_null() {
        return 5; // NOTSET
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(logger as *mut i8);
        let name_str = match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return 5,
        };

        let loggers = LOGGERS.lock().unwrap();
        loggers.get(&name_str).map(|c| c.level).unwrap_or(5)
    }
}

#[no_mangle]
pub extern "C" fn vp_logging_enabled_for(logger: *mut std::ffi::c_void, level: i64) -> i64 {
    if logger.is_null() {
        return 0;
    }

    let logger_level = vp_logging_get_level(logger);
    if level >= logger_level {
        1
    } else {
        0
    }
}

fn log_message(logger: *mut std::ffi::c_void, level: i64, level_str: &str, message: *const i8) {
    if logger.is_null() || message.is_null() {
        return;
    }

    if vp_logging_enabled_for(logger, level) == 0 {
        return;
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(logger as *mut i8);
        let name_str = match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return,
        };

        let msg_str = match std::ffi::CStr::from_ptr(message).to_str() {
            Ok(s) => s,
            Err(_) => return,
        };

        let timestamp = chrono_lite_timestamp();
        let log_line = format!("{} - {} - {} - {}\n", timestamp, level_str, name_str, msg_str);

        let _ = stderr().write_all(log_line.as_bytes());
        let _ = stderr().flush();
    }
}

fn chrono_lite_timestamp() -> String {
    // Simple timestamp without external dependency
    let now =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    format!("{}", secs)
}

#[no_mangle]
pub extern "C" fn vp_logging_debug(logger: *mut std::ffi::c_void, message: *const i8) {
    log_message(logger, 0, "DEBUG", message);
}

#[no_mangle]
pub extern "C" fn vp_logging_info(logger: *mut std::ffi::c_void, message: *const i8) {
    log_message(logger, 1, "INFO", message);
}

#[no_mangle]
pub extern "C" fn vp_logging_warning(logger: *mut std::ffi::c_void, message: *const i8) {
    log_message(logger, 2, "WARNING", message);
}

#[no_mangle]
pub extern "C" fn vp_logging_error(logger: *mut std::ffi::c_void, message: *const i8) {
    log_message(logger, 3, "ERROR", message);
}

#[no_mangle]
pub extern "C" fn vp_logging_critical(logger: *mut std::ffi::c_void, message: *const i8) {
    log_message(logger, 4, "CRITICAL", message);
}

#[no_mangle]
pub extern "C" fn vp_logging_exception(logger: *mut std::ffi::c_void, message: *const i8) {
    log_message(logger, 3, "ERROR", message);
}

#[no_mangle]
pub extern "C" fn vp_logging_get_logger(name: *const i8) -> *mut std::ffi::c_void {
    if name.is_null() {
        return std::ffi::CString::new("root").unwrap().into_raw() as *mut std::ffi::c_void;
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(name);
        let name_str = match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => "root".to_string(),
        };

        // Check if logger exists, create if not
        {
            let loggers = LOGGERS.lock().unwrap();
            if loggers.contains_key(&name_str) {
                return std::ffi::CString::new(name_str).unwrap().into_raw()
                    as *mut std::ffi::c_void;
            }
        }

        // Create new logger with default level
        vp_logging_create_logger(name, 2); // WARNING level
        std::ffi::CString::new(name_str).unwrap().into_raw() as *mut std::ffi::c_void
    }
}

#[no_mangle]
pub extern "C" fn vp_logging_basic_config(
    level: i64,
    format: *const i8,
    stream: *mut std::ffi::c_void,
) {
    let _ = format;
    let _ = stream;

    // Set root logger level
    let root = std::ffi::CString::new("root").unwrap();
    vp_logging_set_level(root.as_ptr() as *mut std::ffi::c_void, level);
}

#[no_mangle]
pub extern "C" fn vp_logging_cleanup() {
    let mut loggers = LOGGERS.lock().unwrap();
    loggers.clear();
}

// Level constants
#[no_mangle]
pub extern "C" fn vp_logging_debug_level() -> i64 {
    0
}

#[no_mangle]
pub extern "C" fn vp_logging_info_level() -> i64 {
    1
}

#[no_mangle]
pub extern "C" fn vp_logging_warning_level() -> i64 {
    2
}

#[no_mangle]
pub extern "C" fn vp_logging_error_level() -> i64 {
    3
}

#[no_mangle]
pub extern "C" fn vp_logging_critical_level() -> i64 {
    4
}

#[no_mangle]
pub extern "C" fn vp_logging_notset_level() -> i64 {
    5
}

// Filter (simplified)
#[no_mangle]
pub extern "C" fn vp_logging_create_filter(name: *const i8) -> *mut std::ffi::c_void {
    if name.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let c_str = std::ffi::CStr::from_ptr(name);
        let name_str = match c_str.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return std::ptr::null_mut(),
        };

        std::ffi::CString::new(name_str).unwrap().into_raw() as *mut std::ffi::c_void
    }
}

#[no_mangle]
pub extern "C" fn vp_logging_filter_free(filter: *mut std::ffi::c_void) {
    if !filter.is_null() {
        unsafe {
            drop(std::ffi::CString::from_raw(filter as *mut i8));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_logging_filter_call(filter: *mut std::ffi::c_void, message: *const i8) -> i64 {
    // Simplified: always allow
    let _ = filter;
    let _ = message;
    1
}
