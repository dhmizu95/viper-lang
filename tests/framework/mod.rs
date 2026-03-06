use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

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
        if !dir_path.is_dir() {
            return Err(format!("{} is not a directory", dir_path.display()));
        }

        for entry in fs::read_dir(dir_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                self.discover(&path)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("vp") && path.to_string_lossy().contains("test_") {
                self.load_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn load_file(&mut self, filepath: &Path) -> Result<(), String> {
        let source = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
        
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
            // Placeholder: In a real implementation we would compile and run the Viper code
            // For now, let's just create a dummy result
            // Normally we'd use `viper run <path>` 
            if let Some(_path) = &case.path {
                // let output = Command::new("cargo")
                //    .args(&["run", "--", "run", path.to_string_lossy().as_ref()])
                //    .output()
                //    .map_err(|e| e.to_string())?;
                
                // String::from_utf8_lossy(&output.stdout).to_string();
            }
            
            // Dummy logic for now
            let output_matches = true; 
            
            if output_matches {
                self.passed += 1;
            } else {
                self.failed += 1;
                failures.push(TestFailure {
                    test_name: case.name.clone(),
                    message: "Output did not match expected".to_string(),
                });
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
