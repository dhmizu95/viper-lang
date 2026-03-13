//! Control Flow Integration Tests

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

// If Statements
#[test]
fn test_if_simple() {
    let code = r#"
def test():
    x = 10
    if x > 5:
        print("yes")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_if_else() {
    let code = r#"
def test():
    x = 3
    if x > 5:
        print("greater")
    else:
        print("less")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_if_elif_else() {
    let code = r#"
def test():
    x = 5
    if x > 10:
        print("a")
    elif x > 3:
        print("b")
    else:
        print("c")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// While Loops
#[test]
fn test_while_simple() {
    let code = r#"
def test():
    i = 0
    while i < 5:
        i = i + 1
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_while_with_break() {
    let code = r#"
def test():
    i = 0
    while True:
        if i >= 5:
            break
        i = i + 1
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_while_with_continue() {
    let code = r#"
def test():
    i = 0
    total = 0
    while i < 5:
        i = i + 1
        if i == 3:
            continue
        total = total + i
    print(total)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_while_nested() {
    let code = r#"
def test():
    i = 0
    j = 0
    while i < 3:
        while j < 3:
            j = j + 1
        i = i + 1
        j = 0
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// Keywords
#[test]
fn test_keyword_break() {
    let code = r#"
def test():
    i = 0
    while i < 10:
        if i == 3:
            break
        i = i + 1
    print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_continue() {
    let code = r#"
def test():
    i = 0
    while i < 5:
        i = i + 1
        if i == 3:
            continue
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_keyword_pass() {
    let code = r#"
def test():
    if True:
        pass
    print("ok")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

// ============================================================================
// For Loops - Issue #4 Fix Verification
// ============================================================================

#[test]
fn test_for_range_one_arg() {
    let code = r#"
def test():
    for i in range(3):
        print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_two_args() {
    let code = r#"
def test():
    for i in range(2, 5):
        print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_three_args() {
    let code = r#"
def test():
    for i in range(0, 10, 2):
        print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_multiple_iterations() {
    let code = r#"
def test():
    total = 0
    for i in range(5):
        total = total + i
    print(total)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_with_function_call() {
    let code = r#"
def test():
    for i in range(3):
        print(i)
    print(999)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_nested_operations() {
    let code = r#"
def test():
    result = 1
    for i in range(1, 6):
        result = result * i
    print(result)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_zero_iterations() {
    let code = r#"
def test():
    for i in range(0):
        print(i)
    print("done")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_single_iteration() {
    let code = r#"
def test():
    for i in range(1):
        print(i)
    print("done")
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_large_count() {
    let code = r#"
def test():
    count = 0
    for i in range(100):
        count = count + 1
    print(count)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_with_break() {
    let code = r#"
def test():
    for i in range(10):
        if i == 3:
            break
        print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_with_continue() {
    let code = r#"
def test():
    for i in range(5):
        if i == 2:
            continue
        print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_reverse_step() {
    let code = r#"
def test():
    for i in range(10, 0, -1):
        print(i)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_accumulator() {
    let code = r#"
def test():
    sum = 0
    for i in range(1, 11):
        sum = sum + i
    print(sum)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_string_concat() {
    let code = r#"
def test():
    result = ""
    for i in range(3):
        result = result + "x"
    print(result)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_for_range_fstring() {
    let code = r#"
def test():
    for i in range(3):
        msg = f"Iteration {i}"
        print(msg)
test()
"#;
    assert!(run_viper_code(code).is_ok());
}

#[test]
fn test_while_else_with_return_in_else_body() {
    let code = r#"
def classify(n: int) -> int:
    while n > 0:
        return 1
    else:
        return 2

    print(classify(1))
print(classify(0))
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("1"));
    assert!(output.contains("2"));
}

#[test]
fn test_for_else_with_return_in_else_body() {
    let code = r#"
def first_or_default(n: int) -> int:
    for i in range(n):
        return i
    else:
        return 99

print(first_or_default(0))
print(first_or_default(3))
"#;
    let output = run_viper_code(code).unwrap();
    assert!(output.contains("\n99\n"));
    assert!(output.contains("\n0\n"));
}
