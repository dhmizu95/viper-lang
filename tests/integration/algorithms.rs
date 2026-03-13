//! Algorithm Integration Tests

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

// Fibonacci
#[test]
fn test_algo_fibonacci_recursive() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def test():
    print(fib(10))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_algo_fibonacci_iterative() {
    let code = r#"
def fib(n):
    if n <= 1:
        return n
    a = 0
    b = 1
    i = 2
    while i <= n:
        temp = a + b
        a = b
        b = temp
        i = i + 1
    return b

def test():
    print(fib(10))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Factorial
#[test]
fn test_algo_factorial_recursive() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test():
    print(factorial(5))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_algo_factorial_iterative() {
    let code = r#"
def factorial(n):
    result = 1
    i = 2
    while i <= n:
        result = result * i
        i = i + 1
    return result

def test():
    print(factorial(5))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// GCD
#[test]
fn test_algo_gcd() {
    let code = r#"
def gcd(a, b):
    while b != 0:
        temp = b
        b = a % b
        a = temp
    return a

def test():
    print(gcd(48, 18))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Power
#[test]
fn test_algo_power_iterative() {
    let code = r#"
def power(base, exp):
    result = 1
    i = 0
    while i < exp:
        result = result * base
        i = i + 1
    return result

def test():
    print(power(2, 10))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_algo_power_recursive() {
    let code = r#"
def power(base, exp):
    if exp == 0:
        return 1
    return base * power(base, exp - 1)

def test():
    print(power(2, 10))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Prime Check
#[test]
fn test_algo_is_prime() {
    let code = r#"
def is_prime(n):
    if n < 2:
        return False
    i = 2
    while i * i <= n:
        if n % i == 0:
            return False
        i = i + 1
    return True

def test():
    print(is_prime(17))
    print(is_prime(18))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Sum Range
#[test]
fn test_algo_sum_range() {
    let code = r#"
def sum_range(n):
    total = 0
    i = 1
    while i <= n:
        total = total + i
        i = i + 1
    return total

def test():
    print(sum_range(100))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Count Digits
#[test]
fn test_algo_count_digits() {
    let code = r#"
def count_digits(n):
    count = 0
    while n > 0:
        n = n // 10
        count = count + 1
    return count

def test():
    print(count_digits(12345))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Reverse Number
#[test]
fn test_algo_reverse_number() {
    let code = r#"
def reverse_number(n):
    result = 0
    while n > 0:
        result = result * 10 + n % 10
        n = n // 10
    return result

def test():
    print(reverse_number(12345))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Palindrome
#[test]
fn test_algo_palindrome() {
    let code = r#"
def is_palindrome(n):
    original = n
    reversed_n = 0
    while n > 0:
        reversed_n = reversed_n * 10 + n % 10
        n = n // 10
    return original == reversed_n

def test():
    print(is_palindrome(121))
    print(is_palindrome(123))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Armstrong Number
#[test]
fn test_algo_armstrong() {
    let code = r#"
def is_armstrong(n):
    original = n
    sum_cubes = 0
    while n > 0:
        digit = n % 10
        sum_cubes = sum_cubes + digit * digit * digit
        n = n // 10
    return original == sum_cubes

def test():
    print(is_armstrong(153))
    print(is_armstrong(100))
test()
"#;
    assert!(run_viper_code(code).is_ok());
}
