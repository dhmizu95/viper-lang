/// Initialize a new Viper project
pub fn init_project(name: &str) -> Result<(), String> {
    // Create project directory
    std::fs::create_dir_all(format!("{}/src", name))
        .map_err(|e| format!("Failed to create project directory: {}", e))?;

    // Create main.vp
    let main_vp = r#"# Viper Project

def main():
    print("Hello from Viper!")

"#;
    std::fs::write(format!("{}/src/main.vp", name), main_vp)
        .map_err(|e| format!("Failed to create main.vp: {}", e))?;

    // Create Cargo.toml for the project
    let cargo_toml = r#"[package]
name = "PROJECT_NAME"
version = "0.1.0"
edition = "2021"

[dependencies]
"#
    .replace("PROJECT_NAME", name);
    std::fs::write(format!("{}/Cargo.toml", name), cargo_toml)
        .map_err(|e| format!("Failed to create Cargo.toml: {}", e))?;

    println!("Created Viper project: {}", name);
    println!("   cd {} && viper run src/main.vp", name);
    Ok(())
}
