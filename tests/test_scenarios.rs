//! Real-world Scenario Integration Tests

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

// Calculator
#[test]
fn test_scenario_calculator() {
    assert!(run_viper_code("def add(a, b):\n    return a + b\ndef sub(a, b):\n    return a - b\ndef mul(a, b):\n    return a * b\ndef div(a, b):\n    return a / b\ndef test():\n    print(add(10, 5))\n    print(sub(10, 5))\n    print(mul(10, 5))\n    print(div(10, 5))\ntest()").is_ok());
}

// Temperature Converter
#[test]
fn test_scenario_temperature_converter() {
    assert!(run_viper_code("def celsius_to_fahrenheit(c):\n    return c * 9 / 5 + 32\ndef fahrenheit_to_celsius(f):\n    return (f - 32) * 5 / 9\ndef test():\n    print(celsius_to_fahrenheit(100))\n    print(fahrenheit_to_celsius(212))\ntest()").is_ok());
}

// Factorial Table
#[test]
fn test_scenario_factorial_table() {
    assert!(run_viper_code("def factorial(n):\n    if n <= 1:\n        return 1\n    return n * factorial(n - 1)\ndef test():\n    i = 1\n    while i <= 5:\n        print(factorial(i))\n        i = i + 1\ntest()").is_ok());
}

// Multiplication Table
#[test]
fn test_scenario_multiplication_table() {
    assert!(run_viper_code("def test():\n    i = 1\n    while i <= 5:\n        j = 1\n        while j <= 5:\n            j = j + 1\n        i = i + 1\n    print(\"ok\")\ntest()").is_ok());
}
