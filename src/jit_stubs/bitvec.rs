// Stub implementations for bit vector functions (JIT mode)
// Bit vectors use 1 bit per boolean for 8x memory savings

use std::ffi::c_void;

// Match the ViperList struct layout from viper_types.h
#[repr(C)]
pub struct ViperList {
    pub ref_count: i64,
    pub length: i64,
    pub capacity: i64,
    pub elem_type: i64,    // ViperListType (i32 but aligned to 8 bytes)
    pub data: *mut c_void, // Union pointer - for bit vectors: *mut Vec<u64>
}

const VIPER_LIST_BITVEC: i64 = 7; // Must match viper_types.h

/// Get the word index for a given bit index
fn word_index(bit_index: i64) -> usize {
    (bit_index / 64) as usize
}

/// Get the bit mask for a given bit index
fn bit_mask(bit_index: i64) -> u64 {
    1u64 << (bit_index % 64)
}

/// Calculate number of words needed for n bits
fn words_needed(n_bits: i64) -> usize {
    ((n_bits + 63) / 64) as usize
}

/// Create a bit vector - JIT stub
pub extern "C" fn vp_bitvec_create_stub() -> *mut ViperList {
    let capacity = 64i64;
    let words = words_needed(capacity);
    let mut vec = Vec::<u64>::with_capacity(words);
    vec.resize(words, 0u64);
    let data_ptr = Box::into_raw(Box::new(vec)) as *mut c_void;

    let list = Box::new(ViperList {
        ref_count: 1,
        length: 0,
        capacity,
        elem_type: VIPER_LIST_BITVEC,
        data: data_ptr,
    });

    Box::into_raw(list)
}

/// Create a bit vector with capacity - JIT stub
pub extern "C" fn vp_bitvec_create_with_capacity_stub(cap: i64) -> *mut ViperList {
    let capacity = if cap > 0 { cap } else { 64 };
    let words = words_needed(capacity);
    let mut vec = Vec::<u64>::with_capacity(words);
    vec.resize(words, 0u64);
    let data_ptr = Box::into_raw(Box::new(vec)) as *mut c_void;

    let list = Box::new(ViperList {
        ref_count: 1,
        length: 0,
        capacity,
        elem_type: VIPER_LIST_BITVEC,
        data: data_ptr,
    });

    Box::into_raw(list)
}

/// Create a bit vector with all elements set to the same value - JIT stub
pub extern "C" fn vp_bitvec_repeat_stub(elem: bool, count: i64) -> *mut ViperList {
    let words = words_needed(count);
    let mut vec = Vec::<u64>::with_capacity(words);

    if elem {
        // Set all bits to 1
        for _ in 0..words {
            vec.push(u64::MAX);
        }
        // Clear extra bits in the last word
        if count > 0 {
            let extra_bits = (words as i64) * 64 - count;
            if extra_bits > 0 {
                let mask = u64::MAX >> extra_bits;
                vec[words - 1] &= mask;
            }
        }
    } else {
        // All bits 0
        vec.resize(words, 0u64);
    }

    let data_ptr = Box::into_raw(Box::new(vec)) as *mut c_void;

    let list = Box::new(ViperList {
        ref_count: 1,
        length: count,
        capacity: count,
        elem_type: VIPER_LIST_BITVEC,
        data: data_ptr,
    });

    Box::into_raw(list)
}

/// Free a bit vector - JIT stub
pub extern "C" fn vp_bitvec_free_stub(list: *mut ViperList) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        let _ = Box::from_raw(list_ref.data as *mut Vec<u64>);
        let _ = Box::from_raw(list);
    }
}

/// Append to bit vector - JIT stub
pub extern "C" fn vp_bitvec_append_stub(list: *mut ViperList, value: bool) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        let vec = &mut *(list_ref.data as *mut Vec<u64>);

        // Grow if needed
        if list_ref.length >= list_ref.capacity {
            let new_capacity = list_ref.capacity * 2;
            let new_words = words_needed(new_capacity);
            vec.resize(new_words, 0u64);
            list_ref.capacity = new_capacity;
        }

        // Set the bit
        let word_idx = word_index(list_ref.length);
        let mask = bit_mask(list_ref.length);

        if value {
            vec[word_idx as usize] |= mask;
        } else {
            vec[word_idx as usize] &= !mask;
        }

        list_ref.length += 1;
    }
}

