//! String JIT stubs - work with ViperString* to match runtime

use crate::jit_stubs::io::ViperString;
use std::alloc;

/// String concatenation stub for JIT
/// Takes two ViperString* and returns a new ViperString*
pub extern "C" fn vp_str_concat_stub(a: *mut ViperString, b: *mut ViperString) -> *mut ViperString {
    if a.is_null() && b.is_null() {
        return std::ptr::null_mut();
    }
    if a.is_null() {
        return b;
    }
    if b.is_null() {
        return a;
    }

    unsafe {
        let str_a = get_string_data(a);
        let str_b = get_string_data(b);
        let concatenated = format!("{}{}", str_a, str_b);
        create_viper_string(&concatenated)
    }
}

/// String repetition stub for JIT
pub extern "C" fn vp_str_repeat_stub(s: *mut ViperString, count: i64) -> *mut ViperString {
    if s.is_null() || count <= 0 {
        return create_viper_string("");
    }

    unsafe {
        let str = get_string_data(s);
        let repeated = str.repeat(count as usize);
        create_viper_string(&repeated)
    }
}

/// Convert i64 to string stub for JIT
pub extern "C" fn vp_str_from_i64_stub(val: i64) -> *mut ViperString {
    let s = val.to_string();
    create_viper_string(&s)
}

/// Convert f64 to string stub for JIT
pub extern "C" fn vp_str_from_f64_stub(val: f64) -> *mut ViperString {
    let s = val.to_string();
    create_viper_string(&s)
}

/// Convert bool to string stub for JIT
pub extern "C" fn vp_str_from_bool_stub(val: bool) -> *mut ViperString {
    let s = if val { "True" } else { "False" };
    create_viper_string(s)
}

/// Get string length stub for JIT
pub extern "C" fn vp_str_len_stub(s: *mut ViperString) -> i64 {
    if s.is_null() {
        return 0;
    }
    unsafe { (*s).length() }
}

pub extern "C" fn vp_str_create_stub(s: *const std::ffi::c_char) -> *mut ViperString {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s);
        if let Ok(rust_str) = c_str.to_str() {
            create_viper_string(rust_str)
        } else {
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn vp_str_upper_stub(s: *mut ViperString) -> *mut ViperString {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let str = get_string_data(s);
        create_viper_string(&str.to_uppercase())
    }
}

pub extern "C" fn vp_str_lower_stub(s: *mut ViperString) -> *mut ViperString {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let str = get_string_data(s);
        create_viper_string(&str.to_lowercase())
    }
}

pub extern "C" fn vp_str_split_stub(
    s: *mut ViperString,
    delim: *mut ViperString,
) -> *mut std::ffi::c_void {
    use std::ffi::c_void;

    let list = Box::new(Vec::<i64>::new());
    if s.is_null() || delim.is_null() {
        return Box::into_raw(list) as *mut c_void;
    }
    unsafe {
        let str = get_string_data(s);
        let delim_str = get_string_data(delim);
        let mut list_val = Vec::<i64>::new();
        for part in str.split(&*delim_str) {
            list_val.push(create_viper_string(part) as i64);
        }
        let boxed = Box::new(list_val);
        Box::into_raw(boxed) as *mut c_void
    }
}

pub extern "C" fn vp_str_replace_stub(
    s: *mut ViperString,
    old_sub: *mut ViperString,
    new_sub: *mut ViperString,
) -> *mut ViperString {
    if s.is_null() || old_sub.is_null() || new_sub.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let str = get_string_data(s);
        let old_str = get_string_data(old_sub);
        let new_str = get_string_data(new_sub);
        let replaced = str.replace(&*old_str, &*new_str);
        create_viper_string(&replaced)
    }
}

/// String equality comparison stub for JIT
pub extern "C" fn vp_str_equals_stub(a: *mut ViperString, b: *mut ViperString) -> bool {
    if a.is_null() && b.is_null() {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }
    unsafe {
        let str_a = get_string_data(a);
        let str_b = get_string_data(b);
        str_a == str_b
    }
}

