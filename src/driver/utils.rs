use std::path::Path;

/// Check basic prerequisites (LLVM, GCC)
pub fn check_basic_prerequisites() -> Result<(), String> {
    // Check GCC for AOT linking
    if !check_command_exists("gcc") {
        return Err("GCC compiler not found in PATH".to_string());
    }

    Ok(())
}

/// Check runtime library exists (only needed for AOT compilation)
pub fn check_runtime_library() -> Result<(), String> {
    let runtime_paths = [
        "runtime/libviper.a",
        "../runtime/libviper.a",
        "../../runtime/libviper.a",
        "/usr/local/lib/viper/libviper.a",
        "/usr/lib/viper/libviper.a",
        "/opt/viper/lib/libviper.a",
        "$HOME/.local/lib/viper/libviper.a",
    ];

    // Expand environment variables in paths
    let expanded_paths: Vec<String> = runtime_paths
        .iter()
        .map(|p| {
            if p.starts_with("$HOME") {
                if let Ok(home) = std::env::var("HOME") {
                    p.replacen("$HOME", &home, 1)
                } else {
                    p.to_string()
                }
            } else {
                p.to_string()
            }
        })
        .collect();

    let runtime_found = expanded_paths.iter().any(|p| Path::new(p).exists());

    if !runtime_found {
        return Err("Viper runtime library not found (runtime/libviper.a)".to_string());
    }

    Ok(())
}

/// Check if a command exists in PATH
pub fn check_command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Show compiler information
pub fn show_info() {
    println!("Viper Compiler {}", env!("CARGO_PKG_VERSION"));
    println!("====================");
    println!("LLVM-based compiler for the Viper programming language");
    println!();
    println!("Features:");
    println!("  - AOT compilation to native binaries");
    println!("  - JIT execution for rapid development");
    println!("  - Python-like syntax");
    println!("  - Static typing with type inference");
    println!("  - List and dictionary data structures");
    println!("  - Math builtins (sqrt, abs, ln, floor)");
    println!();
    println!("Usage:");
    println!("  viper build <file.vp>     Compile to native binary");
    println!("  viper run <file.vp>       JIT compile and execute");
    println!("  viper init <project>      Create new project");
    println!("  viper info                Show this information");
}
#[allow(dead_code)]
pub fn get_opt_level(args: &[String]) -> u32 {
    for (i, arg) in args.iter().enumerate() {
        if arg == "-O" || arg == "--opt" {
            if let Some(level) = args.get(i + 1) {
                return level.parse().unwrap_or(0);
            }
        }
        if arg.starts_with("-O") && arg.len() > 2 {
            return arg[2..].parse().unwrap_or(0);
        }
    }
    0
}
