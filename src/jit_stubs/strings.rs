/// String concatenation stub for JIT
/// Uses CString to ensure proper null-terminated string layout
pub extern "C" fn vp_str_concat_stub(
    a: *const std::ffi::c_char,
    b: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    use std::ffi::CStr;

    if a.is_null() || b.is_null() {
        return std::ptr::null();
    }

    unsafe {
        let str_a = CStr::from_ptr(a).to_string_lossy();
        let str_b = CStr::from_ptr(b).to_string_lossy();
        let concatenated = format!("{}{}", str_a, str_b);

        // Use CString to ensure proper null-terminated layout
        // Leak the CString to keep it alive for JIT execution
        let c_str = std::ffi::CString::new(concatenated).unwrap();
        c_str.into_raw()
    }
}

/// Convert i64 to string stub for JIT
pub extern "C" fn vp_str_from_i64_stub(val: i64) -> *const std::ffi::c_char {
    let s = val.to_string();
    let c_str = std::ffi::CString::new(s).unwrap();
    c_str.into_raw()
}

/// Convert f64 to string stub for JIT
pub extern "C" fn vp_str_from_f64_stub(val: f64) -> *const std::ffi::c_char {
    let s = val.to_string();
    let c_str = std::ffi::CString::new(s).unwrap();
    c_str.into_raw()
}

/// Get string length stub for JIT
pub extern "C" fn vp_str_len_stub(s: *const std::ffi::c_char) -> i64 {
    if s.is_null() {
        return 0;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s);
        c_str.to_str().map(|s| s.len() as i64).unwrap_or(0)
    }
}

pub extern "C" fn vp_str_create_stub(s: *const std::ffi::c_char) -> *const std::ffi::c_char {
    if s.is_null() {
        return std::ptr::null();
    }
    unsafe {
        let str = std::ffi::CStr::from_ptr(s).to_string_lossy();
        let c_str = std::ffi::CString::new(str.into_owned()).unwrap();
        c_str.into_raw()
    }
}

pub extern "C" fn vp_str_upper_stub(s: *const std::ffi::c_char) -> *const std::ffi::c_char {
    if s.is_null() {
        return std::ptr::null();
    }
    unsafe {
        let str = std::ffi::CStr::from_ptr(s).to_string_lossy();
        let upper = str.to_uppercase();
        let c_str = std::ffi::CString::new(upper).unwrap();
        c_str.into_raw()
    }
}

pub extern "C" fn vp_str_lower_stub(s: *const std::ffi::c_char) -> *const std::ffi::c_char {
    if s.is_null() {
        return std::ptr::null();
    }
    unsafe {
        let str = std::ffi::CStr::from_ptr(s).to_string_lossy();
        let lower = str.to_lowercase();
        let c_str = std::ffi::CString::new(lower).unwrap();
        c_str.into_raw()
    }
}

pub extern "C" fn vp_str_split_stub(
    s: *const std::ffi::c_char,
    delim_ptr: *const std::ffi::c_char,
) -> *mut std::ffi::c_void {
    let list = Box::new(Vec::<i64>::new());
    if s.is_null() || delim_ptr.is_null() {
        return Box::into_raw(list) as *mut std::ffi::c_void;
    }
    unsafe {
        let str = std::ffi::CStr::from_ptr(s).to_string_lossy();
        let delim = std::ffi::CStr::from_ptr(delim_ptr).to_string_lossy();
        let mut list_val = Vec::<i64>::new();
        for part in str.split(&*delim) {
            let c_str = std::ffi::CString::new(part).unwrap();
            list_val.push(c_str.into_raw() as i64);
        }
        let boxed = Box::new(list_val);
        Box::into_raw(boxed) as *mut std::ffi::c_void
    }
}

pub extern "C" fn vp_str_replace_stub(
    s: *const std::ffi::c_char,
    old_sub: *const std::ffi::c_char,
    new_sub: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    if s.is_null() || old_sub.is_null() || new_sub.is_null() {
        return std::ptr::null();
    }
    unsafe {
        let str = std::ffi::CStr::from_ptr(s).to_string_lossy();
        let old_str = std::ffi::CStr::from_ptr(old_sub).to_string_lossy();
        let new_str = std::ffi::CStr::from_ptr(new_sub).to_string_lossy();
        let replaced = str.replace(&*old_str, &*new_str);
        let c_str = std::ffi::CString::new(replaced).unwrap();
        c_str.into_raw()
    }
}

/// Convert bool to string stub for JIT
pub extern "C" fn vp_str_from_bool_stub(val: bool) -> *const std::ffi::c_char {
    let s = if val { "True" } else { "False" };
    let c_str = std::ffi::CString::new(s).unwrap();
    c_str.into_raw()
}

/// String equality comparison stub for JIT
/// Returns true if two strings have equal content
pub extern "C" fn vp_str_equals_stub(
    a: *const std::ffi::c_char,
    b: *const std::ffi::c_char,
) -> bool {
    if a.is_null() && b.is_null() {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }
    unsafe {
        let str_a = std::ffi::CStr::from_ptr(a).to_string_lossy();
        let str_b = std::ffi::CStr::from_ptr(b).to_string_lossy();
        str_a == str_b
    }
}

/// String comparison stub for JIT
/// Returns -1 if a < b, 0 if a == b, 1 if a > b
pub extern "C" fn vp_str_compare_stub(
    a: *const std::ffi::c_char,
    b: *const std::ffi::c_char,
) -> i64 {
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
        let str_a = std::ffi::CStr::from_ptr(a).to_string_lossy();
        let str_b = std::ffi::CStr::from_ptr(b).to_string_lossy();
        match str_a.cmp(&str_b) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }
}

/// String format stub for JIT
/// Format: vp_str_format(format_str, args_array, arg_count)
/// args_array points to array of string pointers
pub extern "C" fn vp_str_format_stub(
    format_str: *const std::ffi::c_char,
    args_array: *const *const std::ffi::c_char,
    arg_count: i64,
) -> *const std::ffi::c_char {
    use std::ffi::CStr;

    if format_str.is_null() {
        return std::ptr::null();
    }

    unsafe {
        let fmt = CStr::from_ptr(format_str).to_string_lossy();
        
        // Count {} placeholders and replace with arguments
        let mut result = fmt.into_owned();
        if !args_array.is_null() && arg_count > 0 {
            for i in 0..arg_count {
                let arg_ptr = *args_array.offset(i as isize);
                if !arg_ptr.is_null() {
                    let arg_str = CStr::from_ptr(arg_ptr).to_string_lossy();
                    if let Some(pos) = result.find("{}") {
                        result.replace_range(pos..pos+2, &arg_str);
                    }
                }
            }
        }
        
        let c_str = std::ffi::CString::new(result).unwrap();
        c_str.into_raw()
    }
}
