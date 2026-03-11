// Stub implementations for list functions (JIT mode)
// Using ViperList struct compatible with runtime
// ViperList layout (40 bytes):
//   offset 0-7: ref_count (i64)
//   offset 8-15: length (i64)
//   offset 16-23: capacity (i64)
//   offset 24-31: elem_type (i64)
//   offset 32-39: data pointer (direct pointer to array)

const VIPER_LIST_I64: i64 = 0;
const VIPER_LIST_F64: i64 = 1;

#[repr(C)]
pub struct ViperListStub {
    pub ref_count: i64,
    pub length: i64,
    pub capacity: i64,
    pub elem_type: i64,
    pub data: *mut i64,  // Direct pointer to array data
}

#[repr(C)]
pub struct ViperListF64Stub {
    pub ref_count: i64,
    pub length: i64,
    pub capacity: i64,
    pub elem_type: i64,
    pub data: *mut f64,  // Direct pointer to f64 array data
}

fn create_viper_list_stub(capacity: i64) -> *mut ViperListStub {
    let capacity = if capacity > 0 { capacity } else { 8 };
    
    // Allocate array data directly on heap
    let data_ptr = unsafe {
        libc::malloc((capacity as usize) * std::mem::size_of::<i64>()) as *mut i64
    };
    
    if data_ptr.is_null() {
        panic!("Failed to allocate list data");
    }
    
    // Initialize to zero
    unsafe {
        std::ptr::write_bytes(data_ptr, 0, capacity as usize);
    }

    let list = Box::new(ViperListStub {
        ref_count: 1,
        length: 0,
        capacity,
        elem_type: VIPER_LIST_I64,
        data: data_ptr,
    });

    Box::into_raw(list)
}

fn create_viper_list_f64_stub(capacity: i64) -> *mut ViperListF64Stub {
    let capacity = if capacity > 0 { capacity } else { 8 };
    
    // Allocate array data directly on heap
    let data_ptr = unsafe {
        libc::malloc((capacity as usize) * std::mem::size_of::<f64>()) as *mut f64
    };
    
    if data_ptr.is_null() {
        panic!("Failed to allocate list data");
    }
    
    // Initialize to zero
    unsafe {
        std::ptr::write_bytes(data_ptr, 0, capacity as usize);
    }

    let list = Box::new(ViperListF64Stub {
        ref_count: 1,
        length: 0,
        capacity,
        elem_type: VIPER_LIST_F64,
        data: data_ptr,
    });

    Box::into_raw(list)
}

pub extern "C" fn vp_list_create_stub() -> *mut std::ffi::c_void {
    create_viper_list_stub(8) as *mut std::ffi::c_void
}

pub extern "C" fn vp_list_append_stub(list: *mut std::ffi::c_void, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);

        // Grow if needed
        if list_ref.length >= list_ref.capacity {
            let new_capacity = list_ref.capacity * 2;
            let new_data = libc::realloc(
                list_ref.data as *mut std::ffi::c_void,
                (new_capacity as usize) * std::mem::size_of::<i64>(),
            ) as *mut i64;
            
            if new_data.is_null() {
                panic!("Failed to grow list");
            }
            
            list_ref.data = new_data;
            list_ref.capacity = new_capacity;
        }

        // Append value
        *list_ref.data.add(list_ref.length as usize) = val;
        list_ref.length += 1;
    }
}

pub extern "C" fn vp_list_free_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        libc::free(list_ref.data as *mut std::ffi::c_void);
        let _ = Box::from_raw(list_ref);
    }
}

pub extern "C" fn vp_list_get_stub(list: *mut std::ffi::c_void, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);

        if list_ref.data.is_null() {
            return 0;
        }

        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx < 0 || idx >= list_ref.length {
            return 0;
        }

        *list_ref.data.add(idx as usize)
    }
}

pub extern "C" fn vp_list_len_stub(list: *mut std::ffi::c_void) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        list_ref.length
    }
}

pub extern "C" fn vp_list_set_stub(list: *mut std::ffi::c_void, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx >= 0 && idx < list_ref.length {
            *list_ref.data.add(idx as usize) = val;
        }
    }
}

