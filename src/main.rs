mod ast;
mod cli;
mod codegen;
mod lexer;
mod parser;
mod semantic;
mod utils;

use clap::Parser;
use cli::args::{Args, Commands};
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use std::fs;
use std::path::Path;
use std::process;

fn main() {
    // Check prerequisites before parsing commands
    if let Err(e) = check_prerequisites() {
        eprintln!("Error: {}", e);
        eprintln!();
        eprintln!("Please ensure:");
        eprintln!("  1. LLVM 20.x is installed and in PATH");
        eprintln!("  2. Viper runtime library is built (runtime/libviper.a)");
        eprintln!("  3. GCC is installed for linking AOT binaries");
        eprintln!();
        eprintln!("To build the runtime library:");
        eprintln!("  cd runtime && make");
        process::exit(1);
    }

    let args = Args::parse();

    match args.command {
        Commands::Build {
            input,
            output,
            optimize,
            lto,
            pgo,
        } => {
            if let Err(e) =
                compile_file_aot(&input, optimize, output.as_deref(), lto, pgo.as_deref())
            {
                eprintln!("Compilation failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run {
            input,
            optimize,
            lto: _,
            pgo: _,
        } => {
            if let Err(e) = compile_and_run_jit(&input, optimize) {
                eprintln!("Execution failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| "viper_project".to_string());
            if let Err(e) = init_project(&project_name) {
                eprintln!("Init failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Info => {
            show_info();
        }
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn compile_file(input_path: &str, output_path: Option<&str>) -> Result<(), String> {
    compile_file_aot(input_path, 0, output_path, false, None)
}

fn compile_file_aot(
    input_path: &str,
    opt_level: u32,
    output_path: Option<&str>,
    lto: bool,
    pgo: Option<&str>,
) -> Result<(), String> {
    println!("🐍 Viper Compiler 0.2.3 (AOT)");
    println!("   Compiling: {}", input_path);
    println!("   Optimization: -O{}", opt_level);
    if lto {
        println!("   LTO: enabled");
    }
    if let Some(pgo_mode) = &pgo {
        println!("   PGO: {} mode", pgo_mode);
    }

    let source = fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read '{}': {}", input_path, e))?;

    println!("   [1/4] Lexing...");
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    println!("   ✓ Generated {} tokens", tokens.len());

    println!("   [2/4] Parsing...");
    let mut parser = parser::Parser::new(tokens);
    let mut ast = parser.parse()?;
    println!("   ✓ Parsed {} statements", ast.statements.len());

    // Apply Dead Code Elimination optimization
    if opt_level >= 1 {
        println!("   [2.5/4] Running DCE optimization...");
        let mut dce = codegen::DeadCodeEliminator::new();
        ast = dce.optimize(&ast);
        println!(
            "   ✓ DCE complete, {} statements remaining",
            ast.statements.len()
        );
    }

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
    // Default output name: source file stem + _vp suffix (e.g., sieve.vp -> sieve_vp.o)
    let default_output = format!("{}_vp", module_name);
    let output = output_path.unwrap_or(&default_output);

    // For -O2 and -O3, use external opt for better optimization (mem2reg, etc.)
    if opt_level >= 1 {
        println!("   Using LLVM opt for -O{}...", opt_level);
        let bc_path = format!("{}.bc", module_name);
        module.write_bitcode_to_path(Path::new(&bc_path));

        // Use more aggressive optimization passes
        let opt_level_str = match opt_level {
            1 => "-O2", // -O1 uses -O2 passes for better results
            2 => "-O3",
            _ => "-O3",
        };

        let opt_bc = format!("{}.opt.bc", module_name);

        // Add aggressive optimization passes for better performance
        // LLVM 20 uses --passes= syntax for the new pass manager
        let mut opt_args = vec![
            "-mtriple=x86_64-pc-linux-gnu",
            "-mcpu=native",
            &bc_path,
            "-o",
            &opt_bc,
        ];

        // Build the passes string based on optimization level
        if opt_level >= 2 {
            // Use the default pipeline with extra passes
            opt_args.push("--passes=default<O3>");
        } else {
            opt_args.push(opt_level_str);
        }

        let opt_output = std::process::Command::new("/usr/lib/llvm-20/bin/opt")
            .args(&opt_args)
            .output()
            .map_err(|e| format!("opt failed: {}", e))?;

        if !opt_output.status.success() {
            eprintln!("   ⚠ opt stderr: {}", String::from_utf8_lossy(&opt_output.stderr));
            return Err(format!("opt optimization failed"));
        }

        // Use optimized bitcode for object generation
        let context = Context::create();
        let opt_module =
            inkwell::module::Module::parse_bitcode_from_path(Path::new(&opt_bc), &context)
                .map_err(|e| format!("Failed to load optimized bitcode '{}': {}", opt_bc, e))?;

        println!("   [4/4] Emitting object code...");
        emit_object_file(&opt_module, module_name, output, opt_level, lto, pgo)
    } else {
        println!("   Optimizing and emitting object code...");
        emit_object_file(&module, module_name, output, opt_level, lto, pgo)
    }
}

fn emit_object_file(
    module: &inkwell::module::Module,
    _module_name: &str,
    output: &str,
    opt_level: u32,
    lto: bool,
    pgo: Option<&str>,
) -> Result<(), String> {
    use inkwell::targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple,
    };

    let target_triple = TargetTriple::create("x86_64-unknown-linux-gnu");

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("Failed to initialize native target: {}", e))?;

    let target =
        Target::from_triple(&target_triple).map_err(|e| format!("Failed to get target: {}", e))?;

    // Map optimization level to LLVM optimization
    let llvm_opt = match opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    };

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "",
            "",
            llvm_opt,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| "Failed to create target machine".to_string())?;

    let obj_path = format!("{}.o", output);
    target_machine
        .write_to_file(module, FileType::Object, Path::new(&obj_path))
        .map_err(|e| format!("Failed to write object file: {}", e))?;

    println!("   ✓ Generated object: {}", obj_path);

    // Build GCC link command with LTO and PGO support
    let bin_path = format!("{}_bin", output);
    link_with_gcc(&obj_path, &bin_path, lto, pgo, opt_level)?;

    println!("✅ Compilation successful!");
    println!("   Binary: {}", bin_path);

    Ok(())
}

/// Link object file with GCC, supporting LTO and PGO
fn link_with_gcc(
    obj_path: &str,
    bin_path: &str,
    lto: bool,
    pgo: Option<&str>,
    opt_level: u32,
) -> Result<(), String> {
    let mut args = vec![obj_path.to_string()];

    // Add optimization flags
    if opt_level > 0 {
        args.push(format!("-O{}", opt_level));
    }

    // Add LTO flag
    if lto {
        args.push("-flto".to_string());
        println!("   [LTO] Enabled link-time optimization");
    }

    // Add PGO flags based on mode
    if let Some(pgo_mode) = pgo {
        match pgo_mode {
            "instrument" => {
                // Phase 1: Generate instrumented binary for profile collection
                args.push("-fprofile-generate".to_string());
                println!("   [PGO] Instrumentation mode - run binary to collect profiles");
            }
            "use" => {
                // Phase 2: Use collected profiles for optimization
                args.push("-fprofile-use".to_string());
                args.push("-fprofile-correction".to_string()); // Handle missing profiles gracefully
                println!("   [PGO] Using collected profiles for optimization");
            }
            _ => {}
        }
    }

    // Add output, library paths, and libraries
    args.extend_from_slice(&[
        "-o".to_string(),
        bin_path.to_string(),
        "-L./runtime".to_string(),
        "-lviper".to_string(),
        "-lm".to_string(),
    ]);

    println!("   Linking with GCC...");
    let output = std::process::Command::new("gcc")
        .args(&args)
        .output()
        .map_err(|e| format!("GCC linking failed: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "GCC linking failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    println!("   ✓ Linked binary: {}", bin_path);

    // Print PGO instructions
    if let Some(pgo_mode) = pgo {
        match pgo_mode {
            "instrument" => {
                println!();
                println!("   📊 PGO Phase 1: Profile Collection");
                println!("   Run your binary with representative workloads:");
                println!("   $ ./{}", bin_path);
                println!("   Profiles will be saved to *.gcda files");
                println!();
                println!("   Then rebuild with --pgo=use to apply optimizations");
            }
            "use" => {
                println!();
                println!("   📊 PGO Phase 2: Profile-Guided Optimization Applied");
                println!("   Binary is optimized based on collected profiles");
            }
            _ => {}
        }
    }

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
    let mut ast = parser.parse()?;
    println!("   ✓ Parsed {} statements", ast.statements.len());

    println!("   [2.5/5] Running DCE optimization...");
    let mut dce = codegen::DeadCodeEliminator::new();
    ast = dce.optimize(&ast);
    println!(
        "   ✓ DCE complete, {} statements remaining",
        ast.statements.len()
    );

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

    // Use aggressive optimization passes
    let opt_status = std::process::Command::new("/usr/lib/llvm-20/bin/opt")
        .args(&[
            "-O3",
            "-mtriple=x86_64-pc-linux-gnu",
            "-mcpu=native",
            &bc_path,
            "-o",
            &opt_bc,
            "-mem2reg",
            "-instcombine",
            "-simplifycfg",
            "-loop-unroll",
            "-inline",
            "-gvn",
            "-licm",
            "-slp-vectorize",
            "-loop-vectorize",
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
        let obj_path = format!("{}_vp.o", module_name);
        let llc_status = std::process::Command::new("/usr/lib/llvm-20/bin/llc")
            .args(&[
                "-O3",
                "-mtriple=x86_64-pc-linux-gnu",
                "-mcpu=native",
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
                    "   $ gcc {} -o {}_bin -L./runtime -lviper -lm",
                    obj_path, module_name
                );
                println!("   $ ./{}_bin", module_name);
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

fn run_llvm_optimizations(
    _module: &inkwell::module::Module,
    _opt_level: u32,
) -> Result<(), String> {
    // JIT execution engine handles optimization via OptimizationLevel parameter
    // The mem2reg and other optimizations are applied automatically by the JIT
    Ok(())
}

fn compile_and_run(input_path: &str) -> Result<(), String> {
    compile_and_run_jit(input_path, 0)
}

fn compile_and_run_jit(input_path: &str, opt_level: u32) -> Result<(), String> {
    use inkwell::targets::{InitializationConfig, Target};
    
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

    let print_str_ptr = vp_print_str_stub as extern "C" fn(*mut std::ffi::c_void);
    if let Some(func) = codegen.module().get_function("vp_print_str") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            print_str_ptr as usize,
        );
    }

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
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_create_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_append") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_append_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_free") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_free_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_get") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_get_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_len") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_len_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_set") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_set_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_insert") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_insert_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_remove") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_remove_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_pop") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_pop_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_clear") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_clear_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_list_contains") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_list_contains_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_retain") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_retain_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_release") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_release_stub as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_str_concat") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_str_concat_stub as *const () as usize,
        );
    }
    // Concurrency runtime functions (Phase 3)
    if let Some(func) = codegen.module().get_function("vp_chan_create") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_chan_create as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_chan_destroy") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_chan_destroy as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_chan_send") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_chan_send as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_chan_recv") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_chan_recv as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_waitgroup_create") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_waitgroup_create as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_waitgroup_destroy") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_waitgroup_destroy as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_waitgroup_add") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_waitgroup_add as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_waitgroup_done") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_waitgroup_done as *const () as usize,
        );
    }
    if let Some(func) = codegen.module().get_function("vp_waitgroup_wait") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_waitgroup_wait as *const () as usize,
        );
    }
    // Async/await runtime stub (Phase 3 - partial implementation)
    if let Some(func) = codegen.module().get_function("vp_future_await") {
        execution_engine.add_global_mapping(
            &func.as_global_value(),
            vp_future_await as *const () as usize,
        );
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

extern "C" fn vp_print_str_stub(s: *mut std::ffi::c_void) {
    if s.is_null() {
        return;
    }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(s as *const std::ffi::c_char);
        if let Ok(rust_str) = c_str.to_str() {
            println!("{}", rust_str);
        }
    }
}

