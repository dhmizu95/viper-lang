//! Shared helpers for integration tests

use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run_viper_code_with_stderr(code: &str) -> Result<(String, String), String> {
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

    Ok((stdout, stderr))
}

pub fn run_viper_code(code: &str) -> Result<String, String> {
    run_viper_code_with_stderr(code).map(|(stdout, _)| stdout)
}

/// Run Viper code with JIT and --auto-memoize flag
pub fn run_viper_code_auto_memoize(code: &str) -> Result<(String, String), String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;

    let output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["run", "--auto-memoize"])
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let _ = fs::remove_file(&test_file);

    if !output.status.success() {
        return Err(format!("stdout: {}\nstderr: {}", stdout, stderr));
    }

    Ok((stdout, stderr))
}

/// Build Viper code with AOT and --auto-memoize flag, then run the binary
pub fn build_and_run_auto_memoize(code: &str) -> Result<String, String> {
    let temp_dir = env::temp_dir();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let test_file = temp_dir.join(format!("viper_test_{}.vp", timestamp));
    // AOT adds _bin suffix, so we use base name without suffix
    let output_base = temp_dir.join(format!("viper_test_{}", timestamp));
    let output_binary = format!("{}_bin", output_base.display());
    
    fs::write(&test_file, code).map_err(|e| format!("Failed to write: {}", e))?;

    // Build with auto-memoize
    let build_output = Command::new(env!("CARGO_BIN_EXE_viper"))
        .args(["build", "--auto-memoize", "-O0"])
        .arg(&test_file)
        .arg("-o")
        .arg(&output_base)
        .output()
        .map_err(|e| format!("Failed to build: {}", e))?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&build_output.stdout).to_string();
        return Err(format!("Build failed: {}\n{}", stdout, stderr));
    }

    // Run the binary (AOT adds _bin suffix)
    let run_output = Command::new(&output_binary)
        .output()
        .map_err(|e| format!("Failed to run: {}", e))?;

    let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();
    let _ = fs::remove_file(&test_file);
    let _ = fs::remove_file(&output_binary);
    let _ = fs::remove_file(format!("{}.o", output_base.display()));

    if !run_output.status.success() {
        return Err(format!("Run failed: {}\n{}", stdout, stderr));
    }

    Ok(stdout)
}
