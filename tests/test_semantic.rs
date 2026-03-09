//! Semantic Analysis Integration Tests

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

// Type Inference
#[test]
fn test_type_inference_int() {
    assert!(run_viper_code("def test():\n    x = 42\n    y = x + 1\n    print(y)\ntest()").is_ok());
}

// Scope Tests
#[test]
fn test_scope_local() {
    assert!(run_viper_code("def test():\n    x = 10\n    if True:\n        y = 20\n    print(x)\n    print(y)\ntest()").is_ok());
}

#[test]
fn test_shadowing() {
    assert!(run_viper_code("def test():\n    x = 10\n    if True:\n        x = 20\n    print(x)\ntest()").is_ok());
}

#[test]
fn test_function_scope() {
    assert!(run_viper_code("def inner():\n    x = 42\n    return x\ndef test():\n    print(inner())\ntest()").is_ok());
}
