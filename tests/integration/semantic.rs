//! Semantic Analysis Integration Tests

use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
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
    let code = r#"
def test():
    x = 42
    y = x + 1
    print(y)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("43"));
}

// Scope Tests
#[test]
fn test_scope_local() {
    let code = r#"
def test():
    x = 10
    if True:
        y = 20
    print(x)
    print(y)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("10"));
    assert!(output.contains("20"));
}

#[test]
fn test_shadowing() {
    let code = r#"
def test():
    x = 10
    if True:
        x = 20
    print(x)
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("20"));
}

#[test]
fn test_function_scope() {
    let code = r#"
def inner():
    x = 42
    return x

def test():
    print(inner())
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("42"));
}
