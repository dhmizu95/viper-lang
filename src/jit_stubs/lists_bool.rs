// Stub implementations for bool list functions (JIT mode)
// These stubs use the same ViperList struct layout as the runtime

use std::ffi::c_void;

// Match the ViperList struct layout from viper_types.h
#[repr(C)]
pub struct ViperList {
    pub ref_count: i64,
    pub length: i64,
    pub capacity: i64,
    pub elem_type: i64,    // ViperListType (i32 but aligned to 8 bytes)
    pub data: *mut c_void, // Union pointer - for bool lists: *mut int8_t
}

const VIPER_LIST_BOOL: i64 = 2; // Must match viper_types.h

/// Create a bool list - JIT stub
/// Uses heap allocation with ViperList struct for compatibility
pub extern "C" fn vp_list_bool_create_stub() -> *mut ViperList {
    let vec = Box::new(Vec::<bool>::new());
    let data_ptr = Box::into_raw(vec) as *mut c_void;

    let list = Box::new(ViperList {
        ref_count: 1,
        length: 0,
        capacity: 0,
        elem_type: VIPER_LIST_BOOL,
        data: data_ptr,
    });

    Box::into_raw(list)
}

/// Append to bool list - JIT stub
pub extern "C" fn vp_list_bool_append_stub(list: *mut ViperList, value: bool) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        let vec = &mut *(list_ref.data as *mut Vec<bool>);
        vec.push(value);
        list_ref.length += 1;
    }
}

/// Get element from bool list - JIT stub
pub extern "C" fn vp_list_bool_get_stub(list: *mut ViperList, index: i64) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let list_ref = &*list;
        if index < 0 || index >= list_ref.length {
            return false;
        }
        let vec = &*(list_ref.data as *mut Vec<bool>);
        vec[index as usize]
    }
}

/// Set element in bool list - JIT stub
pub extern "C" fn vp_list_bool_set_stub(list: *mut ViperList, index: i64, value: bool) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        if index >= 0 && index < list_ref.length {
            let vec = &mut *(list_ref.data as *mut Vec<bool>);
            vec[index as usize] = value;
        }
    }
}

/// Create bool list with repeated element - JIT stub
pub extern "C" fn vp_list_bool_repeat_stub(elem: bool, count: i64) -> *mut ViperList {
    let mut vec = Vec::<bool>::new();
    vec.resize(count as usize, elem);
    let data_ptr = Box::into_raw(Box::new(vec)) as *mut c_void;

    let list = Box::new(ViperList {
        ref_count: 1,
        length: count,
        capacity: count,
        elem_type: VIPER_LIST_BOOL,
        data: data_ptr,
    });

    Box::into_raw(list)
}

/// Initialize stack-allocated bool list - JIT stub
/// For JIT, we use heap allocation but maintain struct compatibility
pub extern "C" fn vp_list_bool_init_stack_stub(
    list_ptr: *mut ViperList,
    _buffer: *mut c_void,
    count: i64,
    elem: bool,
) {
    if list_ptr.is_null() {
        return;
    }

    // Create heap-allocated Vec and wrap in ViperList struct
    let mut vec = Vec::<bool>::new();
    vec.resize(count as usize, elem);
    let data_ptr = Box::into_raw(Box::new(vec)) as *mut c_void;

    unsafe {
        // Write the ViperList struct directly to list_ptr
        *list_ptr = ViperList {
            ref_count: 1,
            length: count,
            capacity: count,
            elem_type: VIPER_LIST_BOOL,
            data: data_ptr,
        };
    }
}

/// Free bool list - JIT stub
pub extern "C" fn vp_list_bool_free_stub(list: *mut ViperList) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &*list;
        // Free the data (Vec<bool>)
        let _ = Box::from_raw(list_ref.data as *mut Vec<bool>);
        // Free the list struct
        let _ = Box::from_raw(list);
    }
}