pub extern "C" fn vp_list_insert_stub(list: *mut std::ffi::c_void, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx >= 0 && idx <= list_ref.length {
            // Grow if needed
            if list_ref.length >= list_ref.capacity {
                let new_capacity = list_ref.capacity * 2;
                let new_data = libc::realloc(
                    list_ref.data as *mut std::ffi::c_void,
                    (new_capacity as usize) * std::mem::size_of::<i64>(),
                ) as *mut i64;
                
                if new_data.is_null() {
                    panic!("Failed to grow list");
                }
                
                list_ref.data = new_data;
                list_ref.capacity = new_capacity;
            }

            // Shift elements to the right
            for i in (idx..list_ref.length).rev() {
                *list_ref.data.add((i + 1) as usize) = *list_ref.data.add(i as usize);
            }

            // Insert value
            *list_ref.data.add(idx as usize) = val;
            list_ref.length += 1;
        }
    }
}

pub extern "C" fn vp_list_remove_stub(list: *mut std::ffi::c_void, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx >= 0 && idx < list_ref.length {
            let val = *list_ref.data.add(idx as usize);

            // Shift elements to the left
            for i in idx..list_ref.length - 1 {
                *list_ref.data.add(i as usize) = *list_ref.data.add((i + 1) as usize);
            }

            list_ref.length -= 1;
            val
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
        let list_ref = &mut *(list as *mut ViperListStub);

        if list_ref.length == 0 {
            return 0;
        }

        list_ref.length -= 1;
        *list_ref.data.add(list_ref.length as usize)
    }
}

pub extern "C" fn vp_list_clear_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        list_ref.length = 0;
    }
}

pub extern "C" fn vp_list_contains_stub(list: *mut std::ffi::c_void, val: i64) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        for i in 0..list_ref.length as usize {
            if *list_ref.data.add(i) == val {
                return true;
            }
        }
        false
    }
}

pub extern "C" fn vp_list_print_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        print!("[null]");
        return;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        print!("[");
        for i in 0..list_ref.length as usize {
            if i > 0 {
                print!(", ");
            }
            print!("{}", *list_ref.data.add(i));
        }
        print!("]");
    }
}

// Float list stubs (f64)
pub extern "C" fn vp_list_create_f64_stub() -> *mut std::ffi::c_void {
    create_viper_list_f64_stub(8) as *mut std::ffi::c_void
}

pub extern "C" fn vp_list_append_f64_stub(list: *mut std::ffi::c_void, val: f64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListF64Stub);

        // Grow if needed
        if list_ref.length >= list_ref.capacity {
            let new_capacity = list_ref.capacity * 2;
            let new_data = libc::realloc(
                list_ref.data as *mut std::ffi::c_void,
                (new_capacity as usize) * std::mem::size_of::<f64>(),
            ) as *mut f64;
            
            if new_data.is_null() {
                panic!("Failed to grow list");
            }
            
            list_ref.data = new_data;
            list_ref.capacity = new_capacity;
        }

        // Append value
        *list_ref.data.add(list_ref.length as usize) = val;
        list_ref.length += 1;
    }
}

pub extern "C" fn vp_list_get_f64_stub(list: *mut std::ffi::c_void, index: i64) -> f64 {
    if list.is_null() {
        return 0.0;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListF64Stub);
        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx < 0 || idx >= list_ref.length {
            return 0.0;
        }

        *list_ref.data.add(idx as usize)
    }
}

pub extern "C" fn vp_list_set_f64_stub(list: *mut std::ffi::c_void, index: i64, val: f64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListF64Stub);
        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx >= 0 && idx < list_ref.length {
            *list_ref.data.add(idx as usize) = val;
        }
    }
}

pub extern "C" fn vp_range_stub(start: i64, end: i64) -> *mut std::ffi::c_void {
    let count = if end > start { end - start } else { 0 };
    let list = create_viper_list_stub(count);
    
    unsafe {
        let list_ref = &mut *list;
        for i in 0..count {
            *list_ref.data.add(i as usize) = start + i;
        }
        list_ref.length = count;
    }
    
    list as *mut std::ffi::c_void
}