// Stub implementations for list functions (Phase 2 MVP)
// Using Box<Vec<i64>> as the internal representation
extern "C" fn vp_list_create_stub() -> *mut std::ffi::c_void {
    let list = Box::new(Vec::<i64>::new());
    Box::into_raw(list) as *mut std::ffi::c_void
}

extern "C" fn vp_list_append_stub(list: *mut std::ffi::c_void, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.push(val);
    }
}

extern "C" fn vp_list_free_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let _ = Box::from_raw(list as *mut Vec<i64>);
    }
}

extern "C" fn vp_list_get_stub(list: *mut std::ffi::c_void, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        if index < 0 || index as usize >= vec.len() {
            return 0;
        }
        vec[index as usize]
    }
}

extern "C" fn vp_list_len_stub(list: *mut std::ffi::c_void) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        vec.len() as i64
    }
}

extern "C" fn vp_list_set_stub(list: *mut std::ffi::c_void, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        if index >= 0 && (index as usize) < vec.len() {
            vec[index as usize] = val;
        }
    }
}

extern "C" fn vp_list_insert_stub(list: *mut std::ffi::c_void, index: i64, val: i64) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        if index >= 0 && (index as usize) <= vec.len() {
            vec.insert(index as usize, val);
        }
    }
}

extern "C" fn vp_list_remove_stub(list: *mut std::ffi::c_void, index: i64) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        if index >= 0 && (index as usize) < vec.len() {
            vec.remove(index as usize)
        } else {
            0
        }
    }
}

