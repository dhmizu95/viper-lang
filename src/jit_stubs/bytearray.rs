// Stub implementations for bytearray functions (JIT mode)
// ViperByteArray layout (24 bytes):
//   offset 0-7: length (i64)
//   offset 8-15: capacity (i64)
//   offset 16-23: data pointer (*mut u8)

const BYTEARRAY_INITIAL_CAPACITY: i64 = 64;

#[repr(C)]
pub struct ViperByteArrayStub {
    pub length: i64,
    pub capacity: i64,
    pub data: *mut u8,
}

fn create_bytearray_stub(capacity: i64) -> *mut ViperByteArrayStub {
    let capacity = if capacity > 0 { capacity } else { BYTEARRAY_INITIAL_CAPACITY };

    // Allocate data buffer
    let data_ptr = unsafe {
        libc::malloc(capacity as usize) as *mut u8
    };

    if data_ptr.is_null() {
        panic!("Failed to allocate bytearray data");
    }

    // Initialize to zero
    unsafe {
        std::ptr::write_bytes(data_ptr, 0, capacity as usize);
    }

    let ba = Box::new(ViperByteArrayStub {
        length: 0,
        capacity,
        data: data_ptr,
    });

    Box::into_raw(ba)
}

pub extern "C" fn vp_bytearray_create() -> *mut ViperByteArrayStub {
    create_bytearray_stub(BYTEARRAY_INITIAL_CAPACITY)
}

pub extern "C" fn vp_bytearray_create_with_capacity(cap: i64) -> *mut ViperByteArrayStub {
    create_bytearray_stub(cap)
}

pub extern "C" fn vp_bytearray_len(ba: *mut ViperByteArrayStub) -> i64 {
    if ba.is_null() {
        return 0;
    }
    unsafe { (*ba).length }
}

pub extern "C" fn vp_bytearray_append(ba: *mut ViperByteArrayStub, value: i64) {
    if ba.is_null() {
        return;
    }

    let ba_ref = unsafe { &mut *ba };

    // Ensure capacity
    if ba_ref.length >= ba_ref.capacity {
        let new_capacity = ba_ref.capacity * 2;
        let new_data = unsafe {
            libc::realloc(ba_ref.data as *mut libc::c_void, new_capacity as usize) as *mut u8
        };
        if new_data.is_null() {
            panic!("Failed to grow bytearray");
        }
        ba_ref.data = new_data;
        ba_ref.capacity = new_capacity;
    }

    // Append value (clamp to 0-255)
    let byte_value = ((value as u32) & 0xFF) as u8;
    unsafe {
        *ba_ref.data.add(ba_ref.length as usize) = byte_value;
    }
    ba_ref.length += 1;
}

pub extern "C" fn vp_bytearray_get(ba: *mut ViperByteArrayStub, index: i64) -> i64 {
    if ba.is_null() {
        return 0;
    }

    let ba_ref = unsafe { &*ba };

    if index < 0 || index >= ba_ref.length {
        eprintln!("bytearray index out of range: {}", index);
        return 0;
    }

    unsafe { *ba_ref.data.add(index as usize) as i64 }
}

pub extern "C" fn vp_bytearray_set(ba: *mut ViperByteArrayStub, index: i64, value: i64) {
    if ba.is_null() {
        return;
    }

    let ba_ref = unsafe { &mut *ba };

    if index < 0 || index >= ba_ref.length {
        eprintln!("bytearray index out of range: {}", index);
        return;
    }

    let byte_value = ((value as u32) & 0xFF) as u8;
    unsafe {
        *ba_ref.data.add(index as usize) = byte_value;
    }
}

pub extern "C" fn vp_bytearray_print(ba: *mut ViperByteArrayStub) {
    if ba.is_null() {
        print!("bytearray()");
        return;
    }

    let ba_ref = unsafe { &*ba };
    print!("bytearray(b\"");

    for i in 0..ba_ref.length {
        let c = unsafe { *ba_ref.data.add(i as usize) };
        if c >= 32 && c < 127 && c != b'"' && c != b'\\' {
            print!("{}", c as char);
        } else {
            print!("\\x{:02x}", c);
        }
    }

    println!("\")");
}

pub extern "C" fn vp_bytearray_repeat(ba: *mut ViperByteArrayStub, count: i64) -> *mut ViperByteArrayStub {
    if ba.is_null() || count <= 0 {
        return create_bytearray_stub(0);
    }

    let ba_ref = unsafe { &*ba };
    let orig_len = ba_ref.length;
    let new_len = orig_len * count;

    let result = create_bytearray_stub(new_len);
    let result_ref = unsafe { &mut *result };

    // Copy data count times
    for i in 0..count {
        unsafe {
            std::ptr::copy_nonoverlapping(
                ba_ref.data,
                result_ref.data.add((i * orig_len) as usize),
                orig_len as usize,
            );
        }
    }
    result_ref.length = new_len;

    result
}

pub extern "C" fn vp_bytearray_slice(
    ba: *mut ViperByteArrayStub,
    start: i64,
    end: i64,
    step: i64,
) -> *mut ViperByteArrayStub {
    if ba.is_null() {
        return create_bytearray_stub(0);
    }

    let ba_ref = unsafe { &*ba };
    let len = ba_ref.length;

    // Normalize negative indices
    let mut start = if start < 0 { len + start } else { start };
    let mut end = if end < 0 { len + end } else { end };

    if start < 0 {
        start = 0;
    }
    if end > len {
        end = len;
    }
    if start >= end || step <= 0 {
        return create_bytearray_stub(0);
    }

    // Calculate result length
    let result_len = (end - start + step - 1) / step;
    let result = create_bytearray_stub(result_len);
    let result_ref = unsafe { &mut *result };

    // Copy elements
    let mut src_idx = start;
    let mut dst_idx = 0;
    while src_idx < end && dst_idx < result_len {
        unsafe {
            *result_ref.data.add(dst_idx as usize) = *ba_ref.data.add(src_idx as usize);
        }
        src_idx += step;
        dst_idx += 1;
    }
    result_ref.length = dst_idx;

    result
}

pub extern "C" fn vp_bytearray_free(ba: *mut ViperByteArrayStub) {
    if ba.is_null() {
        return;
    }

    let ba_ref = unsafe { &*ba };
    unsafe {
        libc::free(ba_ref.data as *mut libc::c_void);
    }

    let _ = unsafe { Box::from_raw(ba) };
}

pub extern "C" fn vp_enumerate_bytearray(ba: *mut ViperByteArrayStub, start: i64) -> *mut std::ffi::c_void {
    // For now, return null - full implementation would create list of tuples
    // This is a simplified stub for JIT mode
    eprintln!("enumerate(bytearray) not fully implemented in JIT stubs");
    std::ptr::null_mut()
}
