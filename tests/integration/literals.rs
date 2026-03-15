//! Literal Integration Tests

use crate::utils::run_viper_code;

// Integer Literals
#[test]
fn test_int_literals_basic() {
    let code = r#"
def test():
    a = 42
    b = -17
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_int_literals_hex() {
    let code = r#"
def test():
    a = 0xFF
    b = 0x1A
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_int_literals_binary() {
    let code = r#"
def test():
    a = 0b1010
    b = 0b1111
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_int_literals_octal() {
    let code = r#"
def test():
    a = 0o755
    b = 0o644
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Float Literals
#[test]
fn test_float_literals_basic() {
    let code = r#"
def test():
    a = 3.14
    b = -2.5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_float_literals_exponent() {
    let code = r#"
def test():
    a = 2.5e10
    b = 1.0e-5
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// String Literals
#[test]
fn test_string_literals_double_quote() {
    let code = r#"
def test():
    a = "hello"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_literals_single_quote() {
    let code = r#"
def test():
    a = 'world'
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_literals_escape() {
    let code = r#"
def test():
    a = "hello\nworld"
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Boolean Literals
#[test]
fn test_bool_literals_true() {
    let code = r#"
def test():
    a = True
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bool_literals_false() {
    let code = r#"
def test():
    a = False
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// None Literal
#[test]
fn test_none_literal() {
    let code = r#"
def test():
    a = None
    b = None
    print(a is b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// BigInt Literals
#[test]
fn test_bigint_literals() {
    let code = r#"
def test():
    a = 123456789012345678901234567890
    b = 999999999999999999999999999999
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// F-String and Bytes Literals - Fixed and Working
// ============================================================================

#[test]
fn test_fstring_literals() {
    let code = r#"
def test():
    name = "World"
    a = f"Hello {name}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_int_interpolation() {
    let code = r#"
def test():
    age = 25
    a = f"Age: {age}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_multiple_interpolations() {
    let code = r#"
def test():
    name = "Alice"
    age = 30
    a = f"My name is {name} and I am {age} years old"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_expression() {
    let code = r#"
def test():
    x = 10
    y = 20
    a = f"Sum: {x + y}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bytes_literals() {
    let code = r#"
def test():
    a = b"hello"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bytes_literals_hex() {
    let code = r#"
def test():
    a = b"\x48\x65\x6c\x6c\x6f"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bytes_literals_empty() {
    let code = r#"
def test():
    a = b""
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_concatenation() {
    let code = r#"
def test():
    a = "Hello " + "World"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// Additional F-String Integration Tests (Issue #1)
// ============================================================================

#[test]
fn test_fstring_with_bool() {
    let code = r#"
def test():
    flag = True
    a = f"Flag is {flag}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_float() {
    let code = r#"
def test():
    pi = 3.14
    a = f"Pi is {pi}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_none() {
    let code = r#"
def test():
    val = None
    a = f"Value is {val}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_string_variable() {
    let code = r#"
def test():
    greeting = "Hello"
    a = f"{greeting} World"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_method_call() {
    let code = r#"
def get_name():
    return "Alice"

def test():
    a = f"Name: {get_name()}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_arithmetic() {
    let code = r#"
def test():
    x = 10
    y = 5
    a = f"Result: {x * y}"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_nested() {
    let code = r#"
def test():
    a = 1
    b = 2
    c = 3
    result = f"Sum: {a + b + c}, Product: {a * b * c}"
    print(result)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_fstring_with_literal_mix() {
    let code = r#"
def test():
    name = "Bob"
    a = f"Hello {name}, you are {25 + 5} years old"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// Additional Bytes Literal Integration Tests (Issue #2)
// ============================================================================

#[test]
fn test_bytes_with_escape_sequences() {
    let code = r#"
def test():
    a = b"line1\nline2"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bytes_with_tab() {
    let code = r#"
def test():
    a = b"col1\tcol2"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bytes_single_quote() {
    let code = r#"
def test():
    a = b'hello'
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bytes_mixed_quotes() {
    let code = r#"
def test():
    a = b"it's"
    b = b'he said "hi"'
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_bytes_long() {
    let code = r#"
def test():
    a = b"this is a longer bytes string for testing purposes"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// Additional String Concatenation Integration Tests (Issue #3)
// ============================================================================

#[test]
fn test_string_concat_multiple() {
    let code = r#"
def test():
    a = "Hello" + " " + "World" + "!"
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_concat_with_variables() {
    let code = r#"
def test():
    first = "Hello"
    second = "World"
    a = first + " " + second
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_concat_empty() {
    let code = r#"
def test():
    a = "" + "hello"
    b = "hello" + ""
    c = "" + ""
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_concat_long() {
    let code = r#"
def test():
    a = "This is a longer string part one "
    b = "and this is part two of the string"
    c = a + b
    print(c)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_concat_chained() {
    let code = r#"
def test():
    a = "A"
    b = "B"
    c = "C"
    d = a + b
    e = d + c
    print(e)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_string_concat_with_result() {
    let code = r#"
def greet(name: str):
    return "Hello " + name

def test():
    result = greet("World")
    print(result)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}