/// String comparison stub for JIT
pub extern "C" fn vp_str_compare_stub(a: *mut ViperString, b: *mut ViperString) -> i64 {
    if a.is_null() && b.is_null() {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }
    unsafe {
        let str_a = get_string_data(a);
        let str_b = get_string_data(b);
        match str_a.cmp(&str_b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}

/// String format stub for JIT
pub extern "C" fn vp_str_format_stub(
    format_str: *mut ViperString,
    args_array: *const *mut ViperString,
    arg_count: i64,
) -> *mut ViperString {
    if format_str.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let fmt = get_string_data(format_str);
        let mut result = fmt.to_string();

        if !args_array.is_null() && arg_count > 0 {
            for i in 0..arg_count {
                let arg_ptr = *args_array.offset(i as isize);
                if !arg_ptr.is_null() {
                    let arg_str = get_string_data(arg_ptr);
                    
                    // Simple replacement of the first {...} we find
                    if let Some(start) = result.find('{') {
                        if let Some(end_offset) = result[start..].find('}') {
                            let end = start + end_offset;
                            let placeholder = &result[start..=end];
                            
                            let mut formatted = arg_str.clone();
                            if placeholder.contains(':') {
                                if placeholder.contains(".9f") {
                                    if let Ok(val) = arg_str.parse::<f64>() {
                                        formatted = format!("{:.9}", val);
                                    }
                                }
                            }
                            result.replace_range(start..=end, &formatted);
                        }
                    }
                }
            }
        }

        create_viper_string(&result)
    }
}

// Helper function to extract string data from ViperString
unsafe fn get_string_data(s: *mut ViperString) -> String {
    if s.is_null() {
        return String::new();
    }
    let viper_str = &*s;
    let slice = viper_str.as_slice();
    String::from_utf8_lossy(slice).into_owned()
}

// Helper function to create a new ViperString
pub fn create_viper_string(s: &str) -> *mut ViperString {
    extern "C" {
        fn vp_str_create_with_len(s: *const std::ffi::c_char, len: i64) -> *mut ViperString;
    }
    
    let len = s.len() as i64;
    let c_str = std::ffi::CString::new(s).unwrap_or_else(|_| std::ffi::CString::new("").unwrap());
    
    unsafe { vp_str_create_with_len(c_str.as_ptr(), len) }
}

/// Get first character of a string as i64 (byte value)
/// Used for string indexing comparison: ch == "a" where ch = text[i]
pub extern "C" fn vp_str_get_first_stub(s: *mut ViperString) -> i64 {
    if s.is_null() {
        return 0;
    }
    unsafe {
        let slice = (*s).as_slice();
        if slice.is_empty() {
            0
        } else {
            slice[0] as i64
        }
    }
}

/// Exit stub for JIT - just exits the process
pub extern "C" fn vp_exit_stub(code: i64) {
    std::process::exit(code as i32);
}

// =============================================================================
// Bytes JIT Stubs
// =============================================================================

/// ViperBytes structure (must match runtime/include/viper_stdlib.h)
#[repr(C)]
pub struct ViperBytes {
    data: *mut u8,
    len: i64,
}

/// Create bytes from raw data
pub extern "C" fn vp_bytes_create_stub(data: *const u8, len: i64) -> *mut ViperBytes {
    extern "C" {
        fn vp_bytes_create(data: *const u8, len: i64) -> *mut ViperBytes;
    }
    unsafe { vp_bytes_create(data, len) }
}

/// Free bytes
pub extern "C" fn vp_bytes_free_stub(bytes: *mut ViperBytes) {
    if bytes.is_null() {
        return;
    }

    unsafe {
        extern "C" {
             fn vp_arc_release(ptr: *mut std::ffi::c_void);
        }
        vp_arc_release(bytes as *mut std::ffi::c_void);
    }
}

/// Get bytes length
pub extern "C" fn vp_bytes_len_stub(bytes: *mut ViperBytes) -> i64 {
    if bytes.is_null() {
        return 0;
    }
    unsafe { (*bytes).len }
}

/// Get byte at index
pub extern "C" fn vp_bytes_get_stub(bytes: *mut ViperBytes, index: i64) -> u8 {
    if bytes.is_null() || index < 0 || index >= unsafe { (*bytes).len } {
        return 0;
    }
    unsafe { *(*bytes).data.offset(index as isize) }
}

/// Print bytes
pub extern "C" fn vp_bytes_print_stub(bytes: *mut ViperBytes) {
    use std::io::{self, Write};

    if bytes.is_null() {
        print!("None");
        io::stdout().flush().unwrap();
        return;
    }

    unsafe {
        print!("b\"");
        let b = &*bytes;
        if !b.data.is_null() && b.len > 0 {
            for i in 0..b.len as usize {
                let byte = *b.data.add(i);
                if byte >= 32 && byte < 127 {
                    if byte == b'"' || byte == b'\\' {
                        print!("\\");
                    }
                    print!("{}", byte as char);
                } else {
                    print!("\\x{:02x}", byte);
                }
            }
        }
        print!("\"");
        io::stdout().flush().unwrap();
    }
}
