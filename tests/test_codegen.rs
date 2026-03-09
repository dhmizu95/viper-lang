//! Code Generation Integration Tests

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

// Arithmetic Codegen
#[test]
fn test_codegen_arithmetic_int() {
    let code = r#"
def test():
    a = 10 + 5 * 2
    b = (10 + 5) * 2
    c = 100 / 4
    d = 17 % 5
    print(a)
    print(b)
    print(c)
    print(d)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_codegen_arithmetic_float() {
    let code = r#"
def test():
    a = 3.14 + 2.86
    b = 10.0 / 4.0
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Comparison Codegen
#[test]
fn test_codegen_comparison() {
    let code = r#"
def test():
    print(5 < 10)
    print(5 > 10)
    print(5 == 5)
    print(5 != 10)
    print(5 <= 5)
    print(5 >= 5)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Logical Codegen
#[test]
fn test_codegen_logical() {
    let code = r#"
def test():
    print(True and False)
    print(True or False)
    print(not True)
    print(not False)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Branch Codegen
#[test]
fn test_codegen_branches_if() {
    let code = r#"
def test():
    x = 10
    if x > 5:
        print("branch1")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_codegen_branches_if_else() {
    let code = r#"
def test():
    x = 3
    if x > 5:
        print("greater")
    else:
        print("less")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Loop Codegen
#[test]
fn test_codegen_loops_while() {
    let code = r#"
def test():
    i = 0
    while i < 5:
        i = i + 1
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_codegen_loops_nested() {
    let code = r#"
def test():
    i = 0
    j = 0
    while i < 3:
        while j < 3:
            j = j + 1
        i = i + 1
        j = 0
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Function Codegen
#[test]
fn test_codegen_functions_simple() {
    let code = r#"
def add(a, b):
    return a + b

def test():
    print(add(10, 20))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_codegen_functions_recursive() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test():
    print(factorial(5))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Closure Codegen
#[test]
fn test_codegen_closures() {
    let code = r#"
def test():
    f = lambda x: x * 2
    print(f(21))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}
