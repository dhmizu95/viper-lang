//! Operator Integration Tests

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

// Arithmetic Operators
#[test]
fn test_arithmetic_add() { assert!(run_viper_code("def test():\n    a = 5 + 3\n    print(a)\ntest()").is_ok()); }

#[test]
fn test_arithmetic_sub() { assert!(run_viper_code("def test():\n    a = 10 - 4\n    print(a)\ntest()").is_ok()); }

#[test]
fn test_arithmetic_mul() { assert!(run_viper_code("def test():\n    a = 6 * 7\n    print(a)\ntest()").is_ok()); }

#[test]
fn test_arithmetic_div() { assert!(run_viper_code("def test():\n    a = 20 / 4\n    print(a)\ntest()").is_ok()); }

#[test]
fn test_arithmetic_mod() { assert!(run_viper_code("def test():\n    a = 17 % 5\n    print(a)\ntest()").is_ok()); }

#[test]
fn test_arithmetic_pow() { assert!(run_viper_code("def test():\n    a = 2 ** 8\n    print(a)\ntest()").is_ok()); }

#[test]
fn test_arithmetic_floor_div() { assert!(run_viper_code("def test():\n    a = 17 // 5\n    print(a)\ntest()").is_ok()); }

// Comparison Operators
#[test]
fn test_comparison_eq() { assert!(run_viper_code("def test():\n    print(5 == 5)\ntest()").is_ok()); }

#[test]
fn test_comparison_not_eq() { assert!(run_viper_code("def test():\n    print(5 != 10)\ntest()").is_ok()); }

#[test]
fn test_comparison_lt() { assert!(run_viper_code("def test():\n    print(5 < 10)\ntest()").is_ok()); }

#[test]
fn test_comparison_gt() { assert!(run_viper_code("def test():\n    print(10 > 5)\ntest()").is_ok()); }

#[test]
fn test_comparison_lt_eq() { assert!(run_viper_code("def test():\n    print(5 <= 5)\ntest()").is_ok()); }

#[test]
fn test_comparison_gt_eq() { assert!(run_viper_code("def test():\n    print(10 >= 5)\ntest()").is_ok()); }

// Logical Operators
#[test]
fn test_logical_and() { assert!(run_viper_code("def test():\n    print(True and False)\ntest()").is_ok()); }

#[test]
fn test_logical_or() { assert!(run_viper_code("def test():\n    print(True or False)\ntest()").is_ok()); }

#[test]
fn test_logical_not() { assert!(run_viper_code("def test():\n    print(not True)\ntest()").is_ok()); }

// Identity Operators
#[test]
fn test_identity_is() { assert!(run_viper_code("def test():\n    a = None\n    b = None\n    print(a is b)\ntest()").is_ok()); }

// Augmented Assignment
#[test]
fn test_augmented_add() { assert!(run_viper_code("def test():\n    x = 10\n    x += 5\n    print(x)\ntest()").is_ok()); }

#[test]
fn test_augmented_sub() { assert!(run_viper_code("def test():\n    x = 10\n    x -= 3\n    print(x)\ntest()").is_ok()); }

#[test]
fn test_augmented_mul() { assert!(run_viper_code("def test():\n    x = 5\n    x *= 3\n    print(x)\ntest()").is_ok()); }

#[test]
fn test_augmented_mod() { assert!(run_viper_code("def test():\n    x = 17\n    x %= 5\n    print(x)\ntest()").is_ok()); }

#[test]
fn test_augmented_pow() { assert!(run_viper_code("def test():\n    x = 2\n    x **= 3\n    print(x)\ntest()").is_ok()); }

// Bitwise Operators
#[test]
fn test_bitwise_and() {
    assert!(run_viper_code("def test():\n    a = 12 & 10\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_bitwise_or() {
    assert!(run_viper_code("def test():\n    a = 12 | 10\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_bitwise_xor() {
    assert!(run_viper_code("def test():\n    a = 12 ^ 10\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_bitwise_lshift() {
    assert!(run_viper_code("def test():\n    a = 4 << 2\n    print(a)\ntest()").is_ok());
}

#[test]
fn test_bitwise_rshift() {
    assert!(run_viper_code("def test():\n    a = 16 >> 2\n    print(a)\ntest()").is_ok());
}

// Bitwise Augmented Assignment
#[test]
fn test_augmented_bitwise_and() {
    assert!(run_viper_code("def test():\n    x = 15\n    x &= 7\n    print(x)\ntest()").is_ok());
}

#[test]
fn test_augmented_bitwise_or() {
    assert!(run_viper_code("def test():\n    x = 8\n    x |= 7\n    print(x)\ntest()").is_ok());
}

#[test]
fn test_augmented_bitwise_xor() {
    assert!(run_viper_code("def test():\n    x = 15\n    x ^= 7\n    print(x)\ntest()").is_ok());
}

#[test]
fn test_augmented_lshift() {
    assert!(run_viper_code("def test():\n    x = 4\n    x <<= 2\n    print(x)\ntest()").is_ok());
}

#[test]
fn test_augmented_rshift() {
    assert!(run_viper_code("def test():\n    x = 16\n    x >>= 2\n    print(x)\ntest()").is_ok());
}
