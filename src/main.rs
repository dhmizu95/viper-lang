mod ast;
mod cli;
mod codegen;
mod lexer;
mod parser;
mod utils;

use inkwell::context::Context;
use inkwell::passes::PassManager;
use inkwell::targets::{
    CodeModel, InitializationConfig, RelocMode, Target, TargetMachine, TargetTriple,
};
use inkwell::OptimizationLevel;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Viper Compiler 0.2.2");
        eprintln!("Usage: viper <command> [options]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  build <file.vp>  Compile a Viper source file (AOT)");
        eprintln!("  run <file.vp>    Compile and run a Viper source file (JIT)");
        eprintln!("  run-opt <file.vp> Run with optimizations enabled");
        eprintln!("  help             Show this help message");
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "build" | "compile" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                eprintln!("Usage: viper build <file.vp>");
                std::process::exit(1);
            }
            let opt_level = get_opt_level(&args);
            let input_file = args.iter().find(|a| a.ends_with(".vp"))
                .ok_or("Error: No input file specified")
                .map(|s| s.as_str())
                .unwrap();
            if let Err(e) = compile_file_aot(input_file, opt_level, None) {
                eprintln!("Compilation failed: {}", e);
                std::process::exit(1);
            }
        }
        "build-opt" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                eprintln!("Usage: viper build-opt <file.vp>");
                std::process::exit(1);
            }
            let input_file = &args[2];
            if let Err(e) = compile_file_optimized(input_file) {
                eprintln!("Compilation failed: {}", e);
                std::process::exit(1);
            }
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                eprintln!("Usage: viper run <file.vp>");
                std::process::exit(1);
            }
            let input_file = &args[2];
            let opt_level = get_opt_level(&args);
            if let Err(e) = compile_and_run_jit(input_file, opt_level) {
                eprintln!("Execution failed: {}", e);
                std::process::exit(1);
            }
        }
        "run-opt" => {
            if args.len() < 3 {
                eprintln!("Error: No input file specified");
                eprintln!("Usage: viper run-opt <file.vp>");
                std::process::exit(1);
            }
            let input_file = &args[2];
            if let Err(e) = compile_and_run_jit(input_file, 3) {
                eprintln!("Execution failed: {}", e);
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            println!("Viper Compiler 0.2.2");
            println!("Usage: viper <command> [options]");
            println!();
            println!("Commands:");
            println!("  build <file.vp>       AOT compile to native binary");
            println!("  run <file.vp>         JIT compile and run (no opt)");
            println!("  run-opt <file.vp>     JIT compile and run (O3)");
            println!("  help                  Show this help message");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Use 'viper help' for usage information");
            std::process::exit(1);
        }
    }
}

fn get_opt_level(args: &[String]) -> u32 {
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

fn compile_file(input_path: &str, output_path: Option<&str>) -> Result<(), String> {
    compile_file_aot(input_path, 0, output_path)
}

fn compile_file_aot(
    input_path: &str,
    opt_level: u32,
    output_path: Option<&str>,
) -> Result<(), String> {
    println!("🐍 Viper Compiler 0.2.2 (AOT)");
    println!("   Compiling: {}", input_path);
    println!("   Optimization: -O{}", opt_level);

    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{}': {}", input_path, e))?;

    println!("   [1/4] Lexing...");
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    println!("   ✓ Generated {} tokens", tokens.len());

    println!("   [2/4] Parsing...");
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;
    println!("   ✓ Parsed {} statements", ast.statements.len());

    println!("   [3/4] Generating LLVM IR...");
    let context = Context::create();
    let module_name = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);
    codegen.generate(&ast)?;
    codegen.verify()?;
    println!("   ✓ Generated LLVM IR");

    let module = codegen.module();
    let output = output_path.unwrap_or(module_name);

    // For -O2 and -O3, use external opt for better optimization (mem2reg, etc.)
    if opt_level >= 2 {
        println!("   Using LLVM opt for -O{}...", opt_level);
        let bc_path = format!("{}.bc", module_name);
        module.write_bitcode_to_path(Path::new(&bc_path));
        
        let opt_level_str = match opt_level {
            2 => "-O2",
            _ => "-O3",
        };
        
        let opt_bc = format!("{}.opt.bc", module_name);
        std::process::Command::new("/usr/lib/llvm-20/bin/opt")
            .args(&[opt_level_str, "-mtriple=x86_64-pc-linux-gnu", &bc_path, "-o", &opt_bc])
            .output()
            .map_err(|e| format!("opt failed: {}", e))?;
        
        // Use optimized bitcode for object generation
        let context = Context::create();
        let opt_module = inkwell::module::Module::parse_bitcode_from_path(Path::new(&opt_bc), &context)
            .map_err(|e| format!("Failed to load optimized bitcode: {}", e))?;
        
        println!("   [4/4] Emitting object code...");
        emit_object_file(&opt_module, module_name, output)
    } else {
        println!("   Optimizing and emitting object code...");
        emit_object_file(&module, module_name, output)
    }
}

fn emit_object_file(
    module: &inkwell::module::Module,
    module_name: &str,
    output: &str,
) -> Result<(), String> {
    use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple};
    
    let target_triple = TargetTriple::create("x86_64-unknown-linux-gnu");

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("Failed to initialize native target: {}", e))?;

    let target = Target::from_triple(&target_triple)
        .map_err(|e| format!("Failed to get target: {}", e))?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| "Failed to create target machine".to_string())?;

    let obj_path = format!("{}.o", output);
    target_machine
        .write_to_file(module, FileType::Object, Path::new(&obj_path))
        .map_err(|e| format!("Failed to write object file: {}", e))?;

    println!("   ✓ Generated object: {}", obj_path);
    println!("✅ Compilation successful!");
    println!();
    println!("   To link and run:");
    println!("   $ gcc {}.o -o {} -lm", obj_path, output);
    println!("   $ ./{}", output);

    Ok(())
}

