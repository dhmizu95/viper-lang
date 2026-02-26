use crate::lexer;
use crate::parser;
use crate::codegen;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use std::path::Path;

pub fn run_llvm_optimizations(
    _module: &inkwell::module::Module,
    _opt_level: u32,
) -> Result<(), String> {
    // JIT execution engine handles optimization via OptimizationLevel parameter
    // The mem2reg and other optimizations are applied automatically by the JIT
    Ok(())
}

pub fn compile_and_run(input_path: &str) -> Result<(), String> {
    compile_and_run_jit(input_path, 0)
}

pub fn compile_and_run_jit(input_path: &str, opt_level: u32) -> Result<(), String> {
    use inkwell::targets::{InitializationConfig, Target};

    println!(
        "🐍 Viper Compiler {} (JIT -O{})",
        env!("CARGO_PKG_VERSION"),
        opt_level
    );
    println!("   Running: {}", input_path);

    let source = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{}': {}", input_path, e))?;

    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;

    // Semantic Analysis (Type Checking)
    let mut type_checker = crate::semantic::type_checker::TypeChecker::new();
    type_checker.check(&ast).map_err(|e| {
        format!("Type errors found:\n{}", e.iter().map(|err| format!(" - {}", err)).collect::<Vec<_>>().join("\n"))
    })?;

    let context = Context::create();
    let module_name = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);
    codegen.generate(&ast)?;
    codegen.verify()?;

    // Use optimization level for JIT
    let opt = match opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    };

    println!("   Executing via JIT (O{})...", opt_level);

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("Failed to initialize native target: {}", e))?;

    let execution_engine = codegen
        .module()
        .create_jit_execution_engine(opt)
        .map_err(|e| format!("Failed to create JIT engine: {}", e))?;

    // Register all runtime stubs
    crate::jit_stubs::register_stubs(&execution_engine, codegen.module());

    unsafe {
        if let Some(_main) = codegen.module().get_function("main") {
            let func = execution_engine
                .get_function_value("main")
                .map_err(|e| format!("Failed to find main function in JIT: {}", e))?;

            execution_engine.run_function(func, &[]);
            println!("✅ Execution complete.");
        } else {
            return Err("No main function found to execute".to_string());
        }
    }

    Ok(())
}
