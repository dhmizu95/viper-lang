//! Real-world Scenario Integration Tests

use crate::utils::run_viper_code;

// Calculator
#[test]
fn test_scenario_calculator() {
    let code = r#"
def add(a, b):
    return a + b

def sub(a, b):
    return a - b

def mul(a, b):
    return a * b

def div(a, b):
    return a / b

def test():
    print(add(10, 5))
    print(sub(10, 5))
    print(mul(10, 5))
    print(div(10, 5))
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("15"));
    assert!(output.contains("5"));
}

// Temperature Converter
#[test]
fn test_scenario_temperature_converter() {
    let code = r#"
def celsius_to_fahrenheit(c):
    return c * 9 / 5 + 32

def fahrenheit_to_celsius(f):
    return (f - 32) * 5 / 9

def test():
    print(celsius_to_fahrenheit(100))
    print(fahrenheit_to_celsius(212))
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("212"));
    assert!(output.contains("100"));
}

// Factorial Table
#[test]
fn test_scenario_factorial_table() {
    let code = r#"
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def test():
    i = 1
    while i <= 5:
        print(factorial(i))
        i = i + 1
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Multiplication Table
#[test]
fn test_scenario_multiplication_table() {
    let code = r#"
def test():
    i = 1
    while i <= 5:
        j = 1
        while j <= 5:
            j = j + 1
        i = i + 1
    print("ok")
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("ok"));
}
