use crate::lexer;
use crate::parser;
use crate::codegen;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub fn compile_file(input_path: &str, output_path: Option<&str>) -> Result<(), String> {
    compile_file_aot(input_path, 0, output_path, false, false, None)
}

pub fn compile_file_aot(
    input_path: &str,
    opt_level: u32,
    output_path: Option<&str>,
    lto: bool,
    emit_llvm: bool,
    pgo: Option<&str>,
) -> Result<(), String> {
    println!("🐍 Viper Compiler {} (AOT)", env!("CARGO_PKG_VERSION"));
    println!("   Compiling: {}", input_path);
    println!("   Optimization: -O{}", opt_level);
    if lto {
        println!("   LTO: enabled");
    }
    if emit_llvm {
        println!("   Emit LLVM: enabled");
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

    // Semantic Analysis (Type Checking)
    println!("   [2.2/4] Type checking...");
    let mut type_checker = crate::semantic::type_checker::TypeChecker::new();
    type_checker.check(&ast).map_err(|e| {
        format!("Type errors found:\n{}", e.iter().map(|err| format!(" - {}", err)).collect::<Vec<_>>().join("\n"))
    })?;

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

    /* Emit LLVM IR to .ll file if requested */
    if emit_llvm {
        let ll_path = format!("{}.ll", module_name);
        module
            .print_to_file(&ll_path)
            .map_err(|e| format!("Failed to write LLVM IR to '{}': {}", ll_path, e))?;
        println!("   ✓ Emitted LLVM IR: {}", ll_path);
    }

    // Default output name: source file stem + _vp suffix (e.g., sieve.vp -> sieve_vp.o)
    let default_output = format!("{}_vp", module_name);
    let output = output_path.unwrap_or(&default_output);

    // For -O1 and above, use external opt for better optimization (mem2reg, etc.)
    if opt_level >= 1 {
        println!("   Using LLVM opt for -O{}...", opt_level);
        let bc_path = format!("{}.bc", module_name);
        module.write_bitcode_to_path(Path::new(&bc_path));

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
        // -O1: Basic optimizations with mem2reg for stack-to-register promotion
        // -O2: Adds vectorization
        // -O3: Adds aggressive vectorization and loop optimizations
        let passes = match opt_level {
            1 => "default<O1>,mem2reg,instcombine,simplifycfg",
            2 => "default<O2>,mem2reg,instcombine,simplifycfg,gvn,loop-vectorize",
            3 => "default<O3>,mem2reg,instcombine,simplifycfg,gvn,loop-vectorize,loop-unroll",
            _ => "default<O1>,mem2reg,instcombine,simplifycfg",
        };

        opt_args.push("--passes");
        opt_args.push(passes);

        let opt_output = std::process::Command::new("/usr/lib/llvm-20/bin/opt")
            .args(&opt_args)
            .output()
            .map_err(|e| format!("opt failed: {}", e))?;

        if !opt_output.status.success() {
            eprintln!(
                "   ⚠ opt stderr: {}",
                String::from_utf8_lossy(&opt_output.stderr)
            );
            return Err(format!("opt optimization failed"));
        }

        // Use optimized bitcode for object generation
        let context = Context::create();
        let opt_module =
            inkwell::module::Module::parse_bitcode_from_path(Path::new(&opt_bc), &context)
                .map_err(|e| format!("Failed to load optimized bitcode '{}': {}", opt_bc, e))?;

        // Emit optimized LLVM IR to .ll file if requested (shows optimized IR, not raw)
        if emit_llvm {
            let opt_ll_path = format!("{}.opt.ll", module_name);
            opt_module.print_to_file(&opt_ll_path).map_err(|e| {
                format!(
                    "Failed to write optimized LLVM IR to '{}': {}",
                    opt_ll_path, e
                )
            })?;
            println!("   ✓ Emitted optimized LLVM IR: {}", opt_ll_path);
        }

        println!("   [4/4] Emitting object code...");
        emit_object_file(&opt_module, module_name, output, opt_level, lto, pgo)
    } else {
        println!("   Optimizing and emitting object code...");
        emit_object_file(&module, module_name, output, opt_level, lto, pgo)
    }
}

pub fn emit_object_file(
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
pub fn link_with_gcc(
    obj_path: &str,
    bin_path: &str,
    lto: bool,
    pgo: Option<&str>,
    opt_level: u32,
) -> Result<(), String> {
    // Check local paths first, then system-wide installation
    let runtime_paths = [
        "runtime/obj",
        "../runtime/obj",
        "../../runtime/obj",
        "/usr/local/lib/viper",
        "/usr/lib/viper",
        "/opt/viper/lib",
    ];

    let mut runtime_path: Option<String> = runtime_paths
        .iter()
        .find(|p| Path::new(p).exists())
        .map(|p| p.to_string());

    // Check $HOME/.local/lib/viper
    if runtime_path.is_none() {
        if let Ok(home) = std::env::var("HOME") {
            let home_lib = format!("{}/.local/lib/viper", home);
            if Path::new(&home_lib).exists() {
                runtime_path = Some(home_lib);
            }
        }
    }

    let runtime_path = runtime_path.ok_or_else(|| "Runtime object files not found".to_string())?;

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

    // Add output
    args.extend_from_slice(&["-o".to_string(), bin_path.to_string()]);

    // Add runtime.o for additional runtime functions (only if using local path)
    if runtime_path.starts_with(".") {
        args.push(format!("{}/runtime.o", runtime_path));
    }

    // Add library path and libraries
    args.extend_from_slice(&[
        format!("-L{}", runtime_path),
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

pub fn compile_file_optimized(input_path: &str) -> Result<(), String> {
    println!(
        "🐍 Viper Compiler {} (AOT + opt)",
        env!("CARGO_PKG_VERSION")
    );
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

