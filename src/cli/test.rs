use clap::Args;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

#[derive(Args, Debug)]
pub struct TestArgs {
    /// Discover tests in directory
    #[arg(short, long)]
    pub discover: bool,

    /// Path to test file or directory
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: String,
}

pub struct TestRunner {
    pub tests: Vec<TestCase>,
    pub passed: usize,
    pub failed: usize,
}

pub struct TestCase {
    pub name: String,
    pub source: String,
    pub expected_output: String,
    pub expected_exit_code: i32,
    pub path: Option<PathBuf>,
}

pub struct TestResult {
    pub tests_run: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<TestFailure>,
}

pub struct TestFailure {
    pub test_name: String,
    pub message: String,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            passed: 0,
            failed: 0,
        }
    }

    pub fn discover(&mut self, dir_path: &Path) -> Result<(), String> {
        if !dir_path.exists() {
            return Err(format!("{} does not exist", dir_path.display()));
        }

        if dir_path.is_file() {
            return self.load_file(dir_path);
        }

        if let Ok(entries) = fs::read_dir(dir_path) {
            let mut paths: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect();
            paths.sort();

            for path in paths {
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                    // Skip rust source dirs or similar if needed, or just recurse everywhere
                    if !dir_name.starts_with('.') && dir_name != "target" {
                        self.discover(&path)?;
                    }
                } else if path.extension().and_then(|e| e.to_str()) == Some("vp") && path.file_name().unwrap_or_default().to_string_lossy().starts_with("test_") {
                    self.load_file(&path)?;
                }
            }
        }
        Ok(())
    }

    pub fn load_file(&mut self, filepath: &Path) -> Result<(), String> {
        let source = match fs::read_to_string(filepath) {
            Ok(s) => s,
            Err(_) => return Ok(()), // skip unreadable
        };
        
        let file_stem = filepath.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string());
        
        let mut expected_output = String::new();
        let mut in_expected_output = false;
        
        for line in source.lines() {
            if line.contains("EXPECTED_OUTPUT:") {
                in_expected_output = true;
                continue;
            }
            if in_expected_output && line.starts_with("# ") {
                expected_output.push_str(&line[2..]);
                expected_output.push('\n');
            } else if in_expected_output {
                break;
            }
        }
        
        self.tests.push(TestCase {
            name: file_stem.clone(),
            source,
            expected_output,
            expected_exit_code: 0,
            path: Some(filepath.to_path_buf()),
        });
        
        Ok(())
    }

    pub fn run(&mut self) -> Result<TestResult, String> {
        let mut failures = Vec::new();
        let tests_run = self.tests.len();
        
        for case in &self.tests {
            if let Some(path) = &case.path {
                let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
                
                let output = Command::new(current_exe)
                    .args(&["run", path.to_str().unwrap()])
                    .output()
                    .map_err(|e| e.to_string())?;
                
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                
                // Very basic check logic based on EXPECTED_OUTPUT comment
                if case.expected_output.is_empty() {
                    // For tests without expecting specific output, just check if it exits successfully
                    if output.status.success() {
                        self.passed += 1;
                        println!("  ✓ {}", case.name);
                    } else {
                        self.failed += 1;
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        println!("  ✗ {}", case.name);
                        failures.push(TestFailure {
                            test_name: case.name.clone(),
                            message: format!("Crash or Error:\\n{}", stderr),
                        });
                    }
                } else {
                    if stdout.trim() == case.expected_output.trim() && output.status.success() {
                        self.passed += 1;
                        println!("  ✓ {}", case.name);
                    } else {
                        self.failed += 1;
                        println!("  ✗ {}", case.name);
                        failures.push(TestFailure {
                            test_name: case.name.clone(),
                            message: format!("Expected output mismatch.\\nExpected:\\n{}\\nGot:\\n{}", case.expected_output, stdout),
                        });
                    }
                }
            }
        }
        
        Ok(TestResult {
            tests_run,
            passed: self.passed,
            failed: self.failed,
            failures,
        })
    }
}

pub fn run_test_command(args: &TestArgs) -> Result<(), String> {
    let mut runner = TestRunner::new();
    
    let path = Path::new(&args.path);
    
    if args.discover || path.is_dir() {
        runner.discover(path)?;
    } else {
        runner.load_file(path)?;
    }
    
    println!("Running {} tests...", runner.tests.len());
    let result = runner.run()?;
    
    println!("\\nRan {} tests", result.tests_run);
    println!("Passed: {}", result.passed);
    println!("Failed: {}", result.failed);
    
    if !result.failures.is_empty() {
        println!("\\nFailures:");
        for failure in &result.failures {
            println!("  - {}", failure.test_name);
            for line in failure.message.lines() {
                println!("    {}", line);
            }
        }
        std::process::exit(1);
    }
    
    Ok(())
}
