// Test for @lru_cache and @cache decorators
// Run with: cargo test --test test_decorators

use std::process::Command;

/// Test @lru_cache decorator with fibonacci
#[test]
fn test_lru_cache_fibonacci() {
    let code = r#"
@lru_cache(maxsize=128)
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

    // Write test file
    std::fs::write("/tmp/test_fib_cache.vp", code).unwrap();
    
    // Compile and run
    let output = Command::new("cargo")
        .args(&["run", "--", "/tmp/test_fib_cache.vp"])
        .output()
        .expect("Failed to execute viper compiler");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("9227465"), "fib(35) should equal 9227465, got: {}", stdout);
}

/// Test @cache decorator (unbounded)
#[test]
fn test_cache_unbounded() {
    let code = r#"
@cache
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

    std::fs::write("/tmp/test_factorial_cache.vp", code).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "/tmp/test_factorial_cache.vp"])
        .output()
        .expect("Failed to execute viper compiler");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3628800"), "factorial(10) should equal 3628800, got: {}", stdout);
}

/// Test recursion detection warning
#[test]
fn test_recursion_warning_without_cache() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    result = fib(10)
    print(result)
    return 0

main()
"#;

    std::fs::write("/tmp/test_fib_no_cache.vp", code).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "/tmp/test_fib_no_cache.vp"])
        .output()
        .expect("Failed to execute viper compiler");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should warn about recursive function without memoization
    assert!(stderr.contains("recursive") || stderr.contains("lru_cache"), 
            "Should warn about recursive function, got stderr: {}", stderr);
}

/// Test @lru_cache with maxsize parameter
#[test]
fn test_lru_cache_maxsize() {
    let code = r#"
@lru_cache(maxsize=256)
def gcd(a, b):
    if b == 0:
        return a
    return gcd(b, a % b)

def main():
    result = gcd(48, 18)
    print(result)
    return 0

main()
"#;

    std::fs::write("/tmp/test_gcd_cache.vp", code).unwrap();
    
    let output = Command::new("cargo")
        .args(&["run", "--", "/tmp/test_gcd_cache.vp"])
        .output()
        .expect("Failed to execute viper compiler");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("6"), "gcd(48, 18) should equal 6, got: {}", stdout);
}
