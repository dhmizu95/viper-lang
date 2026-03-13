//! Exception handling JIT stubs for Viper
//!
//! These functions provide the runtime implementation for Viper's
//! exception handling mechanism.

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::ptr;

/// Exception information structure
#[derive(Debug, Clone)]
pub struct ExceptionInfo {
    pub exception_type: String,
    pub message: String,
    pub code: i64,
    /// The cause exception for exception chaining (raise X from Y)
    pub cause: Option<Box<ExceptionInfo>>,
}

impl ExceptionInfo {
    pub fn new(exception_type: &str, message: &str, code: i64) -> Self {
        Self {
            exception_type: exception_type.to_string(),
            message: message.to_string(),
            code,
            cause: None,
        }
    }

    pub fn with_cause(
        exception_type: &str,
        message: &str,
        code: i64,
        cause: ExceptionInfo,
    ) -> Self {
        Self {
            exception_type: exception_type.to_string(),
            message: message.to_string(),
            code,
            cause: Some(Box::new(cause)),
        }
    }
}

impl Default for ExceptionInfo {
    fn default() -> Self {
        Self { exception_type: String::new(), message: String::new(), code: 0, cause: None }
    }
}

// Thread-local storage for current exception
thread_local! {
    static CURRENT_EXCEPTION: RefCell<Option<ExceptionInfo>> = RefCell::new(None);
}

/// Helper to get string from C pointer
fn cstr_to_str(ptr: *const i8) -> Option<&'static str> {
    if ptr.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(ptr).to_str().ok() }
    }
}

/// Raise a Viper exception with type and message
#[no_mangle]
pub extern "C" fn viper_raise_exception(exc_type: *const i8, message: *const i8) {
    let type_str = cstr_to_str(exc_type).unwrap_or("Exception");
    let msg_str = cstr_to_str(message).unwrap_or("");

    let exc = ExceptionInfo::new(type_str, msg_str, 0);

    CURRENT_EXCEPTION.with(|e| {
        *e.borrow_mut() = Some(exc);
    });

    eprintln!("{}: {}", type_str, msg_str);
    std::process::exit(1);
}

/// Raise a Viper exception with error code
#[no_mangle]
pub extern "C" fn viper_raise_with_code(exc_type: *const i8, message: *const i8, code: i64) {
    let type_str = cstr_to_str(exc_type).unwrap_or("Exception");
    let msg_str = cstr_to_str(message).unwrap_or("");

    let exc = ExceptionInfo::new(type_str, msg_str, code);

    CURRENT_EXCEPTION.with(|e| {
        *e.borrow_mut() = Some(exc);
    });

    eprintln!("{} [{}]: {}", type_str, code, msg_str);
    std::process::exit(1);
}

/// Raise a Viper exception with a cause (exception chaining: raise X from Y)
#[no_mangle]
pub extern "C" fn viper_raise_with_cause(
    exc_type: *const i8,
    message: *const i8,
    code: i64,
    cause_type: *const i8,
    cause_message: *const i8,
) {
    let type_str = cstr_to_str(exc_type).unwrap_or("Exception");
    let msg_str = cstr_to_str(message).unwrap_or("");
    let cause_type_str = cstr_to_str(cause_type).unwrap_or("Exception");
    let cause_msg_str = cstr_to_str(cause_message).unwrap_or("");

    // Create the cause exception
    let cause_exc = ExceptionInfo::new(cause_type_str, cause_msg_str, 0);
    // Create the main exception with the cause
    let exc = ExceptionInfo::with_cause(type_str, msg_str, code, cause_exc);

    CURRENT_EXCEPTION.with(|e| {
        *e.borrow_mut() = Some(exc);
    });

    // Print exception chain
    eprintln!("{}: {}", type_str, msg_str);
    eprintln!("\nThe above exception was the direct cause of the following exception:");
    eprintln!("{}: {}", cause_type_str, cause_msg_str);
    std::process::exit(1);
}

/// Check if current exception matches the given type
#[no_mangle]
pub extern "C" fn viper_catch_exception(exc_type: *const i8) -> i8 {
    let type_str = match cstr_to_str(exc_type) {
        Some(s) => s,
        None => return 0,
    };

    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        None => 0,
        Some(ref exc) => {
            if exc.exception_type == type_str {
                1
            } else if type_str == "Exception" {
                1
            } else {
                0
            }
        }
    })
}

/// Get the type of the current exception
#[no_mangle]
pub extern "C" fn viper_get_exception_type() -> *mut i8 {
    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        None => ptr::null_mut(),
        Some(ref exc) => CString::new(exc.exception_type.clone())
            .map(|c| c.into_raw())
            .unwrap_or(ptr::null_mut()),
    })
}

/// Get the message of the current exception
#[no_mangle]
pub extern "C" fn viper_get_exception_message() -> *mut i8 {
    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        None => ptr::null_mut(),
        Some(ref exc) => {
            CString::new(exc.message.clone()).map(|c| c.into_raw()).unwrap_or(ptr::null_mut())
        }
    })
}

