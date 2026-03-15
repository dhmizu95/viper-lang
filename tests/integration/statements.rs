//! Statement Integration Tests

use crate::utils::run_viper_code;

// Assignment
#[test]
fn test_assign_simple() {
    let code = r#"
def test():
    x = 42
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Declare with type
#[test]
fn test_declare_with_type() {
    let code = r#"
def test():
    x: int = 42
    print(x)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Const
#[test]
fn test_const() {
    let code = r#"
const PI = 3.14159

def test():
    print(PI)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Assert
#[test]
fn test_assert_simple() {
    let code = r#"
def test():
    x = 5
    assert x > 0
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_assert_with_message() {
    let code = r#"
def test():
    x = 5
    assert x > 0, "x must be positive"
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Delimiters
#[test]
fn test_delimiters_parentheses() {
    let code = r#"
def test():
    a = (1 + 2) * 3
    print(a)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_delimiters_colon() {
    let code = r#"
def test():
    if True:
        print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Keywords
#[test]
fn test_keyword_def() {
    let code = r#"
def my_func():
    print("ok")

def test():
    my_func()
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_while_kw() {
    let code = r#"
def test():
    i = 0
    while i < 3:
        i = i + 1
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_return() {
    let code = r#"
def get_value():
    return 42

def test():
    print(get_value())
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_if_elif_else() {
    let code = r#"
def test():
    x = 10
    if x > 20:
        print("a")
    elif x > 5:
        print("b")
    else:
        print("c")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// Multiple Assignment - Issue #5
// Basic tuple unpacking is now supported including:
// - Literal tuples: a, b = 1, 2
// - Expression tuples: a, b = x + 1, y + 2
// - Function returns: x, y = get_pair()
// ============================================================================

#[test]
fn test_multiple_assignment_basic() {
    let code = r#"
def test():
    a, b = 1, 2
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_with_expressions() {
    let code = r#"
def test():
    x = 10
    y = 20
    a, b = x + 1, y + 2
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_swap() {
    let code = r#"
def test():
    a = 1
    b = 2
    a, b = b, a
    print(a)
    print(b)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_three_vars() {
    let code = r#"
def test():
    a, b, c = 1, 2, 3
    print(a)
    print(b)
    print(c)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_multiple_assignment_with_function_return() {
    let code = r#"
def get_pair():
    return 10, 20

def test():
    x, y = get_pair()
    print(x)
    print(y)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_delete_simple() {
    let code = r#"
def test():
    x = 42
    del x
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_delete_list_item() {
    let code = r#"
def test():
    l = [1, 2, 3]
    del l[1]
    print(len(l))
test()
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("2"));
}
