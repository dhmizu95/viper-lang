//! Operator Integration Tests

use crate::utils::run_viper_code;

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

// Null Coalescing Operator
#[test]
fn test_null_coalesce() {
    let code = r#"
def test():
    print(None ?? 42)
    print(10 ?? 42)
test()
"#;
    let output = run_viper_code(code).expect("null coalesce should work");
    assert!(output.contains("42"), "got: {}", output);
    assert!(output.contains("10"), "got: {}", output);
}

// Increment / Decrement Operators
#[test]
fn test_pre_increment() {
    let code = r#"
def test():
    x = 10
    print(++x)
    print(x)
test()
"#;
    let output = run_viper_code(code).expect("pre-increment should work");
    assert!(output.contains("11"), "got: {}", output);
}

#[test]
fn test_post_increment() {
    let code = r#"
def test():
    x = 10
    print(x++)
    print(x)
test()
"#;
    let output = run_viper_code(code).expect("post-increment should work");
    assert!(output.contains("10"), "got: {}", output);
    assert!(output.contains("11"), "got: {}", output);
}

#[test]
fn test_pre_decrement() {
    let code = r#"
def test():
    x = 10
    print(--x)
    print(x)
test()
"#;
    let output = run_viper_code(code).expect("pre-decrement should work");
    assert!(output.contains("9"), "got: {}", output);
}

#[test]
fn test_post_decrement() {
    let code = r#"
def test():
    x = 10
    print(x--)
    print(x)
test()
"#;
    let output = run_viper_code(code).expect("post-decrement should work");
    assert!(output.contains("10"), "got: {}", output);
    assert!(output.contains("9"), "got: {}", output);
}