// List repeat stub - creates a new list with element repeated n times
pub extern "C" fn vp_list_repeat_stub(elem: i64, count: i64) -> *mut std::ffi::c_void {
    if count <= 0 {
        return create_viper_list_stub(0) as *mut std::ffi::c_void;
    }

    let list = create_viper_list_stub(count);

    unsafe {
        let list_ref = &mut *list;

        for i in 0..count {
            *list_ref.data.add(i as usize) = elem;
        }
        list_ref.length = count;
    }

    list as *mut std::ffi::c_void
}

// List slice stub
pub extern "C" fn vp_list_slice_stub(
    list: *mut std::ffi::c_void,
    start: i64,
    end: i64,
    step: i64,
) -> *mut std::ffi::c_void {
    if list.is_null() {
        return create_viper_list_stub(0) as *mut std::ffi::c_void;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        let len = list_ref.length;

        // Normalize negative indices
        let mut s = if start < 0 { (start + len).max(0) } else { start.min(len) };
        let mut e = if end < 0 { (end + len).max(0) } else { end.min(len) };

        // Clamp to valid range
        if s < 0 { s = 0; }
        if e > len { e = len; }
        if s >= e {
            return create_viper_list_stub(0) as *mut std::ffi::c_void;
        }

        let step = if step <= 0 { 1 } else { step };
        
        // Calculate result length
        let result_len = ((e - s + step - 1) / step).max(0);
        let result = create_viper_list_stub(result_len);
        
        let result_ref = &mut *result;
        let mut j = 0;
        for i in (s..e).step_by(step as usize) {
            *result_ref.data.add(j as usize) = *list_ref.data.add(i as usize);
            j += 1;
        }
        result_ref.length = result_len;
        
        result as *mut std::ffi::c_void
    }
}

pub extern "C" fn vp_retain_stub(_ptr: *mut std::ffi::c_void) {
    // No-op for JIT
}

pub extern "C" fn vp_retain_local_stub(_ptr: *mut std::ffi::c_void) {
    // No-op for JIT (non-atomic version)
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
        let list_ref = &mut *(list as *mut ViperListStub);
        let other_ref = &*(other as *mut ViperListStub);

        // Grow if needed
        let new_length = list_ref.length + other_ref.length;
        if new_length > list_ref.capacity {
            let new_capacity = (new_length * 2).max(list_ref.capacity * 2);
            let new_data = libc::realloc(
                list_ref.data as *mut std::ffi::c_void,
                (new_capacity as usize) * std::mem::size_of::<i64>(),
            ) as *mut i64;
            
            if new_data.is_null() {
                panic!("Failed to grow list");
            }
            
            list_ref.data = new_data;
            list_ref.capacity = new_capacity;
        }

        // Copy elements
        for i in 0..other_ref.length {
            *list_ref.data.add((list_ref.length + i) as usize) = *other_ref.data.add(i as usize);
        }
        list_ref.length = new_length;
    }
}

pub extern "C" fn vp_list_index_stub(list: *mut std::ffi::c_void, val: i64) -> i64 {
    if list.is_null() {
        return -1;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        for i in 0..list_ref.length as usize {
            if *list_ref.data.add(i) == val {
                return i as i64;
            }
        }
        -1
    }
}

pub extern "C" fn vp_list_count_stub(list: *mut std::ffi::c_void, val: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        let mut count = 0;
        for i in 0..list_ref.length as usize {
            if *list_ref.data.add(i) == val {
                count += 1;
            }
        }
        count
    }
}

pub extern "C" fn vp_list_sort_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        // Use libc qsort
        unsafe extern "C" fn compare_i64(a: *const std::ffi::c_void, b: *const std::ffi::c_void) -> std::ffi::c_int {
            let va = *(a as *const i64);
            let vb = *(b as *const i64);
            if va < vb { -1 } else if va > vb { 1 } else { 0 }
        }
        libc::qsort(
            list_ref.data as *mut std::ffi::c_void,
            list_ref.length as usize,
            std::mem::size_of::<i64>(),
            Some(compare_i64),
        );
    }
}

pub extern "C" fn vp_list_reverse_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        let mut left = 0;
        let mut right = list_ref.length - 1;
        while left < right {
            let temp = *list_ref.data.add(left as usize);
            *list_ref.data.add(left as usize) = *list_ref.data.add(right as usize);
            *list_ref.data.add(right as usize) = temp;
            left += 1;
            right -= 1;
        }
    }
}

