//! Control Flow Integration Tests

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

// If Statements
#[test]
fn test_if_simple() {
    assert!(run_viper_code("def test():\n    x = 10\n    if x > 5:\n        print(\"yes\")\ntest()").is_ok());
}

#[test]
fn test_if_else() {
    assert!(run_viper_code("def test():\n    x = 3\n    if x > 5:\n        print(\"greater\")\n    else:\n        print(\"less\")\ntest()").is_ok());
}

#[test]
fn test_if_elif_else() {
    assert!(run_viper_code("def test():\n    x = 5\n    if x > 10:\n        print(\"a\")\n    elif x > 3:\n        print(\"b\")\n    else:\n        print(\"c\")\ntest()").is_ok());
}

// While Loops
#[test]
fn test_while_simple() {
    assert!(run_viper_code("def test():\n    i = 0\n    while i < 5:\n        i = i + 1\n    print(i)\ntest()").is_ok());
}

#[test]
fn test_while_with_break() {
    assert!(run_viper_code("def test():\n    i = 0\n    while True:\n        if i >= 5:\n            break\n        i = i + 1\n    print(i)\ntest()").is_ok());
}

#[test]
fn test_while_with_continue() {
    assert!(run_viper_code("def test():\n    i = 0\n    total = 0\n    while i < 5:\n        i = i + 1\n        if i == 3:\n            continue\n        total = total + i\n    print(total)\ntest()").is_ok());
}

#[test]
fn test_while_nested() {
    assert!(run_viper_code("def test():\n    i = 0\n    j = 0\n    while i < 3:\n        while j < 3:\n            j = j + 1\n        i = i + 1\n        j = 0\n    print(i)\ntest()").is_ok());
}

// Keywords
#[test]
fn test_keyword_break() {
    assert!(run_viper_code("def test():\n    i = 0\n    while i < 10:\n        if i == 3:\n            break\n        i = i + 1\n    print(i)\ntest()").is_ok());
}

#[test]
fn test_keyword_continue() {
    assert!(run_viper_code("def test():\n    i = 0\n    while i < 5:\n        i = i + 1\n        if i == 3:\n            continue\n    print(\"ok\")\ntest()").is_ok());
}

#[test]
fn test_keyword_pass() {
    assert!(run_viper_code("def test():\n    if True:\n        pass\n    print(\"ok\")\ntest()").is_ok());
}
