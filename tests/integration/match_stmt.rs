//! Match Statement Integration Tests
//! Covers: match/case, wildcard, constant, variable, tuple, list, guard patterns

use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_viper_code(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;
    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run"])
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
