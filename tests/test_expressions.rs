//! Expression Integration Tests

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

// Binary Operations
#[test]
fn test_binary_ops_precedence() {
    let code = r#"
def test():
    a = 1 + 2 * 3
    b = (1 + 2) * 3
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Unary Operations
#[test]
fn test_unary_neg() {
    let code = r#"
def test():
    a = -5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_unary_pos() {
    let code = r#"
def test():
    a = +5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_unary_not() {
    let code = r#"
def test():
    a = not True
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Ternary Expression
#[test]
fn test_ternary() {
    let code = r#"
def test():
    x = 10
    result = 1 if x > 5 else 2
    print(result)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}
