//! With Statement Integration Tests
//! Covers: with statement, context managers, __enter__/__exit__

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
// Context Manager via Class
// ============================================================================

#[test]
fn test_with_basic_enter_exit() {
    let code = r#"
class Ctx:
    def __enter__(self):
        print("enter")
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        print("exit")
        return False

def test():
    with Ctx():
        print("inside")
test()
"#;
    let output = run_viper_code(code).expect("with statement should work");
    assert!(output.contains("enter"), "got: {}", output);
    assert!(output.contains("inside"), "got: {}", output);
    assert!(output.contains("exit"), "got: {}", output);
}

#[test]
fn test_with_as_binding() {
    let code = r#"
class Resource:
    def __init__(self, name):
        self.name = name

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        return False

def test():
    with Resource("db") as r:
        print(r.name)
test()
"#;
    let output = run_viper_code(code).expect("with...as should work");
    assert!(output.contains("db"), "got: {}", output);
}

#[test]
fn test_with_exit_called_on_success() {
    let code = r#"
class Tracker:
    def __init__(self):
        self.exited = False

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.exited = True
        return False

def test():
    t = Tracker()
    with t:
        pass
    print(t.exited)
test()
"#;
    let output = run_viper_code(code).expect("__exit__ called on success should work");
    assert!(output.contains("True"), "got: {}", output);
}

#[test]
fn test_with_exit_called_on_exception() {
    let code = r#"
class Suppressor:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        print("cleanup")
        return True  # suppress the exception

def test():
    with Suppressor():
        raise ValueError("test")
    print("after with")
test()
"#;
    let output = run_viper_code(code).expect("with exception suppression should work");
    assert!(output.contains("cleanup"), "got: {}", output);
    assert!(output.contains("after with"), "got: {}", output);
}

#[test]
fn test_with_body_executes() {
    let code = r#"
class NullCtx:
    def __enter__(self):
        return None

    def __exit__(self, exc_type, exc_val, exc_tb):
        return False

def test():
    result = 0
    with NullCtx():
        result = 42
    print(result)
test()
"#;
    let output = run_viper_code(code).expect("with body should execute");
    assert!(output.contains("42"), "got: {}", output);
}

#[test]
fn test_with_nested() {
    let code = r#"
class Logger:
    def __init__(self, name):
        self.name = name

    def __enter__(self):
        print("open " + self.name)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        print("close " + self.name)
        return False

def test():
    with Logger("outer"):
        with Logger("inner"):
            print("work")
test()
"#;
    let output = run_viper_code(code).expect("nested with should work");
    assert!(output.contains("open outer"), "got: {}", output);
    assert!(output.contains("open inner"), "got: {}", output);
    assert!(output.contains("work"), "got: {}", output);
    assert!(output.contains("close inner"), "got: {}", output);
    assert!(output.contains("close outer"), "got: {}", output);
}

#[test]
fn test_with_return_value_from_enter() {
    let code = r#"
class ValueCtx:
    def __enter__(self):
        return 99

    def __exit__(self, exc_type, exc_val, exc_tb):
        return False

def test():
    with ValueCtx() as v:
        print(v)
test()
"#;
    let output = run_viper_code(code).expect("with enter return value should work");
    assert!(output.contains("99"), "got: {}", output);
}
