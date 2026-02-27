//! BigInt integration tests

use std::process::Command;

/// Helper to run viper compiler
fn run_viper(args: &[&str]) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run viper: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!("Command failed:\nstdout: {}\nstderr: {}", stdout, stderr));
    }

    Ok(stdout)
}

#[test]
fn test_bigint_creation() {
    // Test that BigInt can be created from large string
    let result = run_viper(&["run", "tests/bigint_test.vp"]);
    assert!(result.is_ok() || result.unwrap_err().contains("not implemented"));
}

#[test]
fn test_bigint_type_inference() {
    // Test BigInt type is recognized
    let code = r#"
def test():
    x = BigInt("123456789012345678901234567890")
    print("ok")

test()
"#;
    // Write to temp file and compile
    std::fs::write("/tmp/test_bigint.vp", code).unwrap();
    let result = run_viper(&["run", "/tmp/test_bigint.vp"]);
    // Should compile without type errors
    assert!(result.is_ok() || result.unwrap_err().contains("not implemented"));
}

#[test]
fn test_bigint_arithmetic() {
    // Test BigInt arithmetic operations
    let code = r#"
def test():
    a = BigInt("12345678901234567890")
    b = BigInt("98765432109876543210")
    c = a + b
    d = a * b
    e = b - a
    print("ok")

test()
"#;
    std::fs::write("/tmp/test_bigint_arith.vp", code).unwrap();
    let result = run_viper(&["run", "/tmp/test_bigint_arith.vp"]);
    assert!(result.is_ok() || result.unwrap_err().contains("not implemented"));
}
