//! Integration tests for Viper Language
//! Tests covering end-to-end scenarios from the test plan

use std::fs;
use std::process::Command;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

/// Helper to run viper compiler
fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    
    fs::write(&test_file, code)
        .map_err(|e| format!("Failed to write test file: {}", e))?;
    
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--bin", "viper", "run"])
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run viper: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Clean up
    let _ = fs::remove_file(&test_file);

    if !output.status.success() {
        return Err(format!("Command failed:\nstdout: {}\nstderr: {}", stdout, stderr));
    }

    Ok(stdout)
}

// ============================================================================
// 1. Lexer Integration Tests (test_plan.md Section 1)
// ============================================================================

#[test]
fn test_int_literals_integration() {
    let code = r#"
def test():
    a = 42
    print(a)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Integer literals test failed: {:?}", result.err());
}

#[test]
fn test_float_literals_integration() {
    let code = r#"
def test():
    a = 3.14
    print(a)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Float literals test failed: {:?}", result.err());
}

#[test]
fn test_string_literals_integration() {
    let code = r#"
def test():
    a = "hello"
    print(a)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "String literals test failed: {:?}", result.err());
}

#[test]
fn test_bool_literals_integration() {
    let code = r#"
def test():
    a = True
    b = False
    print(a)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Bool literals test failed: {:?}", result.err());
}

// ============================================================================
// 2. Parser Integration Tests (test_plan.md Section 2)
// ============================================================================

#[test]
fn test_binary_ops_integration() {
    let code = r#"
def test():
    a = 1 + 2 * 3
    print(a)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Binary ops test failed: {:?}", result.err());
}

#[test]
fn test_function_calls_integration() {
    let code = r#"
def add(a, b):
    return a + b

def test():
    result = add(3, 5)
    print(result)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Function calls test failed: {:?}", result.err());
}

// ============================================================================
// 3. Control Flow Integration Tests (test_plan.md Section 2.2)
// ============================================================================

#[test]
fn test_if_statements_integration() {
    let code = r#"
def test():
    x = 10
    if x > 5:
        print("greater")
    else:
        print("less")
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "If statements test failed: {:?}", result.err());
}

#[test]
fn test_while_loops_integration() {
    let code = r#"
def test():
    i = 0
    while i < 5:
        i = i + 1
    print(i)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "While loops test failed: {:?}", result.err());
}

// ============================================================================
// 4. Function Definition Integration Tests (test_plan.md Section 2.3)
// ============================================================================

#[test]
fn test_function_def_integration() {
    let code = r#"
def greet(n):
    print(n)

def test():
    greet(42)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Function definition test failed: {:?}", result.err());
}

#[test]
fn test_recursive_function_integration() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test():
    result = factorial(5)
    print(result)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Recursive function test failed: {:?}", result.err());
}

// ============================================================================
// 5. Expression Integration Tests (test_plan.md Section 2.1)
// ============================================================================

#[test]
fn test_lambda_integration() {
    let code = r#"
def test():
    f = lambda x: x + 1
    result = f(5)
    print(result)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Lambda test failed: {:?}", result.err());
}

// ============================================================================
// 6. Operator Integration Tests (test_plan.md Section 1.2)
// ============================================================================

#[test]
fn test_comparison_ops_integration() {
    let code = r#"
def test():
    a = 5
    b = 10
    print(a < b)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Comparison ops test failed: {:?}", result.err());
}

#[test]
fn test_logical_ops_integration() {
    let code = r#"
def test():
    a = True
    b = False
    print(a and b)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Logical ops test failed: {:?}", result.err());
}

#[test]
fn test_augmented_assign_integration() {
    let code = r#"
def test():
    x = 10
    x += 5
    print(x)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Augmented assignment test failed: {:?}", result.err());
}

#[test]
fn test_identity_ops_integration() {
    let code = r#"
def test():
    a = None
    b = None
    print(a is b)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Identity ops test failed: {:?}", result.err());
}

// ============================================================================
// 7. Error Handling Integration Tests (test_plan.md Section 2.2)
// ============================================================================

#[test]
fn test_assert_integration() {
    let code = r#"
def test():
    x = 5
    assert x > 0
    print("ok")
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Assert test failed: {:?}", result.err());
}

// ============================================================================
// 8. Algorithm Integration Tests (test_plan.md Section 6.1)
// ============================================================================

#[test]
fn test_fibonacci_recursive() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def test():
    result = fib(10)
    print(result)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Fibonacci test failed: {:?}", result.err());
}

#[test]
fn test_factorial_iterative() {
    let code = r#"
def factorial(n):
    result = 1
    i = 1
    while i <= n:
        result = result * i
        i = i + 1
    return result

def test():
    result = factorial(5)
    print(result)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Factorial iterative test failed: {:?}", result.err());
}

#[test]
fn test_gcd_euclidean() {
    let code = r#"
def gcd(a, b):
    while b != 0:
        temp = b
        b = a % b
        a = temp
    return a

def test():
    result = gcd(48, 18)
    print(result)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "GCD Euclidean test failed: {:?}", result.err());
}

#[test]
fn test_power_iterative() {
    let code = r#"
def power(base, exp):
    result = 1
    i = 0
    while i < exp:
        result = result * base
        i = i + 1
    return result

def test():
    result = power(2, 10)
    print(result)
test()
"#;
    let result = run_viper_code(code);
    assert!(result.is_ok(), "Power iterative test failed: {:?}", result.err());
}

// ============================================================================
// Known Issues / Not Yet Implemented Tests
// ============================================================================

// The following features are in the test plan but not yet fully working:
// - List literals (JIT segfault)
// - Dict literals (JIT segfault)  
// - Tuple literals (JIT segfault)
// - For loops (JIT segfault)
// - Class definitions
// - Try/except blocks
// - Async/await
// - Channels and concurrency
// - String concatenation with +
// - Ternary/conditional expressions (codegen issues)
// - Method calls on built-in types

// These tests should be added once the features are fully implemented
