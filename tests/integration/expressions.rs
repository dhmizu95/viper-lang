//! Expression Integration Tests

use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--bin", "viper", "run"])
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

// Walrus Operator (Assignment Expression)
#[test]
fn test_walrus_operator() {
    let code = r#"
def test():
    if (n := 5) > 3:
        print(n)
test()
"#;
    let output = run_viper_code(code).expect("walrus operator should work");
    assert!(output.contains("5"), "got: {}", output);
}

#[test]
fn test_walrus_operator_in_while() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    while (x := len(lst)) > 0:
        lst.pop()
        print(x)
test()
"#;
    let output = run_viper_code(code).expect("walrus operator in while should work");
    assert!(output.contains("3"), "got: {}", output);
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("1"), "got: {}", output);
}

// Membership Operators
#[test]
fn test_in_operator() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    print(2 in lst)
    print(4 in lst)
test()
"#;
    let output = run_viper_code(code).expect("in operator should work");
    assert!(output.contains("True"), "got: {}", output);
    assert!(output.contains("False"), "got: {}", output);
}

#[test]
fn test_not_in_operator() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    print(4 not in lst)
    print(2 not in lst)
test()
"#;
    let output = run_viper_code(code).expect("not in operator should work");
    assert!(output.contains("True"), "got: {}", output);
    assert!(output.contains("False"), "got: {}", output);
}
