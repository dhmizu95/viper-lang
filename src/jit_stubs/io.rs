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
    pub data: ViperStringData,
}

#[repr(C)]
pub union ViperStringData {
    pub heap: ViperStringHeap,
    pub sso: ViperStringSSO,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ViperStringHeap {
    pub heap_data: *const u8, /* 0:  Pointer to heap data */
    pub length: i64,          /* 8:  String length (positive, bit 63 is 0) */
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ViperStringSSO {
    pub sso_data: [u8; 15], /* 0-14: Inline storage */
    pub sso_len_flags: i8,  /* 15:   Length and SSO flag (bit 7) */
}

impl ViperString {
    /// Get the string length (handles SSO flag)
    pub fn length(&self) -> i64 {
        let length = unsafe { self.data.heap.length };
        if length < 0 {
            // SSO mode - flag is bit 63, length is in sso_len_flags (bits 0-6)
            unsafe { (self.data.sso.sso_len_flags & 0x7F) as i64 }
        } else {
            length
        }
    }

    /// Get the string data as a slice
    pub fn as_slice(&self) -> &[u8] {
        let length = self.length() as usize;
        if length == 0 {
            return &[];
        }

        unsafe {
            if self.data.heap.length < 0 {
                // SSO - data is inline
                let sso_len = length.min(15);
                &self.data.sso.sso_data[..sso_len]
            } else {
                // Heap - data pointer
                let data_ptr = self.data.heap.heap_data;
                if data_ptr.is_null() {
                    &[]
                } else {
                    std::slice::from_raw_parts(data_ptr, length)
                }
            }
        }
    }
}

pub extern "C" fn vp_print_str(s: *mut ViperString) {
    use std::io::{self, Write};
    if s.is_null() {
        return;
    }
    unsafe {
        let viper_str = &*s;
        let length = viper_str.data.heap.length;

        // Check SSO flag (bit 63)
        if length < 0 {
            // SSO - data is inline
            let sso_len = (viper_str.data.sso.sso_len_flags & 0x7F) as usize;
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
