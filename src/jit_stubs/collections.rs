// Collections module stubs for JIT - Phase 2
// Deque, Counter, OrderedDict, defaultdict, NamedTuple

use std::collections::VecDeque;
use std::collections::HashMap;

/* ============================================ */
// Deque (Double-ended queue)
/* ============================================ */

pub type ViperDeque = VecDeque<i64>;

#[no_mangle]
pub extern "C" fn vp_deque_create() -> *mut ViperDeque {
    let dq = Box::new(ViperDeque::new());
    Box::into_raw(dq)
}

#[no_mangle]
pub extern "C" fn vp_deque_free(dq: *mut ViperDeque) {
    if !dq.is_null() {
        unsafe { drop(Box::from_raw(dq)); }
    }
}

#[no_mangle]
pub extern "C" fn vp_deque_append(dq: *mut ViperDeque, value: i64) {
    if !dq.is_null() {
        unsafe { (*dq).push_back(value); }
    }
}

#[no_mangle]
pub extern "C" fn vp_deque_appendleft(dq: *mut ViperDeque, value: i64) {
    if !dq.is_null() {
        unsafe { (*dq).push_front(value); }
    }
}

#[no_mangle]
pub extern "C" fn vp_deque_pop(dq: *mut ViperDeque) -> i64 {
    if dq.is_null() {
        return 0;
    }
    unsafe { (*dq).pop_back().unwrap_or(0) }
}

#[no_mangle]
pub extern "C" fn vp_deque_popleft(dq: *mut ViperDeque) -> i64 {
    if dq.is_null() {
        return 0;
    }
    unsafe { (*dq).pop_front().unwrap_or(0) }
}

#[no_mangle]
pub extern "C" fn vp_deque_get(dq: *mut ViperDeque, index: i64) -> i64 {
    if dq.is_null() || index < 0 {
        return 0;
    }
    unsafe { (*dq).get(index as usize).copied().unwrap_or(0) }
}

#[no_mangle]
pub extern "C" fn vp_deque_len(dq: *mut ViperDeque) -> i64 {
    if dq.is_null() {
        return 0;
    }
    unsafe { (*dq).len() as i64 }
}

#[no_mangle]
pub extern "C" fn vp_deque_clear(dq: *mut ViperDeque) {
    if !dq.is_null() {
        unsafe { (*dq).clear(); }
    }
}

