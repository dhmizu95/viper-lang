//! Literal Integration Tests

use std::fs;
use std::process::Command;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;
    let output = Command::new("cargo").args(["run", "--quiet", "--bin", "viper", "run"]).arg(&test_file).output()
        .map_err(|e| format!("Failed to run: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let _ = fs::remove_file(&test_file);
    if !output.status.success() {
        return Err(format!("stdout: {}\nstderr: {}", stdout, stderr));
    }
    Ok(stdout)
}

// Integer Literals
#[test]
fn test_int_literals_basic() {
    let code = r#"
def test():
    a = 42
    b = -17
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_int_literals_hex() {
    let code = r#"
def test():
    a = 0xFF
    b = 0x1A
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_int_literals_binary() {
    let code = r#"
def test():
    a = 0b1010
    b = 0b1111
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_int_literals_octal() {
    let code = r#"
def test():
    a = 0o755
    b = 0o644
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Float Literals
#[test]
fn test_float_literals_basic() {
    let code = r#"
def test():
    a = 3.14
    b = -2.5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_float_literals_exponent() {
    let code = r#"
def test():
    a = 2.5e10
    b = 1.0e-5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// String Literals
#[test]
fn test_string_literals_double_quote() {
    let code = r#"
def test():
    a = "hello"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_literals_single_quote() {
    let code = r#"
def test():
    a = 'world'
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_literals_escape() {
    let code = r#"
def test():
    a = "hello\nworld"
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Boolean Literals
#[test]
fn test_bool_literals_true() {
    let code = r#"
def test():
    a = True
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bool_literals_false() {
    let code = r#"
def test():
    a = False
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// None Literal
#[test]
fn test_none_literal() {
    let code = r#"
def test():
    a = None
    b = None
    print(a is b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// BigInt Literals
#[test]
fn test_bigint_literals() {
    let code = r#"
def test():
    a = 123456789012345678901234567890
    b = 999999999999999999999999999999
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// KNOWN LIMITATIONS - Tests for features not yet fully implemented
// ============================================================================

// The following literal types have known JIT issues:

// 1. f-string literals - String concatenation causes segfault
// Example: f"Hello {name}"
// Root cause: vp_str_concat runtime function has issues
// Fix needed: Debug and fix string concatenation in runtime

// 2. bytes literals - Causes segfault
// Example: b"bytes"
// Root cause: bytes_const or vp_bytes_create has issues
// Fix needed: Debug bytes handling in codegen/runtime

// These tests should be added once the features are fully implemented:

// #[test]
// fn test_fstring_literals() {
//     let code = r#"
// def test():
//     name = "World"
//     a = f"Hello {name}"
//     print(a)
// test()
// "#;
//     assert!(run_viper_code(code).is_ok());
// }

// #[test]
// fn test_bytes_literals() {
//     let code = r#"
// def test():
//     a = b"hello"
//     print(a)
// test()
// "#;
//     assert!(run_viper_code(code).is_ok());
// }