/// Get the error code of the current exception
#[no_mangle]
pub extern "C" fn viper_get_exception_code() -> i64 {
    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        None => 0,
        Some(ref exc) => exc.code,
    })
}

/// Clear the current exception
#[no_mangle]
pub extern "C" fn viper_clear_exception() {
    CURRENT_EXCEPTION.with(|e| {
        *e.borrow_mut() = None;
    });
}

/// Set the current exception (for re-raising)
#[no_mangle]
pub extern "C" fn viper_set_exception(exc_type: *const i8, message: *const i8, code: i64) {
    let type_str = cstr_to_str(exc_type).unwrap_or("Exception");
    let msg_str = cstr_to_str(message).unwrap_or("");

    let exc = ExceptionInfo::new(type_str, msg_str, code);

    CURRENT_EXCEPTION.with(|e| {
        *e.borrow_mut() = Some(exc);
    });
}

/// Format exception info as string with cause chain (caller must free result)
#[no_mangle]
pub extern "C" fn viper_format_exception() -> *mut i8 {
    CURRENT_EXCEPTION.with(|e| {
        match *e.borrow() {
            None => ptr::null_mut(),
            Some(ref exc) => {
                let mut formatted = format!("{}: {}", exc.exception_type, exc.message);
                
                // Add cause chain if present
                let mut current_cause = &exc.cause;
                while let Some(cause) = current_cause {
                    formatted.push_str(&format!(
                        "\n\nThe above exception was the direct cause of the following exception:\n{}: {}",
                        cause.exception_type,
                        cause.message
                    ));
                    current_cause = &cause.cause;
                }
                
                CString::new(formatted)
                    .map(|c| c.into_raw())
                    .unwrap_or(ptr::null_mut())
            }
        }
    })
}

/// Get the cause exception type (for exception chaining)
#[no_mangle]
pub extern "C" fn viper_get_exception_cause_type() -> *mut i8 {
    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        None => ptr::null_mut(),
        Some(ref exc) => match &exc.cause {
            None => ptr::null_mut(),
            Some(cause) => CString::new(cause.exception_type.clone())
                .map(|c| c.into_raw())
                .unwrap_or(ptr::null_mut()),
        },
    })
}

/// Get the cause exception message (for exception chaining)
#[no_mangle]
pub extern "C" fn viper_get_exception_cause_message() -> *mut i8 {
    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        None => ptr::null_mut(),
        Some(ref exc) => match &exc.cause {
            None => ptr::null_mut(),
            Some(cause) => {
                CString::new(cause.message.clone()).map(|c| c.into_raw()).unwrap_or(ptr::null_mut())
            }
        },
    })
}

/// Print stack trace for current exception
#[no_mangle]
pub extern "C" fn viper_print_traceback() {
    eprintln!("Traceback (most recent call last):");
    eprintln!("  <stack trace not available>");
}

/// Check if exception type matches (supports inheritance)
#[no_mangle]
pub extern "C" fn viper_exception_matches(actual_type: *const i8, expected_type: *const i8) -> i8 {
    let actual = match cstr_to_str(actual_type) {
        Some(s) => s,
        None => return 0,
    };
    let expected = match cstr_to_str(expected_type) {
        Some(s) => s,
        None => return 0,
    };

    if actual == expected {
        return 1;
    }

    if expected == "Exception" {
        return 1;
    }

    0
}

/// Helper function to free a string allocated by the runtime
#[no_mangle]
pub extern "C" fn viper_free_string(s: *mut i8) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

/// Check if there is a current exception
#[no_mangle]
pub extern "C" fn viper_has_exception() -> i8 {
    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        Some(_) => 1,
        None => 0,
    })
}

/// Re-raise the current exception
#[no_mangle]
pub extern "C" fn viper_reraise_exception() {
    CURRENT_EXCEPTION.with(|e| match *e.borrow() {
        None => eprintln!("No active exception to re-raise"),
        Some(ref exc) => eprintln!("{}: {}", exc.exception_type, exc.message),
    });

    std::process::exit(1);
}

/// Get the exception info as a formatted string for display
/// Returns a pointer that must be freed with viper_free_string
#[no_mangle]
pub extern "C" fn viper_exception_to_string(
    exc_type: *const i8,
    message: *const i8,
    code: i64,
) -> *mut i8 {
    let type_str = cstr_to_str(exc_type).unwrap_or("Exception");
    let msg_str = cstr_to_str(message).unwrap_or("");

    let result = if code != 0 {
        format!("{} [{}]: {}", type_str, code, msg_str)
    } else {
        format!("{}: {}", type_str, msg_str)
    };

    CString::new(result).map(|c| c.into_raw()).unwrap_or(ptr::null_mut())
}
