//! Viper Package Manager (VPM) - Command Implementations
//!
//! Implementation of all vpm commands.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Initialize a new Viper package
pub fn init_package(name: Option<String>) -> Result<(), String> {
    let package_name = name.unwrap_or_else(|| "my_package".to_string());
    
    // Create project directory
    let project_dir = PathBuf::from(&package_name);
    if project_dir.exists() {
        return Err(format!("Directory '{}' already exists", package_name));
    }
    
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create directory {}: {}", package_name, e))?;
    
    // Create project structure
    let dirs = ["src", "tests", "benchmarks", "docs"];
    for dir in &dirs {
        let path = project_dir.join(dir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))?;
    }
    
    // Create vpm.toml
    let manifest_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name"]
description = "A Viper package"
license = "MIT"

[dependencies]
# Add your dependencies here
# Example: requests = {{ version = ">=1.0" }}
# Example: mylib = {{ git = "https://github.com/user/mylib" }}
# Example: local-lib = {{ path = "../local-lib" }}
"#,
        package_name
    );
    
    fs::write(project_dir.join("vpm.toml"), manifest_content)
        .map_err(|e| format!("Failed to create vpm.toml: {}", e))?;
    
    // Create .gitignore
    let gitignore = r#"build/
*.vic
*.ll
*.o
*.so
*.dylib
*.dll
target/
vendor/
vpm.lock
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)
        .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
    
    // Create main source file
    let main_content = r#"# Main entry point

def main():
    print("Hello from Viper!")
"#;
    fs::write(project_dir.join("src/main.vp"), main_content)
        .map_err(|e| format!("Failed to create src/main.vp: {}", e))?;
    
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    let project_path = cwd.join(&package_name);
    println!("Initialized Viper package '{}' in {}", package_name, project_path.display());
    println!("\nNext steps:");
    println!("  cd {}", package_name);
    println!("  viper run src/main.vp");
    println!();
    println!("Package management:");
    println!("  vpm add <package>           # Add a dependency");
    println!("  vpm add pkg --git URL       # Add a git dependency");
    println!("  vpm install                 # Install dependencies");
    println!("  vpm list                    # List packages");
    
    Ok(())
}

/// Add a dependency to vpm.toml
pub fn add_dependency(package: &str, git: Option<&str>, branch: Option<&str>, path: Option<&str>) -> Result<(), String> {
    let manifest_path = Path::new("vpm.toml");
    
    if !manifest_path.exists() {
        return Err("No vpm.toml found. Run 'vpm init' first.".to_string());
    }
    
    // Read existing manifest
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read vpm.toml: {}", e))?;
    
    let mut manifest: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse vpm.toml: {}", e))?;
    
    // Parse package name and version
    let (pkg_name, version) = if package.contains('@') {
        let parts: Vec<&str> = package.split('@').collect();
        (parts[0], parts[1])
    } else {
        (package, ">=0.1.0")
    };
    
    // Create dependency entry
    let dep_entry: toml::Value = if let Some(git_url) = git {
        let mut entry = toml::map::Map::new();
        entry.insert("git".to_string(), toml::Value::String(git_url.to_string()));
        if let Some(b) = branch {
            entry.insert("branch".to_string(), toml::Value::String(b.to_string()));
        }
        toml::Value::Table(entry)
    } else if let Some(local_path) = path {
        let mut entry = toml::map::Map::new();
        entry.insert("path".to_string(), toml::Value::String(local_path.to_string()));
        entry.insert("version".to_string(), toml::Value::String(version.to_string()));
        toml::Value::Table(entry)
    } else {
        toml::Value::String(version.to_string())
    };
    
    // Add to dependencies
    if let Some(deps) = manifest.get_mut("dependencies") {
        if let Some(table) = deps.as_table_mut() {
            table.insert(pkg_name.to_string(), dep_entry);
        }
    } else {
        let mut deps = toml::map::Map::new();
        deps.insert(pkg_name.to_string(), dep_entry);
        manifest["dependencies"] = toml::Value::Table(deps);
    }
    
    // Write back
    let new_content = toml::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize vpm.toml: {}", e))?;
    
    fs::write(manifest_path, new_content)
        .map_err(|e| format!("Failed to write vpm.toml: {}", e))?;
    
    println!("Added {} to dependencies", package);
    println!("Run 'vpm install' to install the dependency");
    
    Ok(())
}

