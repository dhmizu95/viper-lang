// Stub implementations for list functions (Phase 2 MVP)
// Using Box<Vec<i64>> as the internal representation
pub extern "C" fn vp_list_create_stub() -> *mut std::ffi::c_void {
    let list = Box::new(Vec::<i64>::new());
    Box::into_raw(list) as *mut std::ffi::c_void
}

pub extern "C" fn vp_list_append_stub(list: *mut std::ffi::c_void, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.push(val);
    }
}

pub extern "C" fn vp_list_free_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(list as *mut Vec<i64>);
    }
}

pub extern "C" fn vp_list_get_stub(list: *mut std::ffi::c_void, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        if index < 0 || index as usize >= vec.len() {
            return 0;
        }
        vec[index as usize]
    }
}

pub extern "C" fn vp_list_len_stub(list: *mut std::ffi::c_void) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        vec.len() as i64
    }
}

pub extern "C" fn vp_list_set_stub(list: *mut std::ffi::c_void, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        if index >= 0 && (index as usize) < vec.len() {
            vec[index as usize] = val;
        }
    }
}

pub extern "C" fn vp_list_insert_stub(list: *mut std::ffi::c_void, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        if index >= 0 && (index as usize) <= vec.len() {
            vec.insert(index as usize, val);
        }
    }
}

pub extern "C" fn vp_list_remove_stub(list: *mut std::ffi::c_void, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        if index >= 0 && (index as usize) < vec.len() {
            vec.remove(index as usize)
        } else {
            0
        }
    }
}

pub extern "C" fn vp_list_pop_stub(list: *mut std::ffi::c_void) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.pop().unwrap_or(0)
    }
}

pub extern "C" fn vp_list_clear_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.clear();
    }
}

pub extern "C" fn vp_list_contains_stub(list: *mut std::ffi::c_void, val: i64) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        vec.contains(&val)
    }
}

pub extern "C" fn vp_list_print_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        print!("[null]");
        return;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        print!("[");
        for (i, val) in vec.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", val);
        }
        print!("]");
    }
}

// Float list stubs (f64)
pub extern "C" fn vp_list_create_f64_stub() -> *mut std::ffi::c_void {
    let list = Box::new(Vec::<f64>::new());
    Box::into_raw(list) as *mut std::ffi::c_void
}

pub extern "C" fn vp_list_append_f64_stub(list: *mut std::ffi::c_void, val: f64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<f64>);
        vec.push(val);
    }
}

pub extern "C" fn vp_list_get_f64_stub(list: *mut std::ffi::c_void, index: i64) -> f64 {
    if list.is_null() {
        return 0.0;
    }
    unsafe {
        let vec = &*(list as *mut Vec<f64>);
        if index < 0 || index as usize >= vec.len() {
            return 0.0;
        }
        vec[index as usize]
    }
}

pub extern "C" fn vp_list_set_f64_stub(list: *mut std::ffi::c_void, index: i64, val: f64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<f64>);
        if index >= 0 && (index as usize) < vec.len() {
            vec[index as usize] = val;
        }
    }
}

pub extern "C" fn vp_range_stub(start: i64, end: i64) -> *mut std::ffi::c_void {
    let list: Vec<i64> = (start..end).collect();
    Box::into_raw(Box::new(list)) as *mut std::ffi::c_void
}

// List repeat stub - creates a new list with element repeated n times
pub extern "C" fn vp_list_repeat_stub(elem: i64, count: i64) -> *mut std::ffi::c_void {
    let mut result = Vec::<i64>::new();
    for _ in 0..count {
        result.push(elem);
    }
    let boxed = Box::new(result);
    Box::into_raw(boxed) as *mut std::ffi::c_void
}

// List slice stub - creates a new list with elements from start to end with step
pub extern "C" fn vp_list_slice_stub(
    list: *mut std::ffi::c_void,
    start: i64,
    end: i64,
    step: i64,
) -> *mut std::ffi::c_void {
    if list.is_null() {
        return vp_list_create_stub();
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        let len = vec.len() as i64;

        // Normalize negative indices
        let mut s = if start < 0 { (start + len).max(0) } else { start.min(len) };
        let mut e = if end < 0 { (end + len).max(0) } else { end.min(len) };

        // Clamp to valid range
        if s < 0 {
            s = 0;
        }
        if e > len {
            e = len;
        }
        if s >= e {
            return vp_list_create_stub();
        }

        let step = if step <= 0 { 1 } else { step };
        let mut result = Vec::<i64>::new();
        let mut i = s;
        while i < e {
            result.push(vec[i as usize]);
            i += step;
        }

        let boxed = Box::new(result);
        Box::into_raw(boxed) as *mut std::ffi::c_void
    }
}

pub extern "C" fn vp_retain_stub(_ptr: *mut std::ffi::c_void) {
    // No-op for JIT
}

pub extern "C" fn vp_release_stub(_ptr: *mut std::ffi::c_void) {
    // No-op for JIT
}

// Extended list operations
pub extern "C" fn vp_list_extend_stub(list: *mut std::ffi::c_void, other: *mut std::ffi::c_void) {
    if list.is_null() || other.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        let other_vec = &*(other as *mut Vec<i64>);
        vec.extend(other_vec.iter().cloned());
    }
}

pub extern "C" fn vp_list_index_stub(list: *mut std::ffi::c_void, val: i64) -> i64 {
    if list.is_null() {
        return -1;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        match vec.iter().position(|&x| x == val) {
            Some(idx) => idx as i64,
            None => -1,
        }
    }
}

pub extern "C" fn vp_list_count_stub(list: *mut std::ffi::c_void, val: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        vec.iter().filter(|&&x| x == val).count() as i64
    }
}

pub extern "C" fn vp_list_sort_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.sort();
    }
}

pub extern "C" fn vp_list_reverse_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.reverse();
    }
}

pub extern "C" fn vp_list_copy_stub(list: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if list.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        let copy = vec.clone();
        Box::into_raw(Box::new(copy)) as *mut std::ffi::c_void
    }
}

pub extern "C" fn vp_list_concat_stub(
    list1: *mut std::ffi::c_void,
    list2: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    let mut result = Vec::<i64>::new();
    if !list1.is_null() {
        unsafe {
            let vec1 = &*(list1 as *mut Vec<i64>);
            result.extend(vec1.iter().cloned());
        }
    }
    if !list2.is_null() {
        unsafe {
            let vec2 = &*(list2 as *mut Vec<i64>);
            result.extend(vec2.iter().cloned());
        }
    }
    Box::into_raw(Box::new(result)) as *mut std::ffi::c_void
}

pub extern "C" fn vp_list_sorted_stub(list: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if list.is_null() {
        return vp_list_create_stub();
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        let mut sorted = vec.clone();
        sorted.sort();
        Box::into_raw(Box::new(sorted)) as *mut std::ffi::c_void
    }
}

pub extern "C" fn vp_list_reversed_stub(list: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if list.is_null() {
        return vp_list_create_stub();
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        let mut reversed = vec.clone();
        reversed.reverse();
        Box::into_raw(Box::new(reversed)) as *mut std::ffi::c_void
    }
}
