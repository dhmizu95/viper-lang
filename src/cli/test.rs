//! Test runner for Viper language
//! 
//! Provides a Python-compatible test runner with test discovery,
//! execution, and reporting.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Test case representation
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub source: PathBuf,
    pub expected_output: Option<String>,
    pub expected_exit_code: i32,
}

/// Test result representation
#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
    pub duration_ms: u64,
}

/// Test runner
pub struct TestRunner {
    tests: Vec<TestCase>,
    results: Vec<TestResult>,
    passed: usize,
    failed: usize,
    verbose: bool,
}

impl TestRunner {
    pub fn new(verbose: bool) -> Self {
        Self {
            tests: Vec::new(),
            results: Vec::new(),
            passed: 0,
            failed: 0,
            verbose,
        }
    }

    /// Discover tests in a directory
    pub fn discover(&mut self, path: &str) -> Result<(), String> {
        let path = Path::new(path);
        
        if path.is_file() {
            // Single test file
            if path.extension().map_or(false, |ext| ext == "vp") {
                self.tests.push(TestCase {
                    name: path.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    source: path.to_path_buf(),
                    expected_output: None,
                    expected_exit_code: 0,
                });
            }
        } else if path.is_dir() {
            // Discover all .vp files in directory
            self.discover_recursive(path)?;
        } else {
            return Err(format!("Path does not exist: {}", path.display()));
        }

        Ok(())
    }

    fn discover_recursive(&mut self, dir: &Path) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                // Skip certain directories
                let dir_name = path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                
                if dir_name == "target" || dir_name == "vendor" || dir_name.starts_with('.') {
                    continue;
                }

                self.discover_recursive(&path)?;
            } else if path.extension().map_or(false, |ext| ext == "vp") {
                // Check if it's a test file
                let file_name = path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                if file_name.starts_with("test_") || dir_name_contains_test(&path) {
                    self.tests.push(TestCase {
                        name: file_name,
                        source: path,
                        expected_output: None,
                        expected_exit_code: 0,
                    });
                }
            }
        }

        Ok(())
    }

    /// Run all discovered tests
    pub fn run(&mut self) -> Result<(), String> {
        if self.tests.is_empty() {
            println!("No tests found");
            return Ok(());
        }

        println!("Running {} tests...\n", self.tests.len());

        for test in &self.tests {
            let result = self.run_test(test)?;
            
            if result.passed {
                self.passed += 1;
                if self.verbose {
                    println!("✓ {} passed", result.name);
                }
            } else {
                self.failed += 1;
                println!("✗ {} failed", result.name);
                if let Some(msg) = &result.message {
                    println!("  {}", msg);
                }
            }

            self.results.push(result);
        }

        Ok(())
    }

    fn run_test(&self, test: &TestCase) -> Result<TestResult, String> {
        let start = std::time::Instant::now();

        // Build the test file
        let output = Command::new("cargo")
            .args(["run", "--", "run"])
            .arg(&test.source)
            .output()
            .map_err(|e| format!("Failed to execute test: {}", e))?;

        let duration = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let passed = exit_code == test.expected_exit_code;
        let message = if !passed {
            Some(format!(
                "Exit code: {}, expected: {}\nstdout: {}\nstderr: {}",
                exit_code, test.expected_exit_code, stdout, stderr
            ))
        } else {
            None
        };

        Ok(TestResult {
            name: test.name.clone(),
            passed,
            message,
            duration_ms: duration,
        })
    }

    /// Print test summary
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Test Summary");
        println!("{}", "=".repeat(60));
        println!("Total:  {}", self.tests.len());
        println!("Passed: {}", self.passed);
        println!("Failed: {}", self.failed);

        if self.failed > 0 {
            println!("\nFailed tests:");
            for result in &self.results {
                if !result.passed {
                    println!("  - {}", result.name);
                }
            }
        }

        let success_rate = if self.tests.is_empty() {
            0.0
        } else {
            (self.passed as f64 / self.tests.len() as f64) * 100.0
        };

        println!("\nSuccess rate: {:.1}%", success_rate);
    }

    /// Check if all tests passed
    pub fn was_successful(&self) -> bool {
        self.failed == 0
    }
}

fn dir_name_contains_test(path: &Path) -> bool {
    path.components().any(|c| {
        c.as_os_str()
            .to_string_lossy()
            .contains("test")
    })
}

/// Run test command
pub fn run_test_command(input: &str, discover: bool, verbose: bool, filter: Option<&str>) -> Result<(), String> {
    let mut runner = TestRunner::new(verbose);

    if discover {
        runner.discover(input)?;
    } else {
        // Single file or explicit path
        runner.discover(input)?;
    }

    // Apply filter if specified
    if let Some(pattern) = filter {
        runner.tests.retain(|t| t.name.contains(pattern));
    }

    runner.run()?;
    runner.print_summary();

    if !runner.was_successful() {
        std::process::exit(1);
    }

    Ok(())
}
