
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

pub extern "C" fn vp_print_str_stub(s: *mut std::ffi::c_void) {
    use std::io::{self, Write};
    if s.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s as *const std::ffi::c_char);
        if let Ok(rust_str) = c_str.to_str() {
            print!("{}", rust_str);
            io::stdout().flush().unwrap();
        }
    }
}