/// Get element from bit vector - JIT stub
pub extern "C" fn vp_bitvec_get_stub(list: *mut ViperList, index: i64) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let list_ref = &*list;
        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx < 0 || idx >= list_ref.length {
            return false;
        }

        let vec = &*(list_ref.data as *mut Vec<u64>);
        let word_idx = word_index(idx);
        let mask = bit_mask(idx);

        (vec[word_idx] & mask) != 0
    }
}

/// Set element in bit vector - JIT stub
pub extern "C" fn vp_bitvec_set_stub(list: *mut ViperList, index: i64, value: bool) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        let mut idx = index;

        // Handle negative indexing
        if idx < 0 {
            idx = list_ref.length + idx;
        }

        if idx >= 0 && idx < list_ref.length {
            let vec = &mut *(list_ref.data as *mut Vec<u64>);
            let word_idx = word_index(idx);
            let mask = bit_mask(idx);

            if value {
                vec[word_idx] |= mask;
            } else {
                vec[word_idx] &= !mask;
            }
        }
    }
}

/// Insert element into bit vector - JIT stub
pub extern "C" fn vp_bitvec_insert_stub(list: *mut ViperList, index: i64, value: bool) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;

        if index < 0 || index > list_ref.length {
            return;
        }

        let data_vec = &mut *(list_ref.data as *mut Vec<u64>);

        // Grow if needed
        if list_ref.length >= list_ref.capacity {
            let new_capacity = list_ref.capacity * 2;
            let new_words = words_needed(new_capacity);
            data_vec.resize(new_words, 0u64);
            list_ref.capacity = new_capacity;
        }

        // Shift bits to the right
        for i in (index..list_ref.length).rev() {
            let bit = (data_vec[word_index(i) as usize] & bit_mask(i)) != 0;
            let new_word_idx = word_index(i + 1);
            let new_mask = bit_mask(i + 1);

            // Ensure we have enough words
            if new_word_idx as usize >= data_vec.len() {
                data_vec.resize((new_word_idx + 1) as usize, 0u64);
            }

            if bit {
                data_vec[new_word_idx as usize] |= new_mask;
            } else {
                data_vec[new_word_idx as usize] &= !new_mask;
            }
        }

        // Set the new bit
        let word_idx = word_index(index);
        let mask = bit_mask(index);

        if value {
            data_vec[word_idx as usize] |= mask;
        } else {
            data_vec[word_idx as usize] &= !mask;
        }

        list_ref.length += 1;
    }
}

/// Remove element from bit vector - JIT stub
pub extern "C" fn vp_bitvec_remove_stub(list: *mut ViperList, index: i64) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let list_ref = &mut *list;

        if index < 0 || index >= list_ref.length {
            return false;
        }

        let vec = &mut *(list_ref.data as *mut Vec<u64>);
        let value = (vec[word_index(index) as usize] & bit_mask(index)) != 0;

        // Shift bits to the left
        for i in index..list_ref.length - 1 {
            let bit = (vec[word_index(i + 1) as usize] & bit_mask(i + 1)) != 0;
            let word_idx = word_index(i);
            let mask = bit_mask(i);

            if bit {
                vec[word_idx as usize] |= mask;
            } else {
                vec[word_idx as usize] &= !mask;
            }
        }

        // Clear the last bit
        list_ref.length -= 1;
        let last_word = word_index(list_ref.length);
        let last_mask = bit_mask(list_ref.length);
        vec[last_word as usize] &= !last_mask;

        value
    }
}

/// Pop element from bit vector - JIT stub
pub extern "C" fn vp_bitvec_pop_stub(list: *mut ViperList) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let list_ref = &mut *list;

        if list_ref.length == 0 {
            return false;
        }

        list_ref.length -= 1;
        vp_bitvec_get_stub(list, list_ref.length)
    }
}

