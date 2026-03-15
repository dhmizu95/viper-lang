use crate::utils::run_viper_code;

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

    let stdout = run_viper_code(code).expect("should run");
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

    let _ = run_viper_code(code).expect("factorial cache program should run successfully");
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

    let (_, stderr) = crate::utils::run_viper_code_with_stderr(code).expect("should run");
    // Should warn about recursive function without memoization
    assert!(
        stderr.contains("recursive") || stderr.contains("lru_cache"),
        "Should warn about recursive function, got stderr: {}",
        stderr
    );
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

    let stdout = run_viper_code(code).expect("should run");
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

    let stdout =
        run_viper_code(code).expect("large fibonacci cache program should run successfully");
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

    let stdout = run_viper_code(code)
        .expect("bounded large fibonacci cache program should run successfully");
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

    let stdout = run_viper_code(code).expect("decorator program should run successfully");
    assert!(stdout.contains("double(5) ="), "unexpected stdout: {}", stdout);
    assert!(stdout.contains("double(10) ="), "unexpected stdout: {}", stdout);
}
