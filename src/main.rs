mod ast;
mod cli;
mod codegen;
mod lexer;
mod parser;
mod utils;

use inkwell::context::Context;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Viper Compiler 0.1.0");
        eprintln!("Usage: viper <command> [options]");
        eprintln!();
        eprintln!("Commands:");
        eprintln!("  build <file.vp>  Compile a Viper source file");
        eprintln!("  run <file.vp>    Compile and run a Viper source file");
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
            let input_file = &args[2];
            if let Err(e) = compile_file(input_file, None) {
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
            if let Err(e) = compile_and_run(input_file) {
                eprintln!("Execution failed: {}", e);
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            println!("Viper Compiler 0.1.0");
            println!("Usage: viper <command> [options]");
            println!();
            println!("Commands:");
            println!("  build <file.vp>  Compile a Viper source file");
            println!("  run <file.vp>    Compile and run a Viper source file");
            println!("  help             Show this help message");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Use 'viper help' for usage information");
            std::process::exit(1);
        }
    }
}

fn compile_file(input_path: &str, output_path: Option<&str>) -> Result<(), String> {
    println!("🐍 Viper Compiler 0.1.0");
    println!("   Compiling: {}", input_path);

    // Read source file
    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{}': {}", input_path, e))?;

    // Phase 1: Lexing
    println!("   [1/4] Lexing...");
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    println!("   ✓ Generated {} tokens", tokens.len());

    // Phase 2: Parsing
    println!("   [2/4] Parsing...");
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;
    println!("   ✓ Parsed {} statements", ast.statements.len());

    // Phase 3: Code Generation
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

    // Phase 4: Emit object file
    println!("   [4/4] Emitting object code...");
    let output = output_path.unwrap_or(module_name);

    // Emit LLVM bitcode
    let bitcode_path = format!("{}.bc", output);

    // Write bitcode to file
    codegen
        .module()
        .write_bitcode_to_path(std::path::Path::new(&bitcode_path));

    println!("   ✓ Generated bitcode: {}", bitcode_path);
    println!("✅ Compilation successful!");
    println!();
    println!("   To link and run:");
    println!("   $ llc {} -filetype=obj -o {}.o", bitcode_path, output);
    println!("   $ gcc {}.o -o {} -L./runtime -lviper", output, output);
    println!("   $ ./{}", output);

    Ok(())
}

fn compile_and_run(input_path: &str) -> Result<(), String> {
    println!("🐍 Viper Compiler 0.2.0");
    println!("   Running: {}", input_path);

    // Read source file
    let source = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{}': {}", input_path, e))?;

    // Phase 1: Lexing
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;

    // Phase 2: Parsing
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;

    // Phase 3: Code Generation
    let context = Context::create();
    let module_name = Path::new(input_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);
    codegen.generate(&ast)?;
    codegen.verify()?;

    // Phase 4: JIT Execution
    println!("   [4/4] Executing via JIT...");

    // We need to initialize native targets for JIT
    inkwell::targets::Target::initialize_native(&inkwell::targets::InitializationConfig::default())
        .map_err(|e| format!("Failed to initialize native target: {}", e))?;

    let execution_engine = codegen
        .module()
        .create_jit_execution_engine(inkwell::OptimizationLevel::None)
        .map_err(|e| format!("Failed to create JIT engine: {}", e))?;

    // Register runtime function implementations for JIT
    // For Phase 2, we use simple C function pointers
    unsafe {
        // Register vp_print_i64
        let print_i64_ptr = vp_print_i64 as extern "C" fn(i64);
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_i64")
                .unwrap()
                .as_global_value(),
            print_i64_ptr as usize,
        );

        // Register vp_print_f64
        let print_f64_ptr = vp_print_f64 as extern "C" fn(f64);
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_f64")
                .unwrap()
                .as_global_value(),
            print_f64_ptr as usize,
        );

        // Register vp_print_bool
        let print_bool_ptr = vp_print_bool as extern "C" fn(bool);
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_bool")
                .unwrap()
                .as_global_value(),
            print_bool_ptr as usize,
        );

        // Register vp_print_newline
        let print_newline_ptr = vp_print_newline as extern "C" fn();
        execution_engine.add_global_mapping(
            &codegen
                .module()
                .get_function("vp_print_newline")
                .unwrap()
                .as_global_value(),
            print_newline_ptr as usize,
        );

        // Register list functions (stubs for now)
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