/// Clear bit vector - JIT stub
pub extern "C" fn vp_bitvec_clear_stub(list: *mut ViperList) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        let vec = &mut *(list_ref.data as *mut Vec<u64>);
        for word in vec.iter_mut() {
            *word = 0;
        }
        list_ref.length = 0;
    }
}

/// Check if bit vector contains value - JIT stub
pub extern "C" fn vp_bitvec_contains_stub(list: *mut ViperList, value: bool) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let list_ref = &*list;
        let vec = &*(list_ref.data as *mut Vec<u64>);
        let words = words_needed(list_ref.length);

        if value {
            // Check if any bit is set
            for i in 0..words {
                if vec[i] != 0 {
                    return true;
                }
            }
            false
        } else {
            // Check if any bit is clear
            for i in 0..words - 1 {
                if vec[i] != u64::MAX {
                    return true;
                }
            }
            // Check last word with only relevant bits
            let extra_bits = (words as i64) * 64 - list_ref.length;
            let mask = if extra_bits > 0 { u64::MAX >> extra_bits } else { u64::MAX };
            (vec[words - 1] & mask) != mask
        }
    }
}

/// Copy bit vector - JIT stub
pub extern "C" fn vp_bitvec_copy_stub(list: *mut ViperList) -> *mut ViperList {
    if list.is_null() {
        return std::ptr::null_mut();
    }
    unsafe {
        let list_ref = &*list;
        let vec = &*(list_ref.data as *mut Vec<u64>);
        let new_vec = Box::new(vec.clone());
        let data_ptr = Box::into_raw(new_vec) as *mut c_void;

        let new_list = Box::new(ViperList {
            ref_count: 1,
            length: list_ref.length,
            capacity: list_ref.capacity,
            elem_type: VIPER_LIST_BITVEC,
            data: data_ptr,
        });

        Box::into_raw(new_list)
    }
}

/// Slice bit vector - JIT stub
pub extern "C" fn vp_bitvec_slice_stub(
    list: *mut ViperList,
    start: i64,
    end: i64,
    step: i64,
) -> *mut ViperList {
    if list.is_null() {
        return vp_bitvec_create_stub();
    }
    unsafe {
        let list_ref = &*list;
        let mut s = start;
        let mut e = end;

        // Handle negative indices
        if s < 0 {
            s = list_ref.length + s;
        }
        if e < 0 {
            e = list_ref.length + e;
        }

        // Clamp to valid range
        if s < 0 {
            s = 0;
        }
        if e > list_ref.length {
            e = list_ref.length;
        }
        if s >= e {
            return vp_bitvec_create_stub();
        }

        let mut step_val = step;
        if step_val == 0 {
            step_val = 1;
        }

        let mut result_vec = Vec::<u64>::new();
        let mut result_len = 0i64;

        if step_val > 0 {
            let mut i = s;
            while i < e {
                let bit = vp_bitvec_get_stub(list, i);
                // Grow result vector if needed
                if result_len >= (result_vec.len() as i64) * 64 {
                    result_vec.push(0u64);
                }
                if bit {
                    let word_idx = word_index(result_len);
                    let mask = bit_mask(result_len);
                    result_vec[word_idx] |= mask;
                }
                result_len += 1;
                i += step_val;
            }
        } else {
            let mut i = e - 1;
            while i >= s {
                let bit = vp_bitvec_get_stub(list, i);
                if result_len >= (result_vec.len() as i64) * 64 {
                    result_vec.push(0u64);
                }
                if bit {
                    let word_idx = word_index(result_len);
                    let mask = bit_mask(result_len);
                    result_vec[word_idx] |= mask;
                }
                result_len += 1;
                i += step_val;
            }
        }

        let data_ptr = Box::into_raw(Box::new(result_vec)) as *mut c_void;

        let result_list = Box::new(ViperList {
            ref_count: 1,
            length: result_len,
            capacity: result_len,
            elem_type: VIPER_LIST_BITVEC,
            data: data_ptr,
        });

        Box::into_raw(result_list)
    }
}

