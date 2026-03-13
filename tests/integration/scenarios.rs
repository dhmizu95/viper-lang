//! Real-world Scenario Integration Tests

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

// Calculator
#[test]
fn test_scenario_calculator() {
    let code = r#"
def add(a, b):
    return a + b

def sub(a, b):
    return a - b

def mul(a, b):
    return a * b

def div(a, b):
    return a / b

def test():
    print(add(10, 5))
    print(sub(10, 5))
    print(mul(10, 5))
    print(div(10, 5))
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("15"));
    assert!(output.contains("5"));
}

// Temperature Converter
#[test]
fn test_scenario_temperature_converter() {
    let code = r#"
def celsius_to_fahrenheit(c):
    return c * 9 / 5 + 32

def fahrenheit_to_celsius(f):
    return (f - 32) * 5 / 9

def test():
    print(celsius_to_fahrenheit(100))
    print(fahrenheit_to_celsius(212))
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("212"));
    assert!(output.contains("100"));
}

// Factorial Table
#[test]
fn test_scenario_factorial_table() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test():
    i = 1
    while i <= 5:
        print(factorial(i))
        i = i + 1
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Multiplication Table
#[test]
fn test_scenario_multiplication_table() {
    let code = r#"
def test():
    i = 1
    while i <= 5:
        j = 1
        while j <= 5:
            j = j + 1
        i = i + 1
    print("ok")
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("ok"));
}