#[no_mangle]
pub extern "C" fn vp_deque_rotate(dq: *mut ViperDeque, n: i64) {
    if dq.is_null() {
        return;
    }
    unsafe {
        let len = (*dq).len();
        if len == 0 { return; }
        let n = n.rem_euclid(len as i64) as usize;
        if n > 0 {
            (*dq).rotate_right(n);
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_deque_reverse(dq: *mut ViperDeque) {
    if dq.is_null() {
        return;
    }
    unsafe {
        let len = (*dq).len();
        for i in 0..len / 2 {
            let j = len - 1 - i;
            if let (Some(a), Some(b)) = ((*dq).get_mut(i), (*dq).get_mut(j)) {
                std::mem::swap(a, b);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_deque_remove(dq: *mut ViperDeque, value: i64) -> i64 {
    if dq.is_null() {
        return 0;
    }
    unsafe {
        if let Some(pos) = (*dq).iter().position(|&x| x == value) {
            (*dq).remove(pos);
            return 1;
        }
    }
    0
}

#[no_mangle]
pub extern "C" fn vp_deque_count(dq: *mut ViperDeque, value: i64) -> i64 {
    if dq.is_null() {
        return 0;
    }
    unsafe { (*dq).iter().filter(|&&x| x == value).count() as i64 }
}

#[no_mangle]
pub extern "C" fn vp_deque_contains(dq: *mut ViperDeque, value: i64) -> i64 {
    if dq.is_null() {
        return 0;
    }
    unsafe {
        if (*dq).contains(&value) { 1 } else { 0 }
    }
}

#[no_mangle]
pub extern "C" fn vp_deque_insert(dq: *mut ViperDeque, index: i64, value: i64) {
    if dq.is_null() || index < 0 {
        return;
    }
    unsafe {
        let index = index as usize;
        let len = (*dq).len();
        if index <= len {
            (*dq).insert(index, value);
        }
    }
}

/* ============================================ */
// Counter (Frequency counter)
/* ============================================ */

pub type ViperCounter = HashMap<String, i64>;

#[no_mangle]
pub extern "C" fn vp_counter_create() -> *mut ViperCounter {
    let counter = Box::new(ViperCounter::new());
    Box::into_raw(counter)
}

#[no_mangle]
pub extern "C" fn vp_counter_free(counter: *mut ViperCounter) {
    if !counter.is_null() {
        unsafe { drop(Box::from_raw(counter)); }
    }
}

#[no_mangle]
pub extern "C" fn vp_counter_add(counter: *mut ViperCounter, key: *const i8, count: i64) {
    if counter.is_null() || key.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            let entry = (*counter).entry(key_str.to_string()).or_insert(0);
            *entry += count;
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_counter_get(counter: *mut ViperCounter, key: *const i8) -> i64 {
    if counter.is_null() || key.is_null() {
        return 0;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            *(*counter).get(key_str).unwrap_or(&0)
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_counter_set(counter: *mut ViperCounter, key: *const i8, count: i64) {
    if counter.is_null() || key.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            (*counter).insert(key_str.to_string(), count);
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_counter_total(counter: *mut ViperCounter) -> i64 {
    if counter.is_null() {
        return 0;
    }
    unsafe { (*counter).values().sum() }
}

#[no_mangle]
pub extern "C" fn vp_counter_len(counter: *mut ViperCounter) -> i64 {
    if counter.is_null() {
        return 0;
    }
    unsafe { (*counter).len() as i64 }
}

#[no_mangle]
pub extern "C" fn vp_counter_clear(counter: *mut ViperCounter) {
    if !counter.is_null() {
        unsafe { (*counter).clear(); }
    }
}

/* ============================================ */
// OrderedDict (Dict with insertion order)
/* ============================================ */

pub struct ViperOrderedDict {
    map: HashMap<String, i64>,
    order: Vec<String>,
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_create() -> *mut ViperOrderedDict {
    let od = Box::new(ViperOrderedDict {
        map: HashMap::new(),
        order: Vec::new(),
    });
    Box::into_raw(od)
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_free(od: *mut ViperOrderedDict) {
    if !od.is_null() {
        unsafe { drop(Box::from_raw(od)); }
    }
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_set(od: *mut ViperOrderedDict, key: *const i8, value: i64) {
    if od.is_null() || key.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            let key_str = key_str.to_string();
            if !(*od).map.contains_key(&key_str) {
                (*od).order.push(key_str.clone());
            }
            (*od).map.insert(key_str, value);
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_get(od: *mut ViperOrderedDict, key: *const i8) -> i64 {
    if od.is_null() || key.is_null() {
        return 0;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            *(*od).map.get(key_str).unwrap_or(&0)
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_contains(od: *mut ViperOrderedDict, key: *const i8) -> i64 {
    if od.is_null() || key.is_null() {
        return 0;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            if (*od).map.contains_key(key_str) { 1 } else { 0 }
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_len(od: *mut ViperOrderedDict) -> i64 {
    if od.is_null() {
        return 0;
    }
    unsafe { (*od).map.len() as i64 }
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_clear(od: *mut ViperOrderedDict) {
    if !od.is_null() {
        unsafe {
            (*od).map.clear();
            (*od).order.clear();
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_keys(_od: *mut ViperOrderedDict) -> *mut std::ffi::c_void {
    // Returns a list of keys - simplified, returns null for now
    // Full implementation would create ViperList
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_values(_od: *mut ViperOrderedDict) -> *mut std::ffi::c_void {
    // Returns a list of values - simplified, returns null for now
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn vp_ordered_dict_move_to_end(od: *mut ViperOrderedDict, key: *const i8, last: i64) {
    if od.is_null() || key.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            if !(*od).map.contains_key(key_str) {
                return;
            }
            if let Some(pos) = (*od).order.iter().position(|k| k == key_str) {
                (*od).order.remove(pos);
                if last != 0 {
                    (*od).order.push(key_str.to_string());
                } else {
                    (*od).order.insert(0, key_str.to_string());
                }
            }
        }
    }
}

/* ============================================ */
// Defaultdict
/* ============================================ */

pub struct ViperDefaultDict {
    map: HashMap<String, i64>,
    default_value: i64,
}

#[no_mangle]
pub extern "C" fn vp_default_dict_create(default_value: i64) -> *mut ViperDefaultDict {
    let dd = Box::new(ViperDefaultDict {
        map: HashMap::new(),
        default_value,
    });
    Box::into_raw(dd)
}

#[no_mangle]
pub extern "C" fn vp_default_dict_free(dd: *mut ViperDefaultDict) {
    if !dd.is_null() {
        unsafe { drop(Box::from_raw(dd)); }
    }
}

#[no_mangle]
pub extern "C" fn vp_default_dict_get(dd: *mut ViperDefaultDict, key: *const i8) -> i64 {
    if dd.is_null() || key.is_null() {
        return 0;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            *(*dd).map.get(key_str).unwrap_or(&(*dd).default_value)
        } else {
            (*dd).default_value
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_default_dict_set(dd: *mut ViperDefaultDict, key: *const i8, value: i64) {
    if dd.is_null() || key.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(key);
        if let Ok(key_str) = c_str.to_str() {
            (*dd).map.insert(key_str.to_string(), value);
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_default_dict_len(dd: *mut ViperDefaultDict) -> i64 {
    if dd.is_null() {
        return 0;
    }
    unsafe { (*dd).map.len() as i64 }
}

/* ============================================ */
// NamedTuple (simplified)
/* ============================================ */

pub struct ViperNamedTuple {
    fields: Vec<String>,
    values: Vec<i64>,
}

#[no_mangle]
pub extern "C" fn vp_named_tuple_create(size: i64) -> *mut ViperNamedTuple {
    let nt = Box::new(ViperNamedTuple {
        fields: Vec::with_capacity(size as usize),
        values: vec![0; size as usize],
    });
    Box::into_raw(nt)
}

#[no_mangle]
pub extern "C" fn vp_named_tuple_free(nt: *mut ViperNamedTuple) {
    if !nt.is_null() {
        unsafe { drop(Box::from_raw(nt)); }
    }
}

#[no_mangle]
pub extern "C" fn vp_named_tuple_set_field(nt: *mut ViperNamedTuple, index: i64, name: *const i8) {
    if nt.is_null() || name.is_null() || index < 0 {
        return;
    }
    unsafe {
        let nt_ref = &mut *nt;
        let c_str = std::ffi::CStr::from_ptr(name);
        if let Ok(name_str) = c_str.to_str() {
            if index as usize >= nt_ref.fields.len() {
                nt_ref.fields.resize(index as usize + 1, String::new());
            }
            nt_ref.fields[index as usize] = name_str.to_string();
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_named_tuple_set_value(nt: *mut ViperNamedTuple, index: i64, value: i64) {
    if nt.is_null() || index < 0 {
        return;
    }
    unsafe {
        let nt_ref = &mut *nt;
        if (index as usize) < nt_ref.values.len() {
            nt_ref.values[index as usize] = value;
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_named_tuple_get_value(nt: *mut ViperNamedTuple, index: i64) -> i64 {
    if nt.is_null() || index < 0 {
        return 0;
    }
    unsafe {
        let nt_ref = &*nt;
        if (index as usize) < nt_ref.values.len() {
            nt_ref.values[index as usize]
        } else {
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn vp_named_tuple_len(nt: *mut ViperNamedTuple) -> i64 {
    if nt.is_null() {
        return 0;
    }
    unsafe { (&*nt).values.len() as i64 }
}
