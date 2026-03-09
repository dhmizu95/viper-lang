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

#[test]
fn test_int_literals_basic() {
    assert!(run_viper_code("def test():\n    a = 42\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_int_literals_hex() {
    assert!(run_viper_code("def test():\n    a = 0xFF\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_int_literals_binary() {
    assert!(run_viper_code("def test():\n    a = 0b1010\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_int_literals_octal() {
    assert!(run_viper_code("def test():\n    a = 0o755\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_float_literals_basic() {
    assert!(run_viper_code("def test():\n    a = 3.14\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_string_literals_double_quote() {
    assert!(run_viper_code("def test():\n    a = \"hello\"\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_string_literals_single_quote() {
    assert!(run_viper_code("def test():\n    a = 'world'\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_string_literals_escape() {
    assert!(run_viper_code("def test():\n    a = \"hello\\nworld\"\n    print(\"ok\")\ntest()").is_ok());
}

#[test]
fn test_bool_literals_true() {
    assert!(run_viper_code("def test():\n    a = True\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_bool_literals_false() {
    assert!(run_viper_code("def test():\n    a = False\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_none_literal() {
    assert!(run_viper_code("def test():\n    a = None\n    b = None\n    print(a is b)\ntest()").is_ok());
}
