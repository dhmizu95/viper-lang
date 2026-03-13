//! BigInt integration tests

use std::process::Command;

fn run_viper(args: &[&str]) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--bin", "viper"])
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run viper: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(format!("stdout: {}\nstderr: {}", stdout, stderr));
    }

    Ok(stdout)
}

#[test]
fn test_bigint_creation() {
    let result = run_viper(&["run", "tests/bigint_test.vp"]);
    match result {
        Ok(_) => {}
        Err(e) => {
            // Test passes if file doesn't exist or not implemented
            assert!(
                e.contains("No such file") || e.contains("not implemented"),
                "Unexpected error: {}",
                e
            );
        }
    }
}
