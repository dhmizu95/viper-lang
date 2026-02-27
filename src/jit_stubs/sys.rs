// System module stubs for JIT
use std::env;
use std::process;

pub extern "C" fn vp_sys_exit(code: i64) {
    process::exit(code as i32);
}

pub extern "C" fn vp_sys_getpid() -> i64 {
    std::process::id() as i64
}

pub extern "C" fn vp_sys_get_version() -> *const i8 {
    b"0.4.1\0" as *const u8 as *const i8
}

pub extern "C" fn vp_sys_get_platform() -> *const i8 {
    const PLATFORM: &[u8] = b"linux\0";
    PLATFORM.as_ptr() as *const i8
}

pub extern "C" fn vp_sys_get_sysname() -> *const i8 {
    const SYSNAME: &[u8] = b"Linux\0";
    SYSNAME.as_ptr() as *const i8
}

pub extern "C" fn vp_sys_get_machine() -> *const i8 {
    const MACHINE: &[u8] = b"x86_64\0";
    MACHINE.as_ptr() as *const i8
}

pub extern "C" fn vp_sys_getenv(name: *const i8) -> *const i8 {
    if name.is_null() {
        return std::ptr::null();
    }
    
    unsafe {
        let name_str = std::ffi::CStr::from_ptr(name);
        if let Ok(name_rust) = name_str.to_str() {
            if let Ok(val) = env::var(name_rust) {
                // Leak the string - caller is responsible for freeing
                let c_str = std::ffi::CString::new(val).unwrap();
                return c_str.into_raw();
            }
        }
    }
    std::ptr::null()
}

pub extern "C" fn vp_sys_setenv(name: *const i8, value: *const i8, overwrite: i64) -> i64 {
    if name.is_null() || value.is_null() {
        return -1;
    }
    
    unsafe {
        let name_str = std::ffi::CStr::from_ptr(name);
        let value_str = std::ffi::CStr::from_ptr(value);
        
        if let (Ok(name_rust), Ok(value_rust)) = (name_str.to_str(), value_str.to_str()) {
            if overwrite != 0 || env::var(name_rust).is_err() {
                env::set_var(name_rust, value_rust);
                return 0;
            }
        }
    }
    -1
}

pub extern "C" fn vp_sys_unsetenv(name: *const i8) -> i64 {
    if name.is_null() {
        return -1;
    }
    
    unsafe {
        let name_str = std::ffi::CStr::from_ptr(name);
        if let Ok(name_rust) = name_str.to_str() {
            env::remove_var(name_rust);
            return 0;
        }
    }
    -1
}

pub extern "C" fn vp_sys_init(argc: i64, argv: *const *const i8) {
    // Initialize sys module with argc/argv
    // For JIT, we use Rust's env::args()
    let _ = argc;
    let _ = argv;
}

pub extern "C" fn vp_sys_get_argv() -> *mut std::ffi::c_void {
    // Return a list of command-line arguments
    // This is a stub - full implementation requires Viper list integration
    std::ptr::null_mut()
}