fn compile_file_optimized(input_path: &str) -> Result<(), String> {
    println!("🐍 Viper Compiler 0.2.2 (AOT + opt)");
    println!("   Compiling: {}", input_path);

    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{}': {}", input_path, e))?;

    println!("   [1/5] Lexing...");
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    println!("   ✓ Generated {} tokens", tokens.len());

    println!("   [2/5] Parsing...");
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;
    println!("   ✓ Parsed {} statements", ast.statements.len());

    println!("   [3/5] Generating LLVM IR...");
    let context = Context::create();
    let module_name = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);
    codegen.generate(&ast)?;
    codegen.verify()?;
    println!("   ✓ Generated LLVM IR");

    let module = codegen.module();
    let bc_path = format!("{}.bc", module_name);
    module.write_bitcode_to_path(Path::new(&bc_path));
    println!("   ✓ Generated bitcode: {}", bc_path);

    println!("   [4/5] Running LLVM optimizations...");
    let opt_bc = format!("{}.opt.bc", module_name);
    let opt_status = std::process::Command::new("/usr/lib/llvm-20/bin/opt")
        .args(&[
            "-O3",
            "-mtriple=x86_64-pc-linux-gnu",
            "-mcpu=tigerlake",
            &bc_path,
            "-o",
            &opt_bc,
        ])
        .output();

    match opt_status {
        Ok(output) if output.status.success() => {
            println!("   ✓ LLVM optimizations complete");
        }
        Ok(output) => {
            eprintln!(
                "   ⚠ opt warnings: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            eprintln!("   ⚠ opt not found or failed: {}", e);
        }
    }

    println!("   [5/5] Emitting object code...");
    if Path::new(&opt_bc).exists() {
        let obj_path = format!("{}_opt.o", module_name);
        let llc_status = std::process::Command::new("/usr/lib/llvm-20/bin/llc")
            .args(&[
                "-O3",
                "-mtriple=x86_64-pc-linux-gnu",
                "-mcpu=tigerlake",
                "-filetype=obj",
                &opt_bc,
                "-o",
                &obj_path,
            ])
            .output();

        match llc_status {
            Ok(output) if output.status.success() => {
                println!("   ✓ Generated object: {}", obj_path);
                println!("✅ Compilation successful!");
                println!();
                println!("   To link and run:");
                println!(
                    "   $ gcc {} -o {} -L./runtime -lviper -lm",
                    obj_path, module_name
                );
                println!("   $ ./{}", module_name);
            }
            Ok(output) => {
                return Err(format!(
                    "llc failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Err(e) => {
                return Err(format!("llc not found: {}", e));
            }
        }
    } else {
        return Err("Optimization failed - no output".to_string());
    }

    Ok(())
}

fn run_llvm_optimizations(_module: &inkwell::module::Module, _opt_level: u32) -> Result<(), String> {
    // JIT execution engine handles optimization via OptimizationLevel parameter
    // The mem2reg and other optimizations are applied automatically by the JIT
    Ok(())
}

fn compile_and_run(input_path: &str) -> Result<(), String> {
    compile_and_run_jit(input_path, 0)
}

fn compile_and_run_jit(input_path: &str, opt_level: u32) -> Result<(), String> {
    println!("🐍 Viper Compiler 0.2.2 (JIT -O{})", opt_level);
    println!("   Running: {}", input_path);

    let source = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{}': {}", input_path, e))?;

    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;

    let context = Context::create();
    let module_name = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);
    codegen.generate(&ast)?;
    codegen.verify()?;

    // Use optimization level for JIT
    // Note: OptimizationLevel::None is fastest for simple loops because:
    // 1. No optimization overhead during JIT compilation  
    // 2. LLVM JIT still does basic optimizations
    // 3. For compute-heavy code, Default/Aggressive adds too much compile time
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

    unsafe {
        let print_i64_ptr = vp_print_i64 as extern "C" fn(i64);
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_i64")
                .unwrap()
                .as_global_value(),
            print_i64_ptr as usize,
        );

        let print_f64_ptr = vp_print_f64 as extern "C" fn(f64);
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_f64")
                .unwrap()
                .as_global_value(),
            print_f64_ptr as usize,
        );

        let print_bool_ptr = vp_print_bool as extern "C" fn(bool);
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_bool")
                .unwrap()
                .as_global_value(),
            print_bool_ptr as usize,
        );

        let print_newline_ptr = vp_print_newline as extern "C" fn();
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_newline")
                .unwrap()
                .as_global_value(),
            print_newline_ptr as usize,
        );

        if let Some(func) = codegen.module().get_function("vp_list_create") {
            execution_engine
                .add_global_mapping(&func.as_global_value(), vp_list_create_stub as usize);
        }
        if let Some(func) = codegen.module().get_function("vp_list_append") {
            execution_engine
                .add_global_mapping(&func.as_global_value(), vp_list_append_stub as usize);
        }
    }

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

// Runtime function implementations for JIT
extern "C" fn vp_print_i64(val: i64) {
    println!("{}", val);
}

extern "C" fn vp_print_f64(val: f64) {
    println!("{}", val);
}

extern "C" fn vp_print_bool(val: bool) {
    println!("{}", if val { "True" } else { "False" });
}

extern "C" fn vp_print_newline() {
    // Newline is handled by println!
}

// Stub implementations for list functions (Phase 2 MVP)
extern "C" fn vp_list_create_stub() -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}

extern "C" fn vp_list_append_stub(_list: *mut std::ffi::c_void, _val: i64) {
    // Stub - do nothing for now
}
