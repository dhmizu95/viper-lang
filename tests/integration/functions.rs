//! Function Integration Tests

use std::fs;
use std::process::Command;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;
    let output = Command::new(env!("CARGO_BIN_EXE_viper")).args(["run"]).arg(&test_file).output()
        .map_err(|e| format!("Failed to run: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let _ = fs::remove_file(&test_file);
    if !output.status.success() {
        return Err(format!("stdout: {}\nstderr: {}", stdout, stderr));
    }
    Ok(stdout)
}

// Function Definitions
#[test]
fn test_function_no_params() {
    let code = r#"
def greet():
    print("hello")

def test():
    greet()
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_function_single_param() {
    let code = r#"
def double(x):
    return x * 2

def test():
    print(double(21))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_function_multiple_params() {
    let code = r#"
def add(a, b):
    return a + b

def test():
    print(add(3, 4))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_function_with_return_type() {
    let code = r#"
def add(a: int, b: int) -> int:
    return a + b

def test():
    print(add(3, 4))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Function Calls
#[test]
fn test_function_call_no_args() {
    let code = r#"
def greet():
    print("hello")

def test():
    greet()
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_function_call_with_args() {
    let code = r#"
def add(a, b):
    return a + b

def test():
    print(add(3, 5))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_function_call_nested() {
    let code = r#"
def add(a, b):
    return a + b

def mul(a, b):
    return a * b

def test():
    print(mul(add(2, 3), 4))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Lambda Expressions
#[test]
fn test_lambda_no_params() {
    let code = r#"
def test():
    f = lambda: 42
    print(f())
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_lambda_single_param() {
    let code = r#"
def test():
    f = lambda x: x + 1
    print(f(5))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_lambda_multiple_params() {
    let code = r#"
def test():
    f = lambda a, b: a + b
    print(f(3, 4))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Recursive Functions
#[test]
fn test_recursive_factorial() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test():
    print(factorial(5))
test()
"#;
    let output = run_viper_code(code).expect("factorial program should run");
    assert!(output.contains("120"), "unexpected output: {output}");
}

#[test]
fn test_recursive_fibonacci() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def test():
    print(fib(10))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_recursive_product_of_recursive_results() {
    let code = r#"
def product_tree(n):
    if n <= 1:
        return 2
    return product_tree(n - 1) * product_tree(n - 1)

def test():
    print(product_tree(3))
test()
"#;
    let output = run_viper_code(code).expect("recursive product program should run");
    assert!(output.contains("16"), "unexpected output: {output}");
}

#[test]
fn test_mutual_recursion() {
    let code = r#"
def is_even(n):
    if n == 0:
        return True
    return is_odd(n - 1)

def is_odd(n):
    if n == 0:
        return False
    return is_even(n - 1)

def test():
    print(is_even(4))
    print(is_odd(4))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Return Statements
#[test]
fn test_return_value() {
    let code = r#"
def get_answer():
    return 42

def test():
    print(get_answer())
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_return_early() {
    let code = r#"
def check_positive(x):
    if x > 0:
        return True
    return False

def test():
    print(check_positive(5))
    print(check_positive(-3))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}
