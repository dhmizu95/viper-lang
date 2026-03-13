//! Operator Integration Tests

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

// Arithmetic Operators
#[test]
fn test_arithmetic_add() {
    let code = r#"
def test():
    a = 5 + 3
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_arithmetic_add_overflow_promotes_to_bigint() {
    let code = r#"
def test():
    print(4611686018427387903 + 1)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("4611686018427387904"), "stdout was: {}", stdout);
}

#[test]
fn test_arithmetic_sub() {
    let code = r#"
def test():
    a = 10 - 4
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_arithmetic_sub_negative_small_int_fast_path() {
    let code = r#"
def test():
    print(-5 - 2)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("-7"), "stdout was: {}", stdout);
}

#[test]
fn test_arithmetic_mul() {
    let code = r#"
def test():
    a = 6 * 7
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_arithmetic_mul_overflow_promotes_to_bigint() {
    let code = r#"
def test():
    print(3037000500 * 3037000500)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("9223372037000250000"), "stdout was: {}", stdout);
}

#[test]
fn test_arithmetic_div() {
    let code = r#"
def test():
    a = 20 / 4
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_arithmetic_mod() {
    let code = r#"
def test():
    a = 17 % 5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_arithmetic_pow() {
    let code = r#"
def test():
    a = 2 ** 8
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_arithmetic_floor_div() {
    let code = r#"
def test():
    a = 17 // 5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Comparison Operators
#[test]
fn test_comparison_eq() {
    let code = r#"
def test():
    print(5 == 5)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_comparison_not_eq() {
    let code = r#"
def test():
    print(5 != 10)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_comparison_lt() {
    let code = r#"
def test():
    print(5 < 10)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_comparison_gt() {
    let code = r#"
def test():
    print(10 > 5)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_comparison_lt_eq() {
    let code = r#"
def test():
    print(5 <= 5)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_comparison_gt_eq() {
    let code = r#"
def test():
    print(10 >= 5)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_comparison_mixed_bigint_and_small_int() {
    let code = r#"
def test():
    big = 4611686018427387904
    print(big > 3)
    print(3 < big)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.matches("True").count() >= 2, "stdout was: {}", stdout);
}

// Logical Operators
#[test]
fn test_logical_and() {
    let code = r#"
def test():
    print(True and False)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_logical_or() {
    let code = r#"
def test():
    print(True or False)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_logical_not() {
    let code = r#"
def test():
    print(not True)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Identity Operators
#[test]
fn test_identity_is() {
    let code = r#"
def test():
    a = None
    b = None
    print(a is b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Augmented Assignment
#[test]
fn test_augmented_add() {
    let code = r#"
def test():
    x = 10
    x += 5
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_sub() {
    let code = r#"
def test():
    x = 10
    x -= 3
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_mul() {
    let code = r#"
def test():
    x = 5
    x *= 3
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_mod() {
    let code = r#"
def test():
    x = 17
    x %= 5
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_pow() {
    let code = r#"
def test():
    x = 2
    x **= 3
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Bitwise Operators
#[test]
fn test_bitwise_and() {
    let code = r#"
def test():
    a = 12 & 10
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bitwise_or_mixed_bigint_and_small_int() {
    let code = r#"
def test():
    big = 4611686018427387904
    print(big | 1)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("4611686018427387905"), "stdout was: {}", stdout);
}

#[test]
fn test_bitwise_or() {
    let code = r#"
def test():
    a = 12 | 10
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bitwise_xor() {
    let code = r#"
def test():
    a = 12 ^ 10
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bitwise_lshift() {
    let code = r#"
def test():
    a = 4 << 2
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bitwise_lshift_overflow_promotes_to_bigint() {
    let code = r#"
def test():
    print(1 << 62)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("4611686018427387904"), "stdout was: {}", stdout);
}

#[test]
fn test_bitwise_rshift() {
    let code = r#"
def test():
    a = 16 >> 2
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bitwise_rshift_negative_small_int_fast_path() {
    let code = r#"
def test():
    print(-16 >> 2)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("-4"), "stdout was: {}", stdout);
}

// Bitwise Augmented Assignment
#[test]
fn test_augmented_bitwise_and() {
    let code = r#"
def test():
    x = 15
    x &= 7
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_bitwise_or() {
    let code = r#"
def test():
    x = 8
    x |= 7
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_bitwise_xor() {
    let code = r#"
def test():
    x = 15
    x ^= 7
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_lshift() {
    let code = r#"
def test():
    x = 4
    x <<= 2
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_augmented_rshift() {
    let code = r#"
def test():
    x = 16
    x >>= 2
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Identity Is Not Operator
#[test]
fn test_identity_is_not() {
    let code = r#"
def test():
    a = None
    b = 5
    print(a is not b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Unary Invert Operator
#[test]
fn test_unary_invert() {
    let code = r#"
def test():
    a = ~5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_unary_neg_bigint_boundary() {
    let code = r#"
def test():
    big = 4611686018427387904
    print(-big)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("-4611686018427387904"), "stdout was: {}", stdout);
}

#[test]
fn test_mod_mixed_bigint_and_small_int() {
    let code = r#"
def test():
    big = 4611686018427387904
    print(big % 3)
test()
"#;
    let stdout = run_viper_code(code).expect("program should run");
    assert!(stdout.contains("\n1\n") || stdout.contains("\r\n1\r\n"), "stdout was: {}", stdout);
}
