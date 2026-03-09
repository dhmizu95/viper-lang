//! Tuple JIT stubs

use std::alloc::{self, Layout};

/// ViperTuple structure (must match runtime/include/viper_types.h)
#[repr(C)]
pub struct ViperTuple {
    ref_count: i64,
    size: i64,
    elements: *mut i64,
    _reserved: u64,
}

/// Create a tuple with given size
pub extern "C" fn vp_tuple_create_stub(size: i64) -> *mut ViperTuple {
    if size < 0 {
        return std::ptr::null_mut();
    }

    unsafe {
        let mut tuple = Box::new(ViperTuple {
            ref_count: 1,
            size,
            elements: std::ptr::null_mut(),
            _reserved: 0,
        });

        if size > 0 {
            let elements_layout = Layout::from_size_align((size as usize) * 8, 8).unwrap();
            let elements = alloc::alloc(elements_layout) as *mut i64;
            // Initialize to 0
            for i in 0..size as usize {
                *elements.add(i) = 0;
            }
            tuple.elements = elements;
        }

        Box::into_raw(tuple)
    }
}

/// Free a tuple
pub extern "C" fn vp_tuple_free_stub(tuple: *mut ViperTuple) {
    if tuple.is_null() {
        return;
    }

    unsafe {
        let t = Box::from_raw(tuple);
        if !t.elements.is_null() && t.size > 0 {
            let elements_layout = Layout::from_size_align((t.size as usize) * 8, 8).unwrap();
            alloc::dealloc(t.elements as *mut u8, elements_layout);
        }
    }
}

/// Get element at index from tuple
pub extern "C" fn vp_tuple_get_stub(tuple: *mut ViperTuple, index: i64) -> i64 {
    if tuple.is_null() {
        return 0;
    }

    unsafe {
        let t = &*tuple;
        let mut idx = index;
        
        // Handle negative indices
        if idx < 0 {
            idx = t.size + idx;
        }

        if idx < 0 || idx >= t.size || t.elements.is_null() {
            return 0;
        }

        *t.elements.offset(idx as isize)
    }
}

/// Set element at index in tuple
pub extern "C" fn vp_tuple_set_stub(tuple: *mut ViperTuple, index: i64, value: i64) {
    if tuple.is_null() || index < 0 {
        return;
    }

    unsafe {
        let t = &*tuple;
        if index >= t.size || t.elements.is_null() {
            return;
        }
        *t.elements.offset(index as isize) = value;
    }
}

/// Get length of tuple
pub extern "C" fn vp_tuple_len_stub(tuple: *mut ViperTuple) -> i64 {
    if tuple.is_null() {
        return 0;
    }
    unsafe { (*tuple).size }
}

/// Convert tuple to string
pub extern "C" fn vp_tuple_to_str_stub(tuple: *mut ViperTuple) -> *mut std::ffi::c_void {
    use crate::jit_stubs::strings::create_viper_string;
    
    if tuple.is_null() {
        return create_viper_string("()") as *mut _;
    }

    unsafe {
        let t = &*tuple;
        if t.size == 0 || t.elements.is_null() {
            return create_viper_string("()") as *mut _;
        }

        let mut parts = Vec::new();
        for i in 0..t.size as usize {
            parts.push(format!("{}", *t.elements.add(i)));
        }
        let result = format!("({})", parts.join(", "));
        create_viper_string(&result) as *mut _
    }
}
