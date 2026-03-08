// Runtime function implementations for JIT
pub extern "C" fn vp_print_i64(val: i64) {
    use std::io::{self, Write};
    print!("{}", val);
    io::stdout().flush().unwrap();
}

pub extern "C" fn vp_print_f64(val: f64) {
    use std::io::{self, Write};
    print!("{}", val);
    io::stdout().flush().unwrap();
}

pub extern "C" fn vp_print_bool(val: bool) {
    use std::io::{self, Write};
    print!("{}", if val { "True" } else { "False" });
    io::stdout().flush().unwrap();
}

pub extern "C" fn vp_print_newline() {
    println!();
}

// ViperString structure (must match runtime/include/viper_types.h)
#[repr(C)]
pub struct ViperString {
    data: ViperStringData,
}

#[repr(C)]
union ViperStringData {
    heap: ViperStringHeap,
    sso: ViperStringSSO,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ViperStringHeap {
    ref_count: i64,
    length: i64,
    heap_data: *const u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ViperStringSSO {
    _unused: i64,
    sso_length: i8,
    sso_data: [u8; 15],
}

pub extern "C" fn vp_print_str(s: *mut ViperString) {
    use std::io::{self, Write};
    if s.is_null() {
        return;
    }
    unsafe {
        let viper_str = &*s;
        let length = viper_str.data.heap.length;
        
        // Check SSO flag (high bit)
        if length & 0x80 != 0 {
            // SSO - data is inline
            let sso_len = (length & 0x7F) as usize;
            let sso_data = &viper_str.data.sso.sso_data[..sso_len.min(15)];
            if let Ok(rust_str) = std::str::from_utf8(sso_data) {
                print!("{}", rust_str);
                io::stdout().flush().unwrap();
            }
        } else {
            // Heap - data pointer
            let data_ptr = viper_str.data.heap.heap_data;
            let len = length as usize;
            if !data_ptr.is_null() {
                let slice = std::slice::from_raw_parts(data_ptr, len);
                if let Ok(rust_str) = std::str::from_utf8(slice) {
                    print!("{}", rust_str);
                    io::stdout().flush().unwrap();
                }
            }
        }
    }
}

pub extern "C" fn vp_print_cstr(s: *const std::ffi::c_char) {
    use std::io::{self, Write};
    if s.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s);
        if let Ok(rust_str) = c_str.to_str() {
            print!("{}", rust_str);
            io::stdout().flush().unwrap();
        }
    }
}