/// Print bit vector - JIT stub
pub extern "C" fn vp_bitvec_print_stub(list: *mut ViperList) {
    if list.is_null() {
        print!("(null)");
        return;
    }
    unsafe {
        let list_ref = &*list;
        print!("[");
        for i in 0..list_ref.length {
            if i > 0 {
                print!(", ");
            }
            let bit = vp_bitvec_get_stub(list, i);
            print!("{}", if bit { "True" } else { "False" });
        }
        print!("]");
    }
}

/// Get bit vector length - JIT stub
pub extern "C" fn vp_bitvec_len_stub(list: *mut ViperList) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let list_ref = &*list;
        list_ref.length
    }
}

/* Unchecked versions for hot loops - no bounds checking */
pub extern "C" fn vp_bitvec_get_unchecked_stub(list: *mut ViperList, index: i64) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let list_ref = &*list;
        let word_idx = (index / 64) as usize;
        let mask = 1u64 << (index % 64);
        /* For JIT, data points to Vec<u64>, need to deref */
        let vec = &*(list_ref.data as *mut Vec<u64>);
        (vec[word_idx] & mask) != 0
    }
}

pub extern "C" fn vp_bitvec_set_unchecked_stub(list: *mut ViperList, index: i64, value: bool) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        let word_idx = (index / 64) as usize;
        let mask = 1u64 << (index % 64);
        /* For JIT, data points to Vec<u64>, need to deref */
        let vec = &mut *(list_ref.data as *mut Vec<u64>);
        if value {
            vec[word_idx] |= mask;
        } else {
            vec[word_idx] &= !mask;
        }
    }
}

/// Extend bit vector with another - JIT stub
pub extern "C" fn vp_bitvec_extend_stub(list: *mut ViperList, other: *mut ViperList) {
    if list.is_null() || other.is_null() {
        return;
    }
    unsafe {
        let other_ref = &*other;
        for i in 0..other_ref.length {
            let bit = vp_bitvec_get_stub(other, i);
            vp_bitvec_append_stub(list, bit);
        }
    }
}

/// Find index of value in bit vector - JIT stub
pub extern "C" fn vp_bitvec_index_stub(list: *mut ViperList, value: bool) -> i64 {
    if list.is_null() {
        return -1;
    }
    unsafe {
        let list_ref = &*list;
        for i in 0..list_ref.length {
            if vp_bitvec_get_stub(list, i) == value {
                return i;
            }
        }
        -1
    }
}

/// Count occurrences of value in bit vector - JIT stub
pub extern "C" fn vp_bitvec_count_stub(list: *mut ViperList, value: bool) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let list_ref = &*list;
        let mut count = 0i64;
        for i in 0..list_ref.length {
            if vp_bitvec_get_stub(list, i) == value {
                count += 1;
            }
        }
        count
    }
}

/// Reverse bit vector in place - JIT stub
pub extern "C" fn vp_bitvec_reverse_stub(list: *mut ViperList) {
    if list.is_null() {
        return;
    }
    unsafe {
        let list_ref = &mut *list;
        let mut left = 0i64;
        let mut right = list_ref.length - 1;

        while left < right {
            let left_val = vp_bitvec_get_stub(list, left);
            let right_val = vp_bitvec_get_stub(list, right);
            vp_bitvec_set_stub(list, left, right_val);
            vp_bitvec_set_stub(list, right, left_val);
            left += 1;
            right -= 1;
        }
    }
}

/// Create reversed copy of bit vector - JIT stub
pub extern "C" fn vp_bitvec_reversed_stub(list: *mut ViperList) -> *mut ViperList {
    if list.is_null() {
        return std::ptr::null_mut();
    }
    let result = vp_bitvec_copy_stub(list);
    vp_bitvec_reverse_stub(result);
    result
}

/// Concatenate two bit vectors - JIT stub
pub extern "C" fn vp_bitvec_concat_stub(
    vec1: *mut ViperList,
    vec2: *mut ViperList,
) -> *mut ViperList {
    if vec1.is_null() || vec2.is_null() {
        return std::ptr::null_mut();
    }
    let result = vp_bitvec_copy_stub(vec1);
    vp_bitvec_extend_stub(result, vec2);
    result
}