pub extern "C" fn vp_list_copy_stub(list: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if list.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        let copy = create_viper_list_stub(list_ref.capacity);
        let copy_ref = &mut *copy;
        
        // Copy elements
        for i in 0..list_ref.length {
            *copy_ref.data.add(i as usize) = *list_ref.data.add(i as usize);
        }
        copy_ref.length = list_ref.length;
        
        copy as *mut std::ffi::c_void
    }
}

pub extern "C" fn vp_list_grow_stub(_list: *mut std::ffi::c_void) {
    // No-op for JIT - handled in append/insert
}

pub extern "C" fn vp_list_reserve_stub(list: *mut std::ffi::c_void, capacity: i64) {
    if list.is_null() || capacity <= 0 {
        return;
    }
    unsafe {
        let list_ref = &mut *(list as *mut ViperListStub);
        if capacity > list_ref.capacity {
            let new_data = libc::realloc(
                list_ref.data as *mut std::ffi::c_void,
                (capacity as usize) * std::mem::size_of::<i64>(),
            ) as *mut i64;
            
            if new_data.is_null() {
                panic!("Failed to reserve list capacity");
            }
            
            list_ref.data = new_data;
            list_ref.capacity = capacity;
        }
    }
}

pub extern "C" fn vp_list_concat_stub(
    list1: *mut std::ffi::c_void,
    list2: *mut std::ffi::c_void,
) -> *mut std::ffi::c_void {
    let mut total_len = 0i64;
    
    if !list1.is_null() {
        unsafe {
            let list1_ref = &*(list1 as *mut ViperListStub);
            total_len += list1_ref.length;
        }
    }
    if !list2.is_null() {
        unsafe {
            let list2_ref = &*(list2 as *mut ViperListStub);
            total_len += list2_ref.length;
        }
    }

    let result = create_viper_list_stub(total_len);
    
    unsafe {
        let result_ref = &mut *result;
        let mut j = 0;
        
        if !list1.is_null() {
            let list1_ref = &*(list1 as *mut ViperListStub);
            for i in 0..list1_ref.length {
                *result_ref.data.add(j as usize) = *list1_ref.data.add(i as usize);
                j += 1;
            }
        }
        
        if !list2.is_null() {
            let list2_ref = &*(list2 as *mut ViperListStub);
            for i in 0..list2_ref.length {
                *result_ref.data.add(j as usize) = *list2_ref.data.add(i as usize);
                j += 1;
            }
        }
        
        result_ref.length = total_len;
    }
    
    result as *mut std::ffi::c_void
}

pub extern "C" fn vp_list_sorted_stub(list: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if list.is_null() {
        return create_viper_list_stub(0) as *mut std::ffi::c_void;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        let copy = create_viper_list_stub(list_ref.capacity);
        let copy_ref = &mut *copy;
        
        // Copy elements
        for i in 0..list_ref.length {
            *copy_ref.data.add(i as usize) = *list_ref.data.add(i as usize);
        }
        copy_ref.length = list_ref.length;
        
        // Sort
        unsafe extern "C" fn compare_i64(a: *const std::ffi::c_void, b: *const std::ffi::c_void) -> std::ffi::c_int {
            let va = *(a as *const i64);
            let vb = *(b as *const i64);
            if va < vb { -1 } else if va > vb { 1 } else { 0 }
        }
        libc::qsort(
            copy_ref.data as *mut std::ffi::c_void,
            copy_ref.length as usize,
            std::mem::size_of::<i64>(),
            Some(compare_i64),
        );
        
        copy as *mut std::ffi::c_void
    }
}

pub extern "C" fn vp_list_reversed_stub(list: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
    if list.is_null() {
        return create_viper_list_stub(0) as *mut std::ffi::c_void;
    }
    unsafe {
        let list_ref = &*(list as *mut ViperListStub);
        let copy = create_viper_list_stub(list_ref.capacity);
        let copy_ref = &mut *copy;
        
        // Copy elements in reverse order
        for i in 0..list_ref.length {
            *copy_ref.data.add(i as usize) = *list_ref.data.add((list_ref.length - 1 - i) as usize);
        }
        copy_ref.length = list_ref.length;
        
        copy as *mut std::ffi::c_void
    }
}
