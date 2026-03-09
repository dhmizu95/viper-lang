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
    assert!(run_viper_code("def test():\n    a = 10 + 5 * 2\n    b = (10 + 5) * 2\n    c = 100 / 4\n    d = 17 % 5\n    print(a)\n    print(b)\n    print(c)\n    print(d)\ntest()").is_ok());
}

#[test]
fn test_codegen_arithmetic_float() {
    assert!(run_viper_code("def test():\n    a = 3.14 + 2.86\n    b = 10.0 / 4.0\n    print(a)\n    print(b)\ntest()").is_ok());
}

// Comparison Codegen
#[test]
fn test_codegen_comparison() {
    assert!(run_viper_code("def test():\n    print(5 < 10)\n    print(5 > 10)\n    print(5 == 5)\n    print(5 != 10)\n    print(5 <= 5)\n    print(5 >= 5)\ntest()").is_ok());
}

// Logical Codegen
#[test]
fn test_codegen_logical() {
    assert!(run_viper_code("def test():\n    print(True and False)\n    print(True or False)\n    print(not True)\n    print(not False)\ntest()").is_ok());
}

// Branch Codegen
#[test]
fn test_codegen_branches_if() {
    assert!(run_viper_code("def test():\n    x = 10\n    if x > 5:\n        print(\"branch1\")\ntest()").is_ok());
}

#[test]
fn test_codegen_branches_if_else() {
    assert!(run_viper_code("def test():\n    x = 3\n    if x > 5:\n        print(\"greater\")\n    else:\n        print(\"less\")\ntest()").is_ok());
}

// Loop Codegen
#[test]
fn test_codegen_loops_while() {
    assert!(run_viper_code("def test():\n    i = 0\n    while i < 5:\n        i = i + 1\n    print(i)\ntest()").is_ok());
}

#[test]
fn test_codegen_loops_nested() {
    assert!(run_viper_code("def test():\n    i = 0\n    j = 0\n    while i < 3:\n        while j < 3:\n            j = j + 1\n        i = i + 1\n        j = 0\n    print(i)\ntest()").is_ok());
}

// Function Codegen
#[test]
fn test_codegen_functions_simple() {
    assert!(run_viper_code("def add(a, b):\n    return a + b\ndef test():\n    print(add(10, 20))\ntest()").is_ok());
}

#[test]
fn test_codegen_functions_recursive() {
    assert!(run_viper_code("def factorial(n):\n    if n <= 1:\n        return 1\n    return n * factorial(n - 1)\ndef test():\n    print(factorial(5))\ntest()").is_ok());
}

// Closure Codegen
#[test]
fn test_codegen_closures() {
    assert!(run_viper_code("def test():\n    f = lambda x: x * 2\n    print(f(21))\ntest()").is_ok());
}
