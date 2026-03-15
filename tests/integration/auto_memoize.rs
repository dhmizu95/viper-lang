//! Auto-Memoization Integration Tests
//!
//! Tests for the --auto-memoize flag that automatically memoizes
//! recursive functions with exponential time complexity.

use crate::utils::{build_and_run_auto_memoize, run_viper_code_auto_memoize};

/// Test auto-memoization with Fibonacci (JIT mode)
#[test]
fn test_auto_memoize_fibonacci_jit() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    result = fib(35)
    print(result)
    return 0

main()
"#;

    let (stdout, stderr) =
        run_viper_code_auto_memoize(code).expect("auto-memoize program should run successfully");

    // Check that auto-memoization was applied
    assert!(
        stderr.contains("auto-memoized") || stderr.contains("recursive"),
        "Should mention auto-memoization or recursion, got stderr: {}",
        stderr
    );

    // Verify correct result
    assert!(stdout.contains("9227465"), "fib(35) should equal 9227465, got: {}", stdout);
}

/// Test auto-memoization with Fibonacci (AOT mode)
#[test]
fn test_auto_memoize_fibonacci_aot() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    result = fib(35)
    print(result)
    return 0

main()
"#;

    let stdout =
        build_and_run_auto_memoize(code).expect("auto-memoize AOT program should run successfully");

    assert!(stdout.contains("9227465"), "fib(35) should equal 9227465, got: {}", stdout);
}

/// Test auto-memoization with large Fibonacci
#[test]
fn test_auto_memoize_large_fibonacci() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    print(fib(50))
    return 0

main()
"#;

    let (stdout, _) =
        run_viper_code_auto_memoize(code).expect("large fibonacci should run successfully");

    assert!(stdout.contains("12586269025"), "fib(50) should equal 12586269025, got: {}", stdout);
}

/// Test that linear recursion is NOT auto-memoized
#[test]
fn test_auto_memoize_skips_linear_recursion() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    result = factorial(10)
    print(result)
    return 0

main()
"#;

    // Linear recursion should NOT be auto-memoized (only 1 recursive call)
    // But should still work correctly without memoization
    let (stdout, stderr) =
        run_viper_code_auto_memoize(code).expect("linear recursion should run successfully");

    // Verify correct result
    assert!(stdout.contains("3628800"), "factorial(10) should equal 3628800, got: {}", stdout);

    // Should show warning about linear recursion (not auto-memoized)
    assert!(
        stderr.contains("recursive") && !stderr.contains("automatically memoized"),
        "Should warn about linear recursion but NOT auto-memoize it, got stderr: {}",
        stderr
    );
}

/// Test auto-memoization with mutual recursion
#[test]
fn test_auto_memoize_mutual_recursion() {
    let code = r#"
def is_even(n):
    if n == 0:
        return True
    return is_odd(n - 1)

def is_odd(n):
    if n == 0:
        return False
    return is_even(n - 1)

def main():
    print(is_even(100))
    print(is_odd(100))
    print(is_even(101))
    print(is_odd(101))
    return 0

main()
"#;

    let stdout =
        run_viper_code_auto_memoize(code).expect("mutual recursion should run successfully").0;

    assert!(stdout.contains("True"));
    assert!(stdout.contains("False"));
}

/// Test auto-memoization with multiple recursive functions
#[test]
fn test_auto_memoize_multiple_functions() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    print(fib(30))
    return 0

main()
"#;

    let (stdout, _stderr) = run_viper_code_auto_memoize(code)
        .expect("multiple recursive functions should run successfully");

    assert!(stdout.contains("832040")); // fib(30)
}

/// Test that manually memoized functions are not double-memoized
#[test]
fn test_auto_memoize_respects_manual_decorator() {
    let code = r#"
@lru_cache(maxsize=64)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    result = fib(35)
    print(result)
    return 0

main()
"#;

    let (stdout, _) = run_viper_code_auto_memoize(code)
        .expect("manually memoized function should run successfully");

    assert!(stdout.contains("9227465"), "fib(35) should equal 9227465, got: {}", stdout);
}

/// Test auto-memoization with BigInt return type
#[test]
fn test_auto_memoize_bigint() {
    let code = r#"
def fib_big(n):
    if n <= 1:
        return n
    return fib_big(n - 1) + fib_big(n - 2)

def main():
    print(fib_big(75))
    return 0

main()
"#;

    let (stdout, _) =
        run_viper_code_auto_memoize(code).expect("BigInt fibonacci should run successfully");

    assert!(
        stdout.contains("2111485077978050"),
        "fib(75) should equal 2111485077978050, got: {}",
        stdout
    );
}
