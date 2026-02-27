// OS module stubs for JIT
use std::env;
use std::fs;
use std::path::Path;
use std::io;

pub extern "C" fn vp_os_getcwd() -> *mut i8 {
    match env::current_dir() {
        Ok(path) => {
            let path_str = path.to_string_lossy();
            let c_str = std::ffi::CString::new(path_str.as_bytes()).unwrap();
            c_str.into_raw()
        }
        Err(_) => std::ptr::null_mut(),
    }
}

pub extern "C" fn vp_os_chdir(path: *const i8) -> i64 {
    if path.is_null() {
        return -1;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            match env::set_current_dir(path_rust) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

pub extern "C" fn vp_os_listdir(path: *const i8) -> *mut std::ffi::c_void {
    // Return a ViperList* - stub implementation
    // Full implementation requires Viper list integration
    std::ptr::null_mut()
}

pub extern "C" fn vp_os_path_join(a: *const i8, b: *const i8) -> *mut i8 {
    let a_str = if a.is_null() { "" } else {
        unsafe {
            std::ffi::CStr::from_ptr(a).to_str().unwrap_or("")
        }
    };
    
    let b_str = if b.is_null() { "" } else {
        unsafe {
            std::ffi::CStr::from_ptr(b).to_str().unwrap_or("")
        }
    };
    
    let path = Path::new(a_str).join(b_str);
    let path_str = path.to_string_lossy();
    let c_str = std::ffi::CString::new(path_str.as_bytes()).unwrap();
    c_str.into_raw()
}

pub extern "C" fn vp_os_getenv(name: *const i8) -> *const i8 {
    if name.is_null() {
        return std::ptr::null();
    }
    
    unsafe {
        let name_str = std::ffi::CStr::from_ptr(name);
        if let Ok(name_rust) = name_str.to_str() {
            if let Ok(val) = env::var(name_rust) {
                let c_str = std::ffi::CString::new(val).unwrap();
                return c_str.into_raw();
            }
        }
    }
    std::ptr::null()
}

pub extern "C" fn vp_os_mkdir(path: *const i8, mode: i64) -> i64 {
    if path.is_null() {
        return -1;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            match fs::create_dir(path_rust) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

pub extern "C" fn vp_os_makedirs(path: *const i8, mode: i64) -> i64 {
    if path.is_null() {
        return -1;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            match fs::create_dir_all(path_rust) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

pub extern "C" fn vp_os_remove(path: *const i8) -> i64 {
    if path.is_null() {
        return -1;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            let metadata = fs::metadata(path_rust);
            if let Ok(meta) = metadata {
                if meta.is_dir() {
                    match fs::remove_dir(path_rust) {
                        Ok(_) => 0,
                        Err(_) => -1,
                    }
                } else {
                    match fs::remove_file(path_rust) {
                        Ok(_) => 0,
                        Err(_) => -1,
                    }
                }
            } else {
                -1
            }
        } else {
            -1
        }
    }
}

pub extern "C" fn vp_os_path_exists(path: *const i8) -> i64 {
    if path.is_null() {
        return 0;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            Path::new(path_rust).exists() as i64
        } else {
            0
        }
    }
}

pub extern "C" fn vp_os_path_isfile(path: *const i8) -> i64 {
    if path.is_null() {
        return 0;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            Path::new(path_rust).is_file() as i64
        } else {
            0
        }
    }
}

pub extern "C" fn vp_os_path_isdir(path: *const i8) -> i64 {
    if path.is_null() {
        return 0;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            Path::new(path_rust).is_dir() as i64
        } else {
            0
        }
    }
}

pub extern "C" fn vp_os_path_getsize(path: *const i8) -> i64 {
    if path.is_null() {
        return -1;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            match fs::metadata(path_rust) {
                Ok(meta) => meta.len() as i64,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

pub extern "C" fn vp_os_path_abspath(path: *const i8) -> *mut i8 {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            let abs_path = Path::new(path_rust).canonicalize().unwrap_or_else(|_| Path::new(path_rust).to_path_buf());
            let path_string = abs_path.to_string_lossy();
            let c_str = std::ffi::CString::new(path_string.as_bytes()).unwrap();
            c_str.into_raw()
        } else {
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn vp_os_path_basename(path: *const i8) -> *mut i8 {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            if let Some(base) = Path::new(path_rust).file_name() {
                let base_str = base.to_string_lossy();
                let c_str = std::ffi::CString::new(base_str.as_bytes()).unwrap();
                c_str.into_raw()
            } else {
                std::ptr::null_mut()
            }
        } else {
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn vp_os_path_dirname(path: *const i8) -> *mut i8 {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            let parent = Path::new(path_rust).parent().unwrap_or(Path::new(""));
            let parent_str = parent.to_string_lossy();
            let c_str = std::ffi::CString::new(parent_str.as_bytes()).unwrap();
            c_str.into_raw()
        } else {
            std::ptr::null_mut()
        }
    }
}

pub extern "C" fn vp_os_rename(src: *const i8, dst: *const i8) -> i64 {
    if src.is_null() || dst.is_null() {
        return -1;
    }
    
    unsafe {
        let src_str = std::ffi::CStr::from_ptr(src);
        let dst_str = std::ffi::CStr::from_ptr(dst);
        
        if let (Ok(src_rust), Ok(dst_rust)) = (src_str.to_str(), dst_str.to_str()) {
            match fs::rename(src_rust, dst_rust) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

pub extern "C" fn vp_os_copy(src: *const i8, dst: *const i8) -> i64 {
    if src.is_null() || dst.is_null() {
        return -1;
    }
    
    unsafe {
        let src_str = std::ffi::CStr::from_ptr(src);
        let dst_str = std::ffi::CStr::from_ptr(dst);
        
        if let (Ok(src_rust), Ok(dst_rust)) = (src_str.to_str(), dst_str.to_str()) {
            match fs::copy(src_rust, dst_rust) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}

pub extern "C" fn vp_os_get_home() -> *mut i8 {
    if let Some(home) = env::var("HOME").ok()
        .or_else(|| env::var("USERPROFILE").ok()) {
        let c_str = std::ffi::CString::new(home).unwrap();
        c_str.into_raw()
    } else {
        std::ptr::null_mut()
    }
}

pub extern "C" fn vp_os_getpid() -> i64 {
    std::process::id() as i64
}

pub extern "C" fn vp_os_stat(
    path: *const i8,
    size: *mut i64,
    mode: *mut i64,
    mtime: *mut f64,
    is_dir: *mut i64,
    is_file: *mut i64,
) -> i64 {
    if path.is_null() {
        return -1;
    }
    
    unsafe {
        let path_str = std::ffi::CStr::from_ptr(path);
        if let Ok(path_rust) = path_str.to_str() {
            match fs::metadata(path_rust) {
                Ok(meta) => {
                    if !size.is_null() { *size = meta.len() as i64; }
                    if !mode.is_null() { *mode = 0; } // Mode not easily available in Rust std
                    if !mtime.is_null() {
                        if let Ok(m) = meta.modified() {
                            if let Ok(d) = m.duration_since(std::time::UNIX_EPOCH) {
                                *mtime = d.as_secs_f64();
                            }
                        }
                    }
                    if !is_dir.is_null() { *is_dir = meta.is_dir() as i64; }
                    if !is_file.is_null() { *is_file = meta.is_file() as i64; }
                    0
                }
                Err(_) => -1,
            }
        } else {
            -1
        }
    }
}
