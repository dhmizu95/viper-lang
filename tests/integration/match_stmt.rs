//! Match Statement Integration Tests
//! Covers: match/case, wildcard, constant, variable, tuple, list, guard patterns

use crate::utils::run_viper_code;

// ============================================================================
// Constant Patterns
// ============================================================================

#[test]
fn test_match_int_constant() {
    let code = r#"
def test():
    x = 2
    match x:
        case 1:
            print("one")
        case 2:
            print("two")
        case 3:
            print("three")
test()
"#;
    let output = run_viper_code(code).expect("match int constant should work");
    assert!(output.contains("two"), "got: {}", output);
}

#[test]
fn test_match_string_constant() {
    let code = r#"
def test():
    cmd = "quit"
    match cmd:
        case "start":
            print("starting")
        case "stop":
            print("stopping")
        case "quit":
            print("quitting")
test()
"#;
    let output = run_viper_code(code).expect("match string constant should work");
    assert!(output.contains("quitting"), "got: {}", output);
}

#[test]
fn test_match_bool_constant() {
    let code = r#"
def test():
    flag = True
    match flag:
        case True:
            print("yes")
        case False:
            print("no")
test()
"#;
    let output = run_viper_code(code).expect("match bool constant should work");
    assert!(output.contains("yes"), "got: {}", output);
}

// ============================================================================
// Wildcard Pattern
// ============================================================================

#[test]
fn test_match_wildcard() {
    let code = r#"
def test():
    x = 99
    match x:
        case 1:
            print("one")
        case _:
            print("other")
test()
"#;
    let output = run_viper_code(code).expect("match wildcard should work");
    assert!(output.contains("other"), "got: {}", output);
}

#[test]
fn test_match_wildcard_as_default() {
    let code = r#"
def classify(n):
    match n:
        case 0:
            return "zero"
        case 1:
            return "one"
        case _:
            return "many"

def test():
    print(classify(0))
    print(classify(1))
    print(classify(42))
test()
"#;
    let output = run_viper_code(code).expect("match wildcard as default should work");
    assert!(output.contains("zero"), "got: {}", output);
    assert!(output.contains("one"), "got: {}", output);
    assert!(output.contains("many"), "got: {}", output);
}

// ============================================================================
// Variable Binding
// ============================================================================

#[test]
fn test_match_variable_binding() {
    let code = r#"
def test():
    x = 42
    match x:
        case n:
            print(n)
test()
"#;
    let output = run_viper_code(code).expect("match variable binding should work");
    assert!(output.contains("42"), "got: {}", output);
}

// ============================================================================
// Guard Conditions
// ============================================================================

#[test]
fn test_match_guard_condition() {
    let code = r#"
def classify(n):
    match n:
        case x if x < 0:
            return "negative"
        case x if x == 0:
            return "zero"
        case x if x > 0:
            return "positive"

def test():
    print(classify(-5))
    print(classify(0))
    print(classify(7))
test()
"#;
    let output = run_viper_code(code).expect("match guard should work");
    assert!(output.contains("negative"), "got: {}", output);
    assert!(output.contains("zero"), "got: {}", output);
    assert!(output.contains("positive"), "got: {}", output);
}

// ============================================================================
// Tuple Patterns
// ============================================================================

#[test]
fn test_match_tuple_pattern() {
    let code = r#"
def test():
    point = (0, 1)
    match point:
        case (0, 0):
            print("origin")
        case (0, y):
            print("y-axis")
        case (x, 0):
            print("x-axis")
        case (x, y):
            print("other")
test()
"#;
    let output = run_viper_code(code).expect("match tuple pattern should work");
    assert!(output.contains("y-axis"), "got: {}", output);
}

#[test]
fn test_match_tuple_destructuring() {
    let code = r#"
def describe_pair(pair):
    match pair:
        case (a, b) if a == b:
            return "equal"
        case (a, b) if a > b:
            return "greater"
        case (a, b):
            return "lesser"

def test():
    print(describe_pair((5, 5)))
    print(describe_pair((7, 3)))
    print(describe_pair((2, 8)))
test()
"#;
    let output = run_viper_code(code).expect("match tuple destructuring should work");
    assert!(output.contains("equal"), "got: {}", output);
    assert!(output.contains("greater"), "got: {}", output);
    assert!(output.contains("lesser"), "got: {}", output);
}

// ============================================================================
// Multiple Cases in Sequence
// ============================================================================

#[test]
fn test_match_first_case_wins() {
    let code = r#"
def test():
    x = 5
    match x:
        case 5:
            print("five")
        case 5:
            print("also five")
test()
"#;
    // First match wins — should only see "five" once
    let output = run_viper_code(code).expect("match first case wins should work");
    let count = output.matches("five").count();
    assert_eq!(count, 1, "only first case should match, got: {}", output);
}

#[test]
fn test_match_in_loop() {
    let code = r#"
def test():
    for i in range(4):
        match i:
            case 0:
                print("zero")
            case 1:
                print("one")
            case _:
                print("other")
test()
"#;
    let output = run_viper_code(code).expect("match in loop should work");
    assert!(output.contains("zero"), "got: {}", output);
    assert!(output.contains("one"), "got: {}", output);
    assert!(output.contains("other"), "got: {}", output);
}

#[test]
fn test_match_none_pattern() {
    let code = r#"
def test():
    val = None
    match val:
        case None:
            print("nothing")
        case _:
            print("something")
test()
"#;
    let output = run_viper_code(code).expect("match None should work");
    assert!(output.contains("nothing"), "got: {}", output);
}
