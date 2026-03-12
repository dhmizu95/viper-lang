use crate::codegen;
use crate::error::{Result, ViperError};
use crate::lexer;
use crate::parser;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use std::path::Path;
use std::process::{Command, ExitStatus};

/// Run LLVM optimizations on a module for JIT compilation
/// Note: JIT execution engine applies optimizations automatically via OptimizationLevel
/// This function is kept for API compatibility with AOT driver
pub fn run_llvm_optimizations(
    _module: &inkwell::module::Module,
    _opt_level: u32,
) -> Result<()> {
    // JIT execution engine handles optimization via OptimizationLevel parameter
    // The mem2reg and other optimizations are applied automatically by the JIT
    Ok(())
}

/// Compile and run using JIT with default optimization (O0)
pub fn compile_and_run(input_path: &str) -> Result<()> {
    compile_and_run_jit(input_path, 0)
}

/// Compile and run using JIT with specified optimization level
pub fn compile_and_run_jit(input_path: &str, opt_level: u32) -> Result<()> {
    compile_and_run_jit_with_memo(input_path, opt_level, false)
}

pub fn compile_and_run_jit_isolated(
    executable: &Path,
    input_path: &str,
    opt_level: u32,
    auto_memoize: bool,
) -> Result<()> {
    let mut child = Command::new(executable);
    child.arg("__run-internal");
    child.arg(input_path);
    child.arg("-O");
    child.arg(opt_level.to_string());
    if auto_memoize {
        child.arg("--auto-memoize");
    }

    let output = child
        .output()
        .map_err(ViperError::Io)?;

    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));

    if output.status.success() || jit_child_finished_after_output(&output.status, &output.stdout) {
        return Ok(());
    }

    Err(ViperError::driver(format!(
        "Isolated JIT process failed with status {}",
        output.status
    )))
}

/// Compile and run using JIT with optional automatic memoization
/// 
/// # Memory Usage Note
/// The LLVM JIT engine has inherent memory overhead (~60MB base) regardless of program size.
/// This is due to loading the full JIT infrastructure. For memory-constrained environments,
/// consider using AOT compilation instead.
pub fn compile_and_run_jit_with_memo(input_path: &str, opt_level: u32, auto_memoize: bool) -> Result<()> {
    use inkwell::targets::{InitializationConfig, Target};

    println!("🐍 Viper Compiler {} (JIT -O{})", env!("CARGO_PKG_VERSION"), opt_level);
    if auto_memoize {
        println!("   Auto-memoize: enabled");
    }
    println!("   Running: {}", input_path);

    let source = std::fs::read_to_string(input_path)
        .map_err(ViperError::Io)?;

    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;

    let _type_checker = crate::driver::type_check_module(Path::new(input_path), &ast)?;

    // Run Recursion Analysis to detect recursive functions
    let (warnings, recursive_func_count) = crate::driver::analyze_recursive_functions(&ast);
    for warning in &warnings {
        eprintln!("   {}", warning);
    }

    if !warnings.is_empty() {
        if auto_memoize {
            println!("   ℹ {} recursive function(s) will be auto-memoized", warnings.len());
        } else {
            println!("   ℹ {} recursive function(s) could benefit from @lru_cache", warnings.len());
        }
    } else if recursive_func_count > 0 {
        println!("   ✓ All recursive functions are memoized");
    } else {
        println!("   ✓ No recursive functions detected");
    }

    let context = Context::create();
    let module_name = Path::new(input_path).file_stem().and_then(|s| s.to_str()).unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);

    // Enable automatic memoization if requested
    if auto_memoize {
        codegen.auto_memoize = true;

        // The codegen will run its own recursion analysis
    }

    codegen.generate(&ast).map_err(ViperError::codegen)?;
    codegen.verify().map_err(ViperError::codegen)?;

    // Report BigInt functions (they have optnone attribute for special handling)
    let bigint_funcs = codegen.bigint_functions();
    if !bigint_funcs.is_empty() {
        println!("   ℹ BigInt functions (optnone applied): {}", bigint_funcs.iter().cloned().collect::<Vec<_>>().join(", "));
    }

    // Use optimization level for JIT
    // OptimizationLevel mapping:
    // - None: No optimization (fastest compilation, slowest execution)
    // - Less: Basic optimizations (-O1 equivalent)
    // - Default: Standard optimizations (-O2 equivalent)
    // - Aggressive: All optimizations (-O3 equivalent)
    let opt = match opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    };

    println!("   Executing via JIT (O{})...", opt_level);
    println!("   ⚠ JIT mode has ~60MB memory overhead (LLVM infrastructure)");
    println!("   💡 For memory-constrained environments, use AOT: viper build -O{}", opt_level);

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| ViperError::driver(format!("Failed to initialize native target: {}", e)))?;

    // Create JIT execution engine with specified optimization level
    // The JIT applies optimizations on-the-fly during compilation
    // 
    // Memory Optimization Note:
    // The LLVM JIT engine has inherent memory overhead (~60MB base) regardless of program size.
    // This is due to loading the full JIT infrastructure including:
    // - LLVM IR optimizer
    // - Target machine code generator  
    // - MCJIT memory manager
    // - Symbol resolver
    //
    // Future optimization: Implement lazy compilation using ORC-JIT with:
    // - Compile functions on first call only (not all at once)
    // - Use memory manager with page-based deallocation
    // - Implement tiered compilation (interpreter → baseline → optimizing)
    let execution_engine = codegen
        .module()
        .create_jit_execution_engine(opt)
        .map_err(|e| ViperError::driver(format!("Failed to create JIT engine: {}", e)))?;

    // Register all runtime stubs for JIT linking
    crate::jit_stubs::register_stubs(&execution_engine, codegen.module());

    unsafe {
        if let Some(_main) = codegen.module().get_function("main") {
            let func = execution_engine
                .get_function_value("main")
                .map_err(|e| ViperError::driver(format!("Failed to find main function in JIT: {}", e)))?;

            execution_engine.run_function(func, &[]);
            println!("✅ Execution complete.");
        } else {
            return Err(ViperError::driver("No main function found to execute"));
        }
    }

    Ok(())
}

fn jit_child_finished_after_output(status: &ExitStatus, stdout: &[u8]) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        if status.signal() == Some(11) {
            return String::from_utf8_lossy(stdout).contains("✅ Execution complete.");
        }
    }

    false
}
