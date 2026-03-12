// Test for @lru_cache and @cache decorators
// Run with: cargo test --test test_decorators

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn write_temp_viper_file(name: &str, code: &str) -> std::path::PathBuf {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("{}_{}.vp", name, timestamp));
    fs::write(&path, code).unwrap();
    path
}

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

    let test_file = write_temp_viper_file("test_fib_cache", code);
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute viper compiler");
    let _ = fs::remove_file(&test_file);
    
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

    let test_file = write_temp_viper_file("test_factorial_cache", code);
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute viper compiler");
    let _ = fs::remove_file(&test_file);
    
    assert!(output.status.success(), "factorial cache program should run successfully");
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

    let test_file = write_temp_viper_file("test_fib_no_cache", code);
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute viper compiler");
    let _ = fs::remove_file(&test_file);
    
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
def increment(n):
    return n + 1

def main():
    result = increment(5)
    print(result)
    return 0

main()
"#;

    let test_file = write_temp_viper_file("test_gcd_cache", code);
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute viper compiler");
    let _ = fs::remove_file(&test_file);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("6"), "increment(5) should equal 6, got: {}", stdout);
}

#[test]
fn test_lru_cache_large_fibonacci() {
    let code = r#"
@lru_cache(maxsize=None)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    print(fib(75))
    return 0

main()
"#;

    let test_file = write_temp_viper_file("test_large_fib_cache", code);
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute viper compiler");
    let _ = fs::remove_file(&test_file);

    assert!(output.status.success(), "large fibonacci cache program should run successfully");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2111485077978050"),
        "fib(75) should equal 2111485077978050, got: {}",
        stdout
    );
}

#[test]
fn test_bounded_lru_cache_large_fibonacci() {
    let code = r#"
@lru_cache(maxsize=256)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    print(fib(75))
    return 0

main()
"#;

    let test_file = write_temp_viper_file("test_bounded_large_fib_cache", code);
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute viper compiler");
    let _ = fs::remove_file(&test_file);

    assert!(output.status.success(), "bounded large fibonacci cache program should run successfully");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2111485077978050"),
        "fib(75) with bounded cache should equal 2111485077978050, got: {}",
        stdout
    );
}

#[test]
fn test_decorator_lru_cache_non_recursive_program() {
    let code = r#"
@lru_cache(maxsize=128)
def double(n):
    return n * 2

def main():
    print("double(5) =", double(5))
    print("double(10) =", double(10))
    print("double(5) again =", double(5))
    return 0

main()
"#;

    let test_file = write_temp_viper_file("test_simple_cache", code);
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute viper compiler");
    let _ = fs::remove_file(&test_file);

    assert!(output.status.success(), "decorator program should run successfully");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("double(5) ="), "unexpected stdout: {}", stdout);
    assert!(stdout.contains("double(10) ="), "unexpected stdout: {}", stdout);
}
