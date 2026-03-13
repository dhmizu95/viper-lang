use crate::ast::{Module, Stmt};
use crate::error::{Result, ViperError};
use crate::semantic::{RecursionAnalyzer, TypeChecker};
use std::path::Path;

fn check_command_exists_any(commands: &[&str]) -> bool {
    commands.iter().any(|cmd| which::which(cmd).is_ok())
}

/// Check prerequisites for AOT compilation.
pub fn check_aot_prerequisites() -> Result<()> {
    if !check_command_exists_any(&[
        "opt",
        "opt-21",
        "opt-20",
        "opt-19",
        "opt-18",
        "opt-17",
        "opt-16",
        "opt-15",
        "opt-14",
    ]) {
        return Err(ViperError::driver(
            "LLVM opt tool not found in PATH (looked for opt and versioned opt-* binaries)",
        ));
    }

    if !check_command_exists("gcc") {
        return Err(ViperError::driver("GCC compiler not found in PATH"));
    }

    Ok(())
}

/// Check basic prerequisites for general compiler use.
pub fn check_basic_prerequisites() -> Result<()> {
    Ok(())
}

/// Check runtime library exists (only needed for AOT compilation)
pub fn check_runtime_library() -> Result<()> {
    let runtime_paths = [
        "runtime/obj/libviper.a",
        "../runtime/obj/libviper.a",
        "../../runtime/obj/libviper.a",
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
        return Err(ViperError::driver("Viper runtime library not found (runtime/obj/libviper.a)"));
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

pub fn analyze_recursive_functions(module: &Module) -> (Vec<String>, usize) {
    let mut recursion_analyzer = RecursionAnalyzer::new();

    for stmt in &module.statements {
        if let Stmt::Function { name, .. } = stmt {
            recursion_analyzer.register_function(name);
        }
    }

    for stmt in &module.statements {
        if let Stmt::Function { name, body, .. } = stmt {
            recursion_analyzer.analyze_function(name, body);
        }
    }

    recursion_analyzer.detect_mutual_recursion();

    let recursive_funcs = recursion_analyzer.get_recursive_functions();
    let mut warnings = Vec::new();

    for (name, _info) in recursive_funcs {
        if function_has_memoization_decorator(module, name) {
            continue;
        }

        if let Some(warning) = recursion_analyzer.generate_warning(name) {
            warnings.push(warning);
        }
    }

    (warnings, recursive_funcs.len())
}

pub fn type_check_module(input_path: &Path, ast: &Module) -> Result<TypeChecker> {
    let mut type_checker = TypeChecker::with_input_path(input_path);
    type_checker.check(ast).map_err(|e| {
        ViperError::type_error(
            format!(
                "Type errors found:\n{}",
                e.iter().map(|err| format!(" - {}", err)).collect::<Vec<_>>().join("\n")
            ),
            crate::utils::Span::default(),
        )
    })?;
    Ok(type_checker)
}

fn function_has_memoization_decorator(module: &Module, function_name: &str) -> bool {
    module.statements.iter().any(|stmt| {
        matches!(
            stmt,
            Stmt::Function {
                name,
                decorators,
                ..
            } if name == function_name
                && decorators
                    .iter()
                    .any(|d| d.name == "lru_cache" || d.name == "cache")
        )
    })
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