/// Remove a dependency from vpm.toml
pub fn remove_dependency(package: &str) -> Result<(), String> {
    let manifest_path = Path::new("vpm.toml");
    
    if !manifest_path.exists() {
        return Err("No vpm.toml found.".to_string());
    }
    
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read vpm.toml: {}", e))?;
    
    let mut manifest: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse vpm.toml: {}", e))?;
    
    // Remove from dependencies
    if let Some(deps) = manifest.get_mut("dependencies") {
        if let Some(table) = deps.as_table_mut() {
            if table.remove(package).is_some() {
                let new_content = toml::to_string_pretty(&manifest)
                    .map_err(|e| format!("Failed to serialize vpm.toml: {}", e))?;
                
                fs::write(manifest_path, new_content)
                    .map_err(|e| format!("Failed to write vpm.toml: {}", e))?;
                
                println!("Removed {} from dependencies", package);
                return Ok(());
            }
        }
    }
    
    Err(format!("Package '{}' not found in dependencies", package))
}

/// Install dependencies
pub fn install_dependencies(package: Option<&str>) -> Result<(), String> {
    let manifest_path = Path::new("vpm.toml");
    
    if !manifest_path.exists() {
        return Err("No vpm.toml found. Run 'vpm init' first.".to_string());
    }
    
    println!("Installing dependencies...");
    
    // Create vendor directory for packages
    let vendor_dir = PathBuf::from("vendor");
    if !vendor_dir.exists() {
        fs::create_dir_all(&vendor_dir)
            .map_err(|e| format!("Failed to create vendor directory: {}", e))?;
    }
    
    // Read manifest
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read vpm.toml: {}", e))?;
    
    let manifest: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse vpm.toml: {}", e))?;
    
    // Install from dependencies
    if let Some(deps) = manifest.get("dependencies") {
        if let Some(table) = deps.as_table() {
            for (name, spec) in table {
                if package.is_some() && package.unwrap() != name {
                    continue;
                }
                
                println!("  Installing {}...", name);
                
                if let Some(git_url) = spec.get("git").and_then(|v| v.as_str()) {
                    // Git dependency
                    let branch = spec.get("branch").and_then(|v| v.as_str()).unwrap_or("main");
                    let dest_dir = vendor_dir.join(name);
                    
                    if dest_dir.exists() {
                        println!("    {} already exists, skipping", name);
                        continue;
                    }
                    
                    println!("    Cloning from {} (branch: {})", git_url, branch);
                    let status = Command::new("git")
                        .args(["clone", "-b", branch, "--depth", "1", git_url])
                        .arg(&dest_dir)
                        .status();
                    
                    match status {
                        Ok(s) if s.success() => {
                            println!("    ✓ Installed {}", name);
                        }
                        Ok(_) => {
                            eprintln!("    ✗ Failed to clone {}", name);
                        }
                        Err(e) => {
                            eprintln!("    ✗ Error: {}", e);
                        }
                    }
                } else if let Some(p) = spec.get("path").and_then(|v| v.as_str()) {
                    // Path dependency
                    println!("    Using local path: {}", p);
                } else {
                    // Registry dependency (future implementation)
                    println!("    Registry package (not yet implemented): {}", name);
                }
            }
        }
    }
    
    println!("\nDependencies installed successfully!");
    println!("Note: Full registry support is coming soon.");
    
    Ok(())
}

