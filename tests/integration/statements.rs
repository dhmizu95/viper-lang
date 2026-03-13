//! Statement Integration Tests

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

// Assignment
#[test]
fn test_assign_simple() {
    let code = r#"
def test():
    x = 42
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Declare with type
#[test]
fn test_declare_with_type() {
    let code = r#"
def test():
    x: int = 42
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Const
#[test]
fn test_const() {
    let code = r#"
const PI = 3.14159

def test():
    print(PI)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Assert
#[test]
fn test_assert_simple() {
    let code = r#"
def test():
    x = 5
    assert x > 0
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_assert_with_message() {
    let code = r#"
def test():
    x = 5
    assert x > 0, "x must be positive"
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Delimiters
#[test]
fn test_delimiters_parentheses() {
    let code = r#"
def test():
    a = (1 + 2) * 3
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_delimiters_colon() {
    let code = r#"
def test():
    if True:
        print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Keywords
#[test]
fn test_keyword_def() {
    let code = r#"
def my_func():
    print("ok")

def test():
    my_func()
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_while_kw() {
    let code = r#"
def test():
    i = 0
    while i < 3:
        i = i + 1
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_return() {
    let code = r#"
def get_value():
    return 42

def test():
    print(get_value())
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_if_elif_else() {
    let code = r#"
def test():
    x = 10
    if x > 20:
        print("a")
    elif x > 5:
        print("b")
    else:
        print("c")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// Multiple Assignment - Issue #5
// Basic tuple unpacking is now supported including:
// - Literal tuples: a, b = 1, 2
// - Expression tuples: a, b = x + 1, y + 2
// - Function returns: x, y = get_pair()
// ============================================================================

#[test]
fn test_multiple_assignment_basic() {
    let code = r#"
def test():
    a, b = 1, 2
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_with_expressions() {
    let code = r#"
def test():
    x = 10
    y = 20
    a, b = x + 1, y + 2
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_swap() {
    let code = r#"
def test():
    a = 1
    b = 2
    a, b = b, a
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_three_vars() {
    let code = r#"
def test():
    a, b, c = 1, 2, 3
    print(a)
    print(b)
    print(c)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_with_function_return() {
    let code = r#"
def get_pair():
    return 10, 20

def test():
    x, y = get_pair()
    print(x)
    print(y)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}
