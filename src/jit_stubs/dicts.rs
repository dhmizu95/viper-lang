use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ffi::{c_void, c_char, CStr};


use std::sync::Mutex;

static JIT_DICT_COUNTER: AtomicUsize = AtomicUsize::new(0);
// Simple global dict storage for JIT - maps dict_id to HashMap<String, i64>
static JIT_DICTS: Mutex<Option<HashMap<usize, HashMap<String, i64>>>> = Mutex::new(None);

fn get_jit_dicts() -> std::sync::MutexGuard<'static, Option<HashMap<usize, HashMap<String, i64>>>> {
    let mut dicts = JIT_DICTS.lock().unwrap();
    if dicts.is_none() {
        *dicts = Some(HashMap::new());
    }
    dicts
}

pub extern "C" fn vp_dict_create() -> *mut std::ffi::c_void {
    let id = JIT_DICT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dict: HashMap<String, i64> = HashMap::new();
    get_jit_dicts().as_mut().unwrap().insert(id, dict);
    id as *mut std::ffi::c_void
}

pub extern "C" fn vp_dict_set_str_i64(dict_ptr: *mut std::ffi::c_void, key: *const std::ffi::c_char, value: i64) {
    if dict_ptr.is_null() || key.is_null() {
        return;
    }
    let id = dict_ptr as usize;
    unsafe {
        let key_str = std::ffi::CStr::from_ptr(key).to_string_lossy().into_owned();
        get_jit_dicts().as_mut().unwrap().get_mut(&id).unwrap().insert(key_str, value);
    }
}

pub extern "C" fn vp_dict_set_str_str(dict_ptr: *mut std::ffi::c_void, key: *const std::ffi::c_char, _value: *const std::ffi::c_char) {
    // For now, just handle string values by storing as-is
    if dict_ptr.is_null() || key.is_null() {
        return;
    }
    // String values not fully supported in JIT yet - simplified implementation
}

pub extern "C" fn vp_dict_get_i64(dict_ptr: *mut std::ffi::c_void, key: *const std::ffi::c_char) -> i64 {
    if dict_ptr.is_null() || key.is_null() {
        return 0;
    }
    let id = dict_ptr as usize;
    unsafe {
        let key_str = std::ffi::CStr::from_ptr(key).to_string_lossy();
        get_jit_dicts().as_ref().unwrap().get(&id).and_then(|d| d.get(key_str.as_ref())).copied().unwrap_or(0)
    }
}

pub extern "C" fn vp_dict_free(dict_ptr: *mut std::ffi::c_void) {
    if dict_ptr.is_null() {
        return;
    }
    let id = dict_ptr as usize;
    get_jit_dicts().as_mut().unwrap().remove(&id);
}