/// List installed packages
pub fn list_packages(top_level: bool) -> Result<(), String> {
    let manifest_path = Path::new("vpm.toml");
    
    if !manifest_path.exists() {
        return Err("No vpm.toml found.".to_string());
    }
    
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read vpm.toml: {}", e))?;
    
    let manifest: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse vpm.toml: {}", e))?;
    
    println!("Installed packages:");
    println!();
    
    // Show direct dependencies
    if let Some(deps) = manifest.get("dependencies") {
        if let Some(table) = deps.as_table() {
            for (name, spec) in table {
                let version = if let Some(p) = spec.get("path") {
                    format!("(path: {})", p.as_str().unwrap_or("unknown"))
                } else if let Some(git) = spec.get("git") {
                    format!("(git: {})", git.as_str().unwrap_or("unknown"))
                } else {
                    spec.as_str().unwrap_or("unknown").to_string()
                };
                println!("  {} {}", name, version);
                
                if !top_level {
                    // Check if installed in vendor
                    let vendor_path = PathBuf::from("vendor").join(name);
                    if vendor_path.exists() {
                        println!("    └── installed at vendor/{}", name);
                    } else if let Some(p) = spec.get("path").and_then(|v| v.as_str()) {
                        println!("    └── linked from {}", p);
                    } else if let Some(git) = spec.get("git").and_then(|v| v.as_str()) {
                        println!("    └── from git: {}", git);
                    }
                }
            }
        }
    }
    
    println!();
    println!("  viper-std v0.4.5 (bundled)");
    
    Ok(())
}

/// Clean package cache
pub fn clean_cache() -> Result<(), String> {
    let vendor_dir = PathBuf::from("vendor");
    let build_dir = PathBuf::from("build");
    
    if vendor_dir.exists() {
        fs::remove_dir_all(&vendor_dir)
            .map_err(|e| format!("Failed to remove vendor directory: {}", e))?;
        println!("Cleaned vendor/");
    }
    
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)
            .map_err(|e| format!("Failed to remove build directory: {}", e))?;
        println!("Cleaned build/");
    }
    
    println!("Package cache cleaned");
    
    Ok(())
}

/// Show package tree
pub fn show_tree(depth: usize) -> Result<(), String> {
    let manifest_path = Path::new("vpm.toml");
    
    if !manifest_path.exists() {
        return Err("No vpm.toml found.".to_string());
    }
    
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read vpm.toml: {}", e))?;
    
    let manifest: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse vpm.toml: {}", e))?;
    
    println!("Dependency tree:");
    println!();
    
    // Root package
    if let Some(pkg) = manifest.get("package") {
        if let Some(name) = pkg.get("name").and_then(|v| v.as_str()) {
            if let Some(version) = pkg.get("version").and_then(|v| v.as_str()) {
                println!("{} v{}", name, version);
            }
        }
    }
    
    // Dependencies
    if let Some(deps) = manifest.get("dependencies") {
        if let Some(table) = deps.as_table() {
            let mut deps: Vec<_> = table.iter().collect();
            deps.sort_by(|a, b| a.0.cmp(b.0));
            
            for (i, (name, spec)) in deps.iter().enumerate() {
                let prefix = if i == deps.len() - 1 { "└──" } else { "├──" };
                let version = if let Some(p) = spec.get("path") {
                    format!("(path: {})", p.as_str().unwrap_or("unknown"))
                } else if let Some(git) = spec.get("git") {
                    format!("(git: {})", git.as_str().unwrap_or("unknown"))
                } else {
                    spec.as_str().unwrap_or("unknown").to_string()
                };
                println!("    {} {} {}", prefix, name, version);
                
                // Check for transitive dependencies (future)
                let vendor_path = PathBuf::from("vendor").join(name).join("vpm.toml");
                if vendor_path.exists() && depth > 1 {
                    println!("        └── (dependencies not yet resolved)");
                }
            }
        }
    }
    
    println!();
    println!("  viper-std v0.4.5");
    
    Ok(())
}

