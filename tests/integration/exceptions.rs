//! Exception Handling Integration Tests
//! Covers: try/except, except with type, try/else, try/finally, raise, propagation

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
// Basic try/except
// ============================================================================

#[test]
fn test_try_except_no_exception() {
    let code = r#"
def test():
    try:
        x = 10
        print(x)
    except:
        print("error")
test()
"#;
    let output = run_viper_code(code).expect("try/except no exception should work");
    assert!(output.contains("10"), "got: {}", output);
    assert!(!output.contains("error"), "got: {}", output);
}

#[test]
fn test_try_except_catches_exception() {
    let code = r#"
def test():
    try:
        raise ValueError("oops")
    except:
        print("caught")
test()
"#;
    let output = run_viper_code(code).expect("try/except catching should work");
    assert!(output.contains("caught"), "got: {}", output);
}

#[test]
fn test_try_except_with_type() {
    let code = r#"
def test():
    try:
        raise ValueError("bad value")
    except ValueError:
        print("ValueError caught")
test()
"#;
    let output = run_viper_code(code).expect("typed except should work");
    assert!(output.contains("ValueError caught"), "got: {}", output);
}

#[test]
fn test_try_except_as_binding() {
    let code = r#"
def test():
    try:
        raise RuntimeError("something went wrong")
    except RuntimeError as e:
        print("caught")
test()
"#;
    let output = run_viper_code(code).expect("except as binding should work");
    assert!(output.contains("caught"), "got: {}", output);
}

#[test]
fn test_try_multiple_except() {
    let code = r#"
def test():
    try:
        raise TypeError("wrong type")
    except ValueError:
        print("value error")
    except TypeError:
        print("type error")
    except:
        print("other error")
test()
"#;
    let output = run_viper_code(code).expect("multiple except clauses should work");
    assert!(output.contains("type error"), "got: {}", output);
}

// ============================================================================
// try/else
// ============================================================================

#[test]
fn test_try_else_no_exception() {
    let code = r#"
def test():
    try:
        x = 1 + 1
    except:
        print("error")
    else:
        print("success")
test()
"#;
    let output = run_viper_code(code).expect("try/else should work");
    assert!(output.contains("success"), "got: {}", output);
    assert!(!output.contains("error"), "got: {}", output);
}

#[test]
fn test_try_else_skipped_on_exception() {
    let code = r#"
def test():
    try:
        raise ValueError("err")
    except:
        print("error")
    else:
        print("success")
test()
"#;
    let output = run_viper_code(code).expect("try/else skip should work");
    assert!(output.contains("error"), "got: {}", output);
    assert!(!output.contains("success"), "got: {}", output);
}

// ============================================================================
// try/finally
// ============================================================================

#[test]
fn test_try_finally_no_exception() {
    let code = r#"
def test():
    try:
        print("try")
    finally:
        print("finally")
test()
"#;
    let output = run_viper_code(code).expect("try/finally should work");
    assert!(output.contains("try"), "got: {}", output);
    assert!(output.contains("finally"), "got: {}", output);
}

#[test]
fn test_try_finally_with_exception() {
    let code = r#"
def test():
    try:
        try:
            raise ValueError("err")
        finally:
            print("finally")
    except:
        print("caught")
test()
"#;
    let output = run_viper_code(code).expect("try/finally with exception should work");
    assert!(output.contains("finally"), "got: {}", output);
    assert!(output.contains("caught"), "got: {}", output);
}

#[test]
fn test_try_except_finally_all() {
    let code = r#"
def test():
    try:
        raise RuntimeError("err")
    except RuntimeError:
        print("except")
    finally:
        print("finally")
test()
"#;
    let output = run_viper_code(code).expect("try/except/finally should work");
    assert!(output.contains("except"), "got: {}", output);
    assert!(output.contains("finally"), "got: {}", output);
}

// ============================================================================
// raise Statement
// ============================================================================

#[test]
fn test_raise_simple() {
    let code = r#"
def test():
    try:
        raise ValueError("test error")
    except ValueError:
        print("ok")
test()
"#;
    let output = run_viper_code(code).expect("raise should work");
    assert!(output.contains("ok"), "got: {}", output);
}

#[test]
fn test_raise_in_function() {
    let code = r#"
def validate(x):
    if x < 0:
        raise ValueError("must be non-negative")
    return x

def test():
    try:
        validate(-1)
    except ValueError:
        print("validation failed")
test()
"#;
    let output = run_viper_code(code).expect("raise in function should work");
    assert!(output.contains("validation failed"), "got: {}", output);
}

#[test]
fn test_raise_propagation() {
    let code = r#"
def inner():
    raise RuntimeError("deep error")

def middle():
    inner()

def test():
    try:
        middle()
    except RuntimeError:
        print("propagated")
test()
"#;
    let output = run_viper_code(code).expect("exception propagation should work");
    assert!(output.contains("propagated"), "got: {}", output);
}

#[test]
fn test_raise_reraise() {
    let code = r#"
def test():
    try:
        try:
            raise ValueError("original")
        except ValueError:
            raise
    except ValueError:
        print("reraised")
test()
"#;
    let output = run_viper_code(code).expect("reraise should work");
    assert!(output.contains("reraised"), "got: {}", output);
}

// ============================================================================
// Nested try/except
// ============================================================================

#[test]
fn test_nested_try_except() {
    let code = r#"
def test():
    try:
        try:
            raise ValueError("inner")
        except ValueError:
            print("inner caught")
            raise RuntimeError("outer")
    except RuntimeError:
        print("outer caught")
test()
"#;
    let output = run_viper_code(code).expect("nested try/except should work");
    assert!(output.contains("inner caught"), "got: {}", output);
    assert!(output.contains("outer caught"), "got: {}", output);
}

#[test]
fn test_try_in_loop() {
    let code = r#"
def test():
    results = []
    for i in range(5):
        try:
            if i == 2:
                raise ValueError("skip")
            results.append(i)
        except ValueError:
            pass
    print(len(results))
test()
"#;
    let output = run_viper_code(code).expect("try in loop should work");
    assert!(output.contains("4"), "got: {}", output);
}
