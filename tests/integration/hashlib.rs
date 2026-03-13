use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe extern "C" {
    fn vp_hash_md5(data: *const c_char, len: i64) -> *mut c_char;
    fn vp_hash_sha256(data: *const c_char, len: i64) -> *mut c_char;
    fn vp_hash_sha512(data: *const c_char, len: i64) -> *mut c_char;
}

fn digest(
    func: unsafe extern "C" fn(*const c_char, i64) -> *mut c_char,
    input: &str,
) -> String {
    let input = CString::new(input).expect("input should not contain NUL bytes");
    let ptr = unsafe { func(input.as_ptr(), input.as_bytes().len() as i64) };
    assert!(!ptr.is_null(), "hash function returned null");

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .expect("digest should be valid UTF-8 hex")
        .to_owned()
}

#[test]
fn test_md5_known_vector() {
    assert_eq!(digest(vp_hash_md5, "abc"), "900150983cd24fb0d6963f7d28e17f72");
}

#[test]
fn test_sha256_known_vector() {
    assert_eq!(
        digest(vp_hash_sha256, "abc"),
        "ba7816bf8f01cfea414140de5dae2223\
         b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn test_sha512_known_vector() {
    assert_eq!(
        digest(vp_hash_sha512, "abc"),
        "ddaf35a193617abacc417349ae204131\
         12e6fa4e89a97ea20a9eeee64b55d39a\
         2192992a274fc1a836ba3c23a3feebbd\
         454d4423643ce80e2a9ac94fa54ca49f"
    );
}
