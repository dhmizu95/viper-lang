//! Control Flow Integration Tests

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
