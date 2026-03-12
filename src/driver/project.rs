/// Initialize a new Viper project
pub fn init_project(name: &str) -> crate::error::Result<()> {
    use std::fs;
    use std::path::PathBuf;
    
    // Create project directory structure
    let dirs = ["src", "tests", "benchmarks", "docs"];
    for dir in &dirs {
        let path = PathBuf::from(format!("{}/{}", name, dir));
        fs::create_dir_all(&path)
            .map_err(crate::error::ViperError::Io)?;
    }

    // Create vpm.toml
    let vpm_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["Your Name"]
description = "A Viper project"
license = "MIT"

[dependencies]
# Add dependencies with: viper vpm add <package>
# Example: requests = ">=1.0"
# Example: mylib = {{ git = "https://github.com/user/mylib" }}
# Example: local-lib = {{ path = "../local-lib" }}

[dev-dependencies]
# Test dependencies

[features]
# Optional features
# default = ["async"]
# async = []
"#,
        name
    );
    fs::write(format!("{}/vpm.toml", name), vpm_toml)
        .map_err(crate::error::ViperError::Io)?;

    // Create main.vp
    let main_vp = r#"# Viper Project

def main():
    print("Hello from Viper!")
"#;
    fs::write(format!("{}/src/main.vp", name), main_vp)
        .map_err(crate::error::ViperError::Io)?;

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
    fs::write(format!("{}/.gitignore", name), gitignore)
        .map_err(crate::error::ViperError::Io)?;

    println!("Created Viper project: {}", name);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  viper run src/main.vp");
    println!();
    println!("Package management:");
    println!("  viper vpm add <package>     # Add a dependency");
    println!("  viper vpm add pkg --git URL # Add a git dependency");
    println!("  viper vpm install           # Install dependencies");
    println!("  viper vpm list              # List packages");
    
    Ok(())
}
