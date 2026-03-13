// Regex (re) module stubs for JIT - Phase 2
// POSIX regex wrappers

use std::ffi::{CStr, CString};

/* ============================================ */
// Pattern Object
/* ============================================ */

pub struct ViperPattern {
    pattern: String,
    flags: i64,
}

#[no_mangle]
pub extern "C" fn vp_re_compile(pattern: *const i8, flags: i64) -> *mut ViperPattern {
    if pattern.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let c_str = CStr::from_ptr(pattern);
        if let Ok(pattern_str) = c_str.to_str() {
            let p = Box::new(ViperPattern { pattern: pattern_str.to_string(), flags });
            return Box::into_raw(p);
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_re_pattern_free(pattern: *mut ViperPattern) {
    if !pattern.is_null() {
        unsafe {
            drop(Box::from_raw(pattern));
        }
    }
}

/* ============================================ */
// Match Object
/* ============================================ */

pub struct ViperMatch {
    start: i64,
    end: i64,
    group: String,
}

#[no_mangle]
pub extern "C" fn vp_re_match(
    pattern: *mut ViperPattern,
    string: *const i8,
    pos: i64,
) -> *mut ViperMatch {
    if pattern.is_null() || string.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let c_str = CStr::from_ptr(string);
        let string_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        let pat = &*pattern;
        let regex_pattern = &pat.pattern;

        // Simplified: use Rust regex for matching
        if let Ok(re) = regex::Regex::new(regex_pattern) {
            let search_str = if pos > 0 && (pos as usize) < string_str.len() {
                &string_str[pos as usize..]
            } else {
                string_str
            };

            if let Some(m) = re.find(search_str) {
                let match_obj = Box::new(ViperMatch {
                    start: pos + m.start() as i64,
                    end: pos + m.end() as i64,
                    group: m.as_str().to_string(),
                });
                return Box::into_raw(match_obj);
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_re_search(
    pattern: *mut ViperPattern,
    string: *const i8,
    pos: i64,
    endpos: i64,
) -> *mut ViperMatch {
    if pattern.is_null() || string.is_null() {
        return std::ptr::null_mut();
    }

    unsafe {
        let c_str = CStr::from_ptr(string);
        let string_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return std::ptr::null_mut(),
        };

        let pat = &*pattern;
        let regex_pattern = &pat.pattern;

        if let Ok(re) = regex::Regex::new(regex_pattern) {
            let start = if pos > 0 { pos as usize } else { 0 };
            let end = if endpos > 0 && (endpos as usize) <= string_str.len() {
                endpos as usize
            } else {
                string_str.len()
            };

            let search_str = &string_str[start..end];

            if let Some(m) = re.find(search_str) {
                let match_obj = Box::new(ViperMatch {
                    start: start as i64 + m.start() as i64,
                    end: start as i64 + m.end() as i64,
                    group: m.as_str().to_string(),
                });
                return Box::into_raw(match_obj);
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_re_findall(
    _pattern: *mut ViperPattern,
    _string: *const i8,
) -> *mut std::ffi::c_void {
    // Returns ViperList* - simplified, returns null for now
    // Full implementation would create list of matches
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_re_split(
    _pattern: *mut ViperPattern,
    _string: *const i8,
) -> *mut std::ffi::c_void {
    // Returns ViperList* - simplified, returns null for now
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_re_sub(
    pattern: *mut ViperPattern,
    repl: *const i8,
    string: *const i8,
    count: i64,
) -> *mut i8 {
    if pattern.is_null() || repl.is_null() || string.is_null() {
        return CString::new("").unwrap().into_raw();
    }

    unsafe {
        let c_str = CStr::from_ptr(string);
        let string_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("").unwrap().into_raw(),
        };

        let repl_c_str = CStr::from_ptr(repl);
        let repl_str = match repl_c_str.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("").unwrap().into_raw(),
        };

        let pat = &*pattern;
        let regex_pattern = &pat.pattern;

        if let Ok(re) = regex::Regex::new(regex_pattern) {
            let result = if count > 0 {
                re.replacen(string_str, count as usize, repl_str)
            } else {
                re.replace_all(string_str, repl_str)
            };

            return CString::new(result.into_owned()).unwrap().into_raw();
        }
    }
    CString::new("").unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn vp_re_fullmatch(pattern: *mut ViperPattern, string: *const i8) -> i64 {
    if pattern.is_null() || string.is_null() {
        return 0;
    }

    unsafe {
        let c_str = CStr::from_ptr(string);
        let string_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        let pat = &*pattern;
        let regex_pattern = &pat.pattern;

        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            if re.is_match(string_str) {
                return 1;
            }
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn vp_re_escape(string: *const i8) -> *mut i8 {
    if string.is_null() {
        return CString::new("").unwrap().into_raw();
    }

    unsafe {
        let c_str = CStr::from_ptr(string);
        let string_str = match c_str.to_str() {
            Ok(s) => s,
            Err(_) => return CString::new("").unwrap().into_raw(),
        };

        // Escape regex special characters
        let escaped = regex::escape(string_str);
        return CString::new(escaped).unwrap().into_raw();
    }
}

/* ============================================ */
// Match object methods
/* ============================================ */

#[no_mangle]
pub extern "C" fn vp_match_free(m: *mut ViperMatch) {
    if !m.is_null() {
        unsafe {
            drop(Box::from_raw(m));
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_match_start(m: *mut ViperMatch) -> i64 {
    if m.is_null() {
        return 0;
    }
    unsafe { (*m).start }
}

#[no_mangle]
pub extern "C" fn vp_match_end(m: *mut ViperMatch) -> i64 {
    if m.is_null() {
        return 0;
    }
    unsafe { (*m).end }
}

#[no_mangle]
pub extern "C" fn vp_match_group(m: *mut ViperMatch) -> *mut i8 {
    if m.is_null() {
        return CString::new("").unwrap().into_raw();
    }
    unsafe {
        let group = &(*m).group;
        CString::new(group.clone()).unwrap().into_raw()
    }
}

#[no_mangle]
pub extern "C" fn vp_match_span(m: *mut ViperMatch, start: *mut i64, end: *mut i64) {
    if m.is_null() {
        return;
    }
    unsafe {
        if !start.is_null() {
            *start = (*m).start;
        }
        if !end.is_null() {
            *end = (*m).end;
        }
    }
}

/* ============================================ */
// Flag constants
/* ============================================ */

#[no_mangle]
pub extern "C" fn vp_re_ignorecase() -> i64 {
    0x01
}

#[no_mangle]
pub extern "C" fn vp_re_multiline() -> i64 {
    0x02
}

#[no_mangle]
pub extern "C" fn vp_re_dotall() -> i64 {
    0x04
}

#[no_mangle]
pub extern "C" fn vp_re_verbose() -> i64 {
    0x08
}