extern "C" fn vp_list_pop_stub(list: *mut std::ffi::c_void) -> i64 {
    if list.is_null() {
        return 0;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.pop().unwrap_or(0)
    }
}

extern "C" fn vp_list_clear_stub(list: *mut std::ffi::c_void) {
    if list.is_null() {
        return;
    }
    unsafe {
        let vec = &mut *(list as *mut Vec<i64>);
        vec.clear();
    }
}

extern "C" fn vp_list_contains_stub(list: *mut std::ffi::c_void, val: i64) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe {
        let vec = &*(list as *mut Vec<i64>);
        vec.contains(&val)
    }
}

extern "C" fn vp_retain_stub(_ptr: *mut std::ffi::c_void) {
    // No-op for JIT
}

extern "C" fn vp_release_stub(_ptr: *mut std::ffi::c_void) {
    // No-op for JIT
}

/// String concatenation stub for JIT
/// Uses a simple static buffer approach for JIT execution
extern "C" fn vp_str_concat_stub(
    a: *const std::ffi::c_char,
    b: *const std::ffi::c_char,
) -> *const std::ffi::c_char {
    use std::ffi::CStr;

    if a.is_null() || b.is_null() {
        return std::ptr::null();
    }

    unsafe {
        let str_a = CStr::from_ptr(a).to_string_lossy();
        let str_b = CStr::from_ptr(b).to_string_lossy();
        let concatenated = format!("{}{}", str_a, str_b);

        // Leak the string to keep it alive for JIT execution
        // This is a memory leak but acceptable for short-lived JIT execution
        let boxed = Box::new(concatenated);
        let ptr = Box::into_raw(boxed) as *const std::ffi::c_char;
        ptr
    }
}

