//! Statement Integration Tests

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

// Assignment
#[test]
fn test_assign_simple() {
    assert!(run_viper_code("def test():\n    x = 42\n    print(x)\ntest()").is_ok());
}

// Declare with type
#[test]
fn test_declare_with_type() {
    assert!(run_viper_code("def test():\n    x: int = 42\n    print(x)\ntest()").is_ok());
}

// Const
#[test]
fn test_const() {
    assert!(run_viper_code("const PI = 3.14159\ndef test():\n    print(PI)\ntest()").is_ok());
}

// Assert
#[test]
fn test_assert_simple() {
    assert!(run_viper_code("def test():\n    x = 5\n    assert x > 0\n    print(\"ok\")\ntest()").is_ok());
}

#[test]
fn test_assert_with_message() {
    assert!(run_viper_code("def test():\n    x = 5\n    assert x > 0, \"x must be positive\"\n    print(\"ok\")\ntest()").is_ok());
}

// Delimiters
#[test]
fn test_delimiters_parentheses() {
    assert!(run_viper_code("def test():\n    a = (1 + 2) * 3\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_delimiters_colon() {
    assert!(run_viper_code("def test():\n    if True:\n        print(\"ok\")\ntest()").is_ok());
}

// Keywords
#[test]
fn test_keyword_def() {
    assert!(run_viper_code("def my_func():\n    print(\"ok\")\ndef test():\n    my_func()\ntest()").is_ok());
}

#[test]
fn test_keyword_while_kw() {
    assert!(run_viper_code("def test():\n    i = 0\n    while i < 3:\n        i = i + 1\n    print(i)\ntest()").is_ok());
}

#[test]
fn test_keyword_return() {
    assert!(run_viper_code("def get_value():\n    return 42\ndef test():\n    print(get_value())\ntest()").is_ok());
}

#[test]
fn test_keyword_if_elif_else() {
    assert!(run_viper_code("def test():\n    x = 10\n    if x > 20:\n        print(\"a\")\n    elif x > 5:\n        print(\"b\")\n    else:\n        print(\"c\")\ntest()").is_ok());
}