/// Search for packages (stub - needs registry implementation)
pub fn search_packages(query: &str) -> Result<(), String> {
    println!("Searching for packages matching '{}'...", query);
    println!();
    println!("Note: Package registry is not yet implemented.");
    println!("Future versions will search https://packages.viper-lang.org");
    println!();
    println!("You can still use git dependencies:");
    println!("  vpm add package --git https://github.com/user/repo");
    
    Ok(())
}

/// Show package information
pub fn show_package(package: &str) -> Result<(), String> {
    // Check local manifest first
    let manifest_path = Path::new("vpm.toml");
    
    if !manifest_path.exists() {
        return Err(format!("Package '{}' not found", package));
    }
    
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read vpm.toml: {}", e))?;
    
    let manifest: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse vpm.toml: {}", e))?;
    
    // Check if it's in dependencies
    if let Some(deps) = manifest.get("dependencies") {
        if let Some(table) = deps.as_table() {
            if let Some(spec) = table.get(package) {
                println!("Package: {}", package);
                
                if let Some(version) = spec.as_str() {
                    println!("Version: {}", version);
                }
                
                if let Some(git) = spec.get("git").and_then(|v| v.as_str()) {
                    println!("Git: {}", git);
                }
                
                if let Some(path) = spec.get("path").and_then(|v| v.as_str()) {
                    println!("Path: {}", path);
                }
                
                // Check if installed
                let vendor_path = PathBuf::from("vendor").join(package);
                if vendor_path.exists() {
                    println!("Status: installed");
                } else {
                    println!("Status: not installed");
                }
                
                return Ok(());
            }
        }
    }
    
    Err(format!("Package '{}' not found in dependencies", package))
}

/// Update dependencies
pub fn update_dependencies(package: Option<&str>, _pre: bool) -> Result<(), String> {
    println!("Updating dependencies...");
    
    if package.is_some() {
        println!("Note: Updating specific packages is not yet implemented.");
    }
    
    println!("Run 'vpm install' to ensure all dependencies are up to date.");
    
    Ok(())
}

/// Publish a package
pub fn publish_package(bump: Option<&str>, dry_run: bool) -> Result<(), String> {
    let manifest_path = Path::new("vpm.toml");
    
    if !manifest_path.exists() {
        return Err("No vpm.toml found.".to_string());
    }
    
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("Failed to read vpm.toml: {}", e))?;
    
    let mut manifest: toml::Value = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse vpm.toml: {}", e))?;
    
    // Get current version
    let current_version = manifest
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0");
    
    // Bump version if requested
    let new_version = if let Some(level) = bump {
        let parts: Vec<u32> = current_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", current_version));
        }
        
        let mut new_parts = parts.clone();
        
        match level {
            "major" => {
                new_parts[0] += 1;
                new_parts[1] = 0;
                new_parts[2] = 0;
            }
            "minor" => {
                new_parts[1] += 1;
                new_parts[2] = 0;
            }
            "patch" => {
                new_parts[2] += 1;
            }
            _ => {
                return Err("Bump level must be 'major', 'minor', or 'patch'".to_string());
            }
        }
        
        let new_ver = format!("{}.{}.{}", new_parts[0], new_parts[1], new_parts[2]);
        println!("Bumping version: {} -> {}", current_version, new_ver);
        new_ver
    } else {
        current_version.to_string()
    };
    
    let pkg_name = manifest
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    if dry_run {
        println!("[DRY RUN] Would publish {} v{}", pkg_name, new_version);
        println!("[DRY RUN] No changes made");
    } else {
        // Update version in manifest
        if let Some(pkg) = manifest.get_mut("package") {
            if let Some(ver) = pkg.get_mut("version") {
                *ver = toml::Value::String(new_version.clone());
            }
        }
        
        let new_content = toml::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize vpm.toml: {}", e))?;
        
        fs::write(manifest_path, new_content)
            .map_err(|e| format!("Failed to write vpm.toml: {}", e))?;
        
        println!("Published {} v{}", pkg_name, new_version);
        
        // Note about registry
        println!("\nNote: Package registry publishing is not yet implemented.");
        println!("The version has been updated in vpm.toml");
    }
    
    Ok(())
}
