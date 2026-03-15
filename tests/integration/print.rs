//! Print Function Integration Tests

mod utils;
use utils::run_code;

// Integer Printing
#[test]
fn test_print_int_positive() {
    let code = r#"
def test():
    print(42)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("42"));
}

#[test]
fn test_print_int_negative() {
    let code = r#"
def test():
    print(-17)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("-17"));
}

#[test]
fn test_print_int_zero() {
    let code = r#"
def test():
    print(0)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("0"));
}

#[test]
fn test_print_int_large() {
    let code = r#"
def test():
    print(999999999999999999)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("999999999999999999"));
}

// Float Printing
#[test]
fn test_print_float_basic() {
    let code = r#"
def test():
    print(3.14)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("3.14"));
}

#[test]
fn test_print_float_negative() {
    let code = r#"
def test():
    print(-2.5)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("-2.5"));
}

#[test]
fn test_print_float_scientific() {
    let code = r#"
def test():
    print(1.5e10)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("e"));
}

// String Printing
#[test]
fn test_print_string_double_quotes() {
    let code = r#"
def test():
    print("Hello, World!")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Hello, World!"));
}

#[test]
fn test_print_string_single_quotes() {
    let code = r#"
def test():
    print('Single quotes')
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Single quotes"));
}

#[test]
fn test_print_string_empty() {
    let code = r#"
def test():
    print("")
test()
"#;
    // Empty string prints as just a newline - verify code runs without error
    let result = run_code(code);
    assert!(result.is_ok());
}

#[test]
fn test_print_string_with_escape() {
    let code = r#"
def test():
    print("Line1\nLine2")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Line1"));
    assert!(output.contains("Line2"));
}

// Boolean Printing
#[test]
fn test_print_bool_true() {
    let code = r#"
def test():
    print(True)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("True"));
}

#[test]
fn test_print_bool_false() {
    let code = r#"
def test():
    print(False)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("False"));
}

// None Printing
#[test]
fn test_print_none() {
    let code = r#"
def test():
    print(None)
test()
"#;
    let output = run_code(code).unwrap();
    // None is represented as 0 in Viper's internal representation
    assert!(output.contains("0"));
}

// Multiple Arguments
#[test]
fn test_print_multiple_args() {
    let code = r#"
def test():
    print(1, 2, 3)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
    assert!(output.contains("3"));
}

#[test]
fn test_print_mixed_types() {
    let code = r#"
def test():
    print("Value:", 42, "is", True)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Value:"));
    assert!(output.contains("42"));
    assert!(output.contains("is"));
    assert!(output.contains("True"));
}

// Variable Printing
#[test]
fn test_print_variable() {
    let code = r#"
def test():
    x = 100
    print(x)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("100"));
}

#[test]
fn test_print_string_variable() {
    let code = r#"
def test():
    msg = "Hello"
    print(msg)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Hello"));
}

// Expression Printing
#[test]
fn test_print_arithmetic_expression() {
    let code = r#"
def test():
    print(10 + 5 * 2)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("20"));
}

#[test]
fn test_print_comparison_expression() {
    let code = r#"
def test():
    print(10 > 5)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("True"));
}

#[test]
fn test_print_string_concatenation() {
    let code = r#"
def test():
    print("Hello" + " " + "World")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Hello World"));
}

// Function Return Value Printing
#[test]
fn test_print_function_return() {
    let code = r#"
def add(a, b):
    return a + b

def test():
    print(add(3, 7))
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("10"));
}

// BigInt Printing
#[test]
fn test_print_bigint() {
    let code = r#"
def test():
    x = 123456789012345678901234567890
    print(x)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("123456789012345678901234567890"));
}

// List Printing
#[test]
fn test_print_list() {
    let code = r#"
def test():
    lst = [1, 2, 3]
    print(lst)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("["));
    assert!(output.contains("]"));
}

#[test]
fn test_print_list_empty() {
    let code = r#"
def test():
    lst = []
    print(lst)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("[]"));
}

// Dict Printing
#[test]
fn test_print_dict() {
    let code = r#"
def test():
    d = {"a": 1, "b": 2}
    print(d)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("{"));
    assert!(output.contains("}"));
}

// Bytes Printing
#[test]
fn test_print_bytes() {
    let code = r#"
def test():
    b = b"hello"
    print(b)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("hello"));
}

// f-string Printing
#[test]
fn test_print_fstring() {
    let code = r#"
def test():
    name = "Viper"
    print(f"Hello, {name}!")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Hello, Viper!"));
}

#[test]
fn test_print_fstring_with_int() {
    let code = r#"
def test():
    x = 42
    print(f"The answer is {x}")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("The answer is 42"));
}

#[test]
fn test_print_fstring_with_float() {
    let code = r#"
def test():
    pi = 3.14159
    print(f"Pi = {pi}")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Pi = 3.14159"));
}

#[test]
fn test_print_fstring_with_expression() {
    let code = r#"
def test():
    a = 10
    b = 20
    print(f"Sum = {a + b}")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Sum = 30"));
}

// Multiple Print Statements
#[test]
fn test_print_multiple_statements() {
    let code = r#"
def test():
    print("Line 1")
    print("Line 2")
    print("Line 3")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("Line 1"));
    assert!(output.contains("Line 2"));
    assert!(output.contains("Line 3"));
}

// Print in Loop
#[test]
fn test_print_in_loop() {
    let code = r#"
def test():
    for i in range(3):
        print(i)
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("0"));
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

// Print with Conditional
#[test]
fn test_print_conditional() {
    let code = r#"
def test():
    x = 10
    if x > 5:
        print("greater")
    else:
        print("lesser")
test()
"#;
    let output = run_code(code).unwrap();
    assert!(output.contains("greater"));
}
