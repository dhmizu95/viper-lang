//! Import Statement Integration Tests
//! Covers: import, from-import, aliasing, standard library modules

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
// import math
// ============================================================================

#[test]
fn test_import_math_sqrt() {
    let code = r#"
import math

def test():
    result = math.sqrt(16.0)
    print(result)
test()
"#;
    let output = run_viper_code(code).expect("import math sqrt should work");
    assert!(output.contains("4"), "got: {}", output);
}

#[test]
fn test_import_math_floor() {
    let code = r#"
import math

def test():
    print(math.floor(3.7))
    print(math.ceil(3.1))
test()
"#;
    let output = run_viper_code(code).expect("math.floor/ceil should work");
    assert!(output.contains("3"), "got: {}", output);
    assert!(output.contains("4"), "got: {}", output);
}

#[test]
fn test_import_math_pi() {
    let code = r#"
import math

def test():
    print(math.pi > 3.14)
    print(math.pi < 3.15)
test()
"#;
    let output = run_viper_code(code).expect("math.pi should work");
    let true_count = output.matches("True").count();
    assert!(true_count >= 2, "got: {}", output);
}

#[test]
fn test_import_math_abs() {
    let code = r#"
import math

def test():
    print(abs(-5))
    print(abs(3))
test()
"#;
    let output = run_viper_code(code).expect("abs should work");
    assert!(output.contains("5"), "got: {}", output);
    assert!(output.contains("3"), "got: {}", output);
}

// ============================================================================
// from math import ...
// ============================================================================

#[test]
fn test_from_import_sqrt() {
    let code = r#"
from math import sqrt

def test():
    result = sqrt(25.0)
    print(result)
test()
"#;
    let output = run_viper_code(code).expect("from math import sqrt should work");
    assert!(output.contains("5"), "got: {}", output);
}

#[test]
fn test_from_import_multiple() {
    let code = r#"
from math import floor, ceil

def test():
    print(floor(2.9))
    print(ceil(2.1))
test()
"#;
    let output = run_viper_code(code).expect("from math import multiple should work");
    assert!(output.contains("2"), "got: {}", output);
    assert!(output.contains("3"), "got: {}", output);
}

// ============================================================================
// import ... as alias
// ============================================================================

#[test]
fn test_import_alias() {
    let code = r#"
import math as m

def test():
    print(m.sqrt(9.0))
test()
"#;
    let output = run_viper_code(code).expect("import alias should work");
    assert!(output.contains("3"), "got: {}", output);
}

#[test]
fn test_from_import_alias() {
    let code = r#"
from math import sqrt as sq

def test():
    print(sq(36.0))
test()
"#;
    let output = run_viper_code(code).expect("from import alias should work");
    assert!(output.contains("6"), "got: {}", output);
}

// ============================================================================
// Other Standard Library Modules
// ============================================================================

#[test]
fn test_import_sys() {
    let code = r#"
import sys

def test():
    # sys module exists
    print("ok")
test()
"#;
    let output = run_viper_code(code).expect("import sys should work");
    assert!(output.contains("ok"), "got: {}", output);
}

#[test]
fn test_import_os_path() {
    let code = r#"
import os

def test():
    # os module exists
    print("ok")
test()
"#;
    let output = run_viper_code(code).expect("import os should work");
    assert!(output.contains("ok"), "got: {}", output);
}

// ============================================================================
// Import at module level (top-level usage)
// ============================================================================

#[test]
fn test_import_used_directly() {
    let code = r#"
import math

result = math.sqrt(100.0)
print(result)
"#;
    let output = run_viper_code(code).expect("top-level import usage should work");
    assert!(output.contains("10"), "got: {}", output);
}

#[test]
fn test_from_import_used_directly() {
    let code = r#"
from math import pi

print(pi > 3)
"#;
    let output = run_viper_code(code).expect("top-level from import should work");
    assert!(output.contains("True"), "got: {}", output);
}
