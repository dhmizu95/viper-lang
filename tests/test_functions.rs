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

// Function Definitions
#[test]
fn test_function_no_params() {
    assert!(run_viper_code("def greet():\n    print(\"hello\")\ndef test():\n    greet()\ntest()").is_ok());
}

#[test]
fn test_function_single_param() {
    assert!(run_viper_code("def double(x):\n    return x * 2\ndef test():\n    print(double(21))\ntest()").is_ok());
}

#[test]
fn test_function_multiple_params() {
    assert!(run_viper_code("def add(a, b):\n    return a + b\ndef test():\n    print(add(3, 4))\ntest()").is_ok());
}

#[test]
fn test_function_with_return_type() {
    assert!(run_viper_code("def add(a: int, b: int) -> int:\n    return a + b\ndef test():\n    print(add(3, 4))\ntest()").is_ok());
}

// Function Calls
#[test]
fn test_function_call_no_args() {
    assert!(run_viper_code("def greet():\n    print(\"hello\")\ndef test():\n    greet()\ntest()").is_ok());
}

#[test]
fn test_function_call_with_args() {
    assert!(run_viper_code("def add(a, b):\n    return a + b\ndef test():\n    print(add(3, 5))\ntest()").is_ok());
}

#[test]
fn test_function_call_nested() {
    assert!(run_viper_code("def add(a, b):\n    return a + b\ndef mul(a, b):\n    return a * b\ndef test():\n    print(mul(add(2, 3), 4))\ntest()").is_ok());
}

// Lambda Expressions
#[test]
fn test_lambda_no_params() {
    assert!(run_viper_code("def test():\n    f = lambda: 42\n    print(f())\ntest()").is_ok());
}

#[test]
fn test_lambda_single_param() {
    assert!(run_viper_code("def test():\n    f = lambda x: x + 1\n    print(f(5))\ntest()").is_ok());
}

#[test]
fn test_lambda_multiple_params() {
    assert!(run_viper_code("def test():\n    f = lambda a, b: a + b\n    print(f(3, 4))\ntest()").is_ok());
}

// Recursive Functions
#[test]
fn test_recursive_factorial() {
    assert!(run_viper_code("def factorial(n):\n    if n <= 1:\n        return 1\n    return n * factorial(n - 1)\ndef test():\n    print(factorial(5))\ntest()").is_ok());
}

#[test]
fn test_recursive_fibonacci() {
    assert!(run_viper_code("def fib(n):\n    if n <= 1:\n        return n\n    return fib(n - 1) + fib(n - 2)\ndef test():\n    print(fib(10))\ntest()").is_ok());
}

#[test]
fn test_mutual_recursion() {
    assert!(run_viper_code("def is_even(n):\n    if n == 0:\n        return True\n    return is_odd(n - 1)\ndef is_odd(n):\n    if n == 0:\n        return False\n    return is_even(n - 1)\ndef test():\n    print(is_even(4))\n    print(is_odd(4))\ntest()").is_ok());
}

// Return Statements
#[test]
fn test_return_value() {
    assert!(run_viper_code("def get_answer():\n    return 42\ndef test():\n    print(get_answer())\ntest()").is_ok());
}

#[test]
fn test_return_early() {
    assert!(run_viper_code("def check_positive(x):\n    if x > 0:\n        return True\n    return False\ndef test():\n    print(check_positive(5))\n    print(check_positive(-3))\ntest()").is_ok());
}