/// Initialize a new Viper project
fn init_project(name: &str) -> Result<(), String> {
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

// Concurrency runtime stubs for JIT (Phase 3)
// Simplified implementations using atomics for safety

use std::sync::atomic::{AtomicUsize, AtomicI64, Ordering};

static JIT_CHANNEL_COUNTER: AtomicUsize = AtomicUsize::new(0);
static JIT_CHANNEL_VALUE: AtomicI64 = AtomicI64::new(0);
static JIT_WG_COUNTER: AtomicUsize = AtomicUsize::new(0);

extern "C" fn vp_chan_create(_capacity: i64) -> *mut std::ffi::c_void {
    let id = JIT_CHANNEL_COUNTER.fetch_add(1, Ordering::SeqCst);
    id as *mut std::ffi::c_void
}

extern "C" fn vp_chan_destroy(_chan: *mut std::ffi::c_void) {
    // No-op for JIT
}

extern "C" fn vp_chan_send(_chan: *mut std::ffi::c_void, value: i64) {
    JIT_CHANNEL_VALUE.store(value, Ordering::SeqCst);
}

extern "C" fn vp_chan_recv(_chan: *mut std::ffi::c_void) -> i64 {
    JIT_CHANNEL_VALUE.load(Ordering::SeqCst)
}

extern "C" fn vp_waitgroup_create() -> *mut std::ffi::c_void {
    let id = JIT_WG_COUNTER.fetch_add(1, Ordering::SeqCst);
    id as *mut std::ffi::c_void
}

extern "C" fn vp_waitgroup_destroy(_wg: *mut std::ffi::c_void) {
    // No-op for JIT
}

extern "C" fn vp_waitgroup_add(_wg: *mut std::ffi::c_void, _n: i64) {
    // No-op for JIT stub
}

extern "C" fn vp_waitgroup_done(_wg: *mut std::ffi::c_void) {
    // No-op for JIT stub
}

extern "C" fn vp_waitgroup_wait(_wg: *mut std::ffi::c_void) {
    // No-op for JIT stub
}

extern "C" fn vp_init_threadpool(_num_threads: usize) {
    // No-op for JIT
}

extern "C" fn vp_shutdown_threadpool() {
    // No-op for JIT
}

extern "C" fn vp_future_await(future: i64) -> i64 {
    // Stub for async/await - just returns the future value as-is
    // A full implementation would suspend and resume the coroutine
    future
}

/// Check that all prerequisites are available
fn check_prerequisites() -> Result<(), String> {
    // Check runtime library exists (only needed for AOT compilation)
    let runtime_paths = [
        "runtime/libviper.a",
        "../runtime/libviper.a",
        "../../runtime/libviper.a",
    ];

    let runtime_found = runtime_paths.iter().any(|p| Path::new(p).exists());

    if !runtime_found {
        return Err("Viper runtime library not found (runtime/libviper.a)".to_string());
    }

    // Check GCC for AOT linking
    if !check_command_exists("gcc") {
        return Err("GCC compiler not found in PATH".to_string());
    }

    Ok(())
}

/// Check if a command exists in PATH
fn check_command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}

/// Show compiler information
fn show_info() {
    println!("Viper Compiler 0.2.3");
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
