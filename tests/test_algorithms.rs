//! Algorithm Integration Tests

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

// Fibonacci
#[test]
fn test_algo_fibonacci_recursive() {
    assert!(run_viper_code("def fib(n):\n    if n <= 1:\n        return n\n    return fib(n - 1) + fib(n - 2)\ndef test():\n    print(fib(10))\ntest()").is_ok());
}

#[test]
fn test_algo_fibonacci_iterative() {
    assert!(run_viper_code("def fib(n):\n    if n <= 1:\n        return n\n    a = 0\n    b = 1\n    i = 2\n    while i <= n:\n        temp = a + b\n        a = b\n        b = temp\n        i = i + 1\n    return b\ndef test():\n    print(fib(10))\ntest()").is_ok());
}

// Factorial
#[test]
fn test_algo_factorial_recursive() {
    assert!(run_viper_code("def factorial(n):\n    if n <= 1:\n        return 1\n    return n * factorial(n - 1)\ndef test():\n    print(factorial(5))\ntest()").is_ok());
}

#[test]
fn test_algo_factorial_iterative() {
    assert!(run_viper_code("def factorial(n):\n    result = 1\n    i = 2\n    while i <= n:\n        result = result * i\n        i = i + 1\n    return result\ndef test():\n    print(factorial(5))\ntest()").is_ok());
}

// GCD
#[test]
fn test_algo_gcd() {
    assert!(run_viper_code("def gcd(a, b):\n    while b != 0:\n        temp = b\n        b = a % b\n        a = temp\n    return a\ndef test():\n    print(gcd(48, 18))\ntest()").is_ok());
}

// Power
#[test]
fn test_algo_power_iterative() {
    assert!(run_viper_code("def power(base, exp):\n    result = 1\n    i = 0\n    while i < exp:\n        result = result * base\n        i = i + 1\n    return result\ndef test():\n    print(power(2, 10))\ntest()").is_ok());
}

#[test]
fn test_algo_power_recursive() {
    assert!(run_viper_code("def power(base, exp):\n    if exp == 0:\n        return 1\n    return base * power(base, exp - 1)\ndef test():\n    print(power(2, 10))\ntest()").is_ok());
}

// Prime Check
#[test]
fn test_algo_is_prime() {
    assert!(run_viper_code("def is_prime(n):\n    if n < 2:\n        return False\n    i = 2\n    while i * i <= n:\n        if n % i == 0:\n            return False\n        i = i + 1\n    return True\ndef test():\n    print(is_prime(17))\n    print(is_prime(18))\ntest()").is_ok());
}

// Sum Range
#[test]
fn test_algo_sum_range() {
    assert!(run_viper_code("def sum_range(n):\n    total = 0\n    i = 1\n    while i <= n:\n        total = total + i\n        i = i + 1\n    return total\ndef test():\n    print(sum_range(100))\ntest()").is_ok());
}

// Count Digits
#[test]
fn test_algo_count_digits() {
    assert!(run_viper_code("def count_digits(n):\n    count = 0\n    while n > 0:\n        n = n // 10\n        count = count + 1\n    return count\ndef test():\n    print(count_digits(12345))\ntest()").is_ok());
}

// Reverse Number
#[test]
fn test_algo_reverse_number() {
    assert!(run_viper_code("def reverse_number(n):\n    result = 0\n    while n > 0:\n        result = result * 10 + n % 10\n        n = n // 10\n    return result\ndef test():\n    print(reverse_number(12345))\ntest()").is_ok());
}

// Palindrome
#[test]
fn test_algo_palindrome() {
    assert!(run_viper_code("def is_palindrome(n):\n    original = n\n    reversed_n = 0\n    while n > 0:\n        reversed_n = reversed_n * 10 + n % 10\n        n = n // 10\n    return original == reversed_n\ndef test():\n    print(is_palindrome(121))\n    print(is_palindrome(123))\ntest()").is_ok());
}

// Armstrong Number
#[test]
fn test_algo_armstrong() {
    assert!(run_viper_code("def is_armstrong(n):\n    original = n\n    sum_cubes = 0\n    while n > 0:\n        digit = n % 10\n        sum_cubes = sum_cubes + digit * digit * digit\n        n = n // 10\n    return original == sum_cubes\ndef test():\n    print(is_armstrong(153))\n    print(is_armstrong(100))\ntest()").is_ok());
}
