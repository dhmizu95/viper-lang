use crate::codegen;
use crate::error::{Result, ViperError};
use crate::lexer;
use crate::parser;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use std::fs;
use std::path::Path;

/// Find LLVM tool path - checks environment variable first, then uses default
fn find_llvm_tool(tool: &str) -> String {
    std::env::var(format!("LLVM_{}_PATH", tool.to_uppercase()))
        .unwrap_or_else(|_| format!("/usr/lib/llvm-21/bin/{}", tool))
}

#[allow(dead_code)]
pub fn compile_file(input_path: &str, output_path: Option<&str>) -> Result<()> {
    compile_file_aot(input_path, 0, output_path, false, false, None)
}

pub fn compile_file_aot(
    input_path: &str,
    opt_level: u32,
    output_path: Option<&str>,
    lto: bool,
    emit_llvm: bool,
    pgo: Option<&str>,
) -> Result<()> {
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
        .map_err(ViperError::Io)?;

    println!("   [1/4] Lexing...");
    let mut lexer = lexer::Lexer::new(&source);
    let tokens = lexer.tokenize()?;
    println!("   ✓ Generated {} tokens", tokens.len());

    println!("   [2/4] Parsing...");
    let mut parser = parser::Parser::new(tokens);
    let mut ast = parser.parse()?;
    println!("   ✓ Parsed {} statements", ast.statements.len());

    println!("   [2.1/4] Loading modules...");
    println!("   [2.2/4] Type checking...");
    let type_checker = crate::driver::type_check_module(Path::new(input_path), &ast)?;
    let loaded_count = type_checker.module_loader.loaded_modules().len();
    if loaded_count > 0 {
        println!("   ✓ Loaded {} module(s)", loaded_count);
    } else {
        println!("   ✓ No external modules imported");
    }

    // Apply Constant Folding optimization
    if opt_level >= 1 {
        println!("   [2.3/4] Running constant folding...");
        let mut constant_folder = crate::semantic::ConstantFolder::new();
        constant_folder.fold(&mut ast);
        println!("   ✓ Constant folding complete");
    }

    // Run Recursion Analysis to detect recursive functions
    println!("   [2.4/4] Running recursion analysis...");
    let (warnings, recursive_func_count) = crate::driver::analyze_recursive_functions(&ast);
    for warning in &warnings {
        eprintln!("   {}", warning);
    }

    if !warnings.is_empty() {
        println!("   ℹ {} recursive function(s) could benefit from @lru_cache", warnings.len());
    } else if recursive_func_count > 0 {
        println!("   ✓ All recursive functions are memoized");
    } else {
        println!("   ✓ No recursive functions detected");
    }

    // Apply Loop Invariant Code Motion (LICM) optimization
    if opt_level >= 2 {
        println!("   [2.4/4] Running LICM (Loop Invariant Code Motion)...");
        let mut licm = codegen::LicmPass::new();
        licm.run(&mut ast);
        println!("   ✓ LICM complete");
    }

    // Apply Dead Code Elimination optimization
    if opt_level >= 1 {
        println!("   [2.5/4] Running DCE optimization...");
        let mut dce = codegen::DeadCodeEliminator::new();
        ast = dce.optimize(&ast);
        println!("   ✓ DCE complete, {} statements remaining", ast.statements.len());
    }

    println!("   [3/4] Generating LLVM IR...");
    let context = Context::create();
    let module_name = Path::new(input_path).file_stem().and_then(|s| s.to_str()).unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);
    codegen.generate(&ast).map_err(ViperError::codegen)?;
    codegen.verify().map_err(ViperError::codegen)?;
    println!("   ✓ Generated LLVM IR");

    // Report BigInt functions (they have special optimization handling)
    let bigint_funcs = codegen.bigint_functions();
    if !bigint_funcs.is_empty() {
        println!("   ℹ BigInt functions (optnone applied): {}", bigint_funcs.iter().cloned().collect::<Vec<_>>().join(", "));
    }

    let module = codegen.module();

    /* Emit LLVM IR to .ll file if requested */
    if emit_llvm {
        let ll_path = format!("{}.ll", module_name);
        module
            .print_to_file(&ll_path)
            .map_err(|e| ViperError::driver(format!("Failed to write LLVM IR to '{}': {}", ll_path, e)))?;
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
        let mut opt_args =
            vec!["-mtriple=x86_64-pc-linux-gnu", "-mcpu=native", &bc_path, "-o", &opt_bc];

        // Build the passes string based on optimization level
        // -O0: No optimization (debug builds)
        // -O1: Basic optimizations for fast compilation
        // -O2: Balanced optimizations for production builds
        // -O3: Aggressive optimizations with tuned parameters to avoid regressions
        // Note: LLVM 21 default<O1/O2/O3> passes include comprehensive optimization pipelines
        // Custom O3 pipeline avoids aggressive loop unrolling that can cause code bloat
        let passes = match opt_level {
            0 => "verify",
            1 => "default<O1>",
            2 => "default<O2>",
            3 => {
                // Custom O3 pipeline: aggressive inlining + vectorization without excessive loop unrolling
                // This avoids the O3 regression where default<O3> is slower than O2 on some benchmarks
                "mem2reg,instcombine,simplifycfg,inline,loop-vectorize,slp-vectorize,gvn,licm,loop-unroll(max-unroll=4)"
            }
            _ => "default<O1>",
        };

        opt_args.push("--passes");
        opt_args.push(passes);

        let opt_output = std::process::Command::new(find_llvm_tool("opt"))
            .args(&opt_args)
            .output()
            .map_err(ViperError::Io)?;

        if !opt_output.status.success() {
            eprintln!("   ⚠ opt stderr: {}", String::from_utf8_lossy(&opt_output.stderr));
            return Err(ViperError::driver("opt optimization failed"));
        }

        // Use optimized bitcode for object generation
        let context = Context::create();
        let opt_module =
            inkwell::module::Module::parse_bitcode_from_path(Path::new(&opt_bc), &context)
                .map_err(|e| ViperError::driver(format!("Failed to load optimized bitcode '{}': {}", opt_bc, e)))?;

        // Emit optimized LLVM IR to .ll file if requested (shows optimized IR, not raw)
        if emit_llvm {
            let opt_ll_path = format!("{}.opt.ll", module_name);
            opt_module.print_to_file(&opt_ll_path).map_err(|e| {
                ViperError::driver(format!("Failed to write optimized LLVM IR to '{}': {}", opt_ll_path, e))
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
) -> Result<()> {
    use inkwell::targets::{
        CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetTriple,
    };

    let target_triple = TargetTriple::create("x86_64-unknown-linux-gnu");

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| ViperError::driver(format!("Failed to initialize native target: {}", e)))?;

    let target =
        Target::from_triple(&target_triple).map_err(|e| ViperError::driver(format!("Failed to get target: {}", e)))?;

    // Map optimization level to LLVM optimization
    let llvm_opt = match opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    };

    let target_machine = target
        .create_target_machine(&target_triple, "", "", llvm_opt, RelocMode::PIC, CodeModel::Default)
        .ok_or_else(|| ViperError::driver("Failed to create target machine"))?;

    let obj_path = format!("{}.o", output);
    target_machine
        .write_to_file(module, FileType::Object, Path::new(&obj_path))
        .map_err(|e| ViperError::driver(format!("Failed to write object file: {}", e)))?;

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
) -> Result<()> {
    // Check local paths first, then system-wide installation
    let runtime_paths = [
        "runtime/obj",
        "../runtime/obj",
        "../../runtime/obj",
        "/usr/local/lib/viper",
        "/usr/lib/viper",
        "/opt/viper/lib",
    ];

    let mut runtime_path: Option<String> =
        runtime_paths.iter().find(|p| Path::new(p).exists()).map(|p| p.to_string());

    // Check $HOME/.local/lib/viper
    if runtime_path.is_none() {
        if let Ok(home) = std::env::var("HOME") {
            let home_lib = format!("{}/.local/lib/viper", home);
            if Path::new(&home_lib).exists() {
                runtime_path = Some(home_lib);
            }
        }
    }

    let runtime_path = runtime_path.ok_or_else(|| ViperError::driver("Runtime object files not found"))?;

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

    // Note: Don't add runtime.o separately - libviper.a already contains all runtime symbols
    // Adding both causes duplicate symbol errors (e.g., vp_print_str defined in both)

    // Add library path and libraries
    args.extend_from_slice(&[
        format!("-L{}", runtime_path),
        "-lviper".to_string(),
    ]);

    // Add vendor GMP library path
    args.push("-Lvendor/gmp/lib".to_string());

    // Link libraries (GMP for BigInt support)
    args.extend_from_slice(&[
        "-lgmp".to_string(),
        "-lm".to_string(),
        "-lpthread".to_string(),
    ]);

    println!("   Linking with GCC...");
        let output = std::process::Command::new("gcc")
        .args(&args)
        .output()
            .map_err(ViperError::Io)?;

    if !output.status.success() {
        return Err(ViperError::driver(format!(
            "GCC linking failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
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

pub fn compile_file_optimized(input_path: &str) -> Result<()> {
    println!("🐍 Viper Compiler {} (AOT + opt)", env!("CARGO_PKG_VERSION"));
    println!("   Compiling: {}", input_path);

    let source = fs::read_to_string(input_path)
        .map_err(ViperError::Io)?;

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
    println!("   ✓ DCE complete, {} statements remaining", ast.statements.len());

    println!("   [3/5] Generating LLVM IR...");
    let context = Context::create();
    let module_name = Path::new(input_path).file_stem().and_then(|s| s.to_str()).unwrap_or("main");

    let mut codegen = codegen::CodeGen::new(&context, module_name);
    codegen.generate(&ast).map_err(ViperError::codegen)?;
    codegen.verify().map_err(ViperError::codegen)?;
    println!("   ✓ Generated LLVM IR");

    let module = codegen.module();
    let bc_path = format!("{}.bc", module_name);
    module.write_bitcode_to_path(Path::new(&bc_path));
    println!("   ✓ Generated bitcode: {}", bc_path);

    println!("   [4/5] Running LLVM optimizations...");
    let opt_bc = format!("{}.opt.bc", module_name);

    // Use aggressive optimization passes with comprehensive coverage
    // Passes are ordered for optimal interaction:
    // 1. mem2reg: Promote allocas to SSA registers (must be early)
    // 2. simplifycfg: Clean up control flow
    // 3. instcombine: Combine instructions
    // 4. gvn: Global Value Numbering (redundant load elimination)
    // 5. licm: Loop-invariant code motion
    // 6. loop-vectorize: Auto-vectorize loops
    // 7. slp-vectorize: Straight-line code vectorization
    // 8. inline: Function inlining
    // 9. loop-unroll: Unroll small loops
    // 10. coro-early: Early coroutine optimization
    // 11. cg-sccp: Interprocedural constant propagation
    let opt_status = std::process::Command::new(find_llvm_tool("opt"))
        .args(&[
            "-O3",
            "-mtriple=x86_64-pc-linux-gnu",
            "-mcpu=native",
            &bc_path,
            "-o",
            &opt_bc,
            "-mem2reg",
            "-simplifycfg",
            "-instcombine",
            "-gvn",
            "-licm",
            "-loop-vectorize",
            "-slp-vectorize",
            "-inline",
            "-loop-unroll",
            "-aggressive-instcombine",
            "-coro-early",
            "-cg-sccp",
            "-ipsccp",
            "-memcpyopt",
            "-sink",
        ])
        .output();

    match opt_status {
        Ok(output) if output.status.success() => {
            println!("   ✓ LLVM optimizations complete");
        }
        Ok(output) => {
            eprintln!("   ⚠ opt warnings: {}", String::from_utf8_lossy(&output.stderr));
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
                println!("   $ gcc {} -o {}_bin -L./runtime -lviper -lm", obj_path, module_name);
                println!("   $ ./{}_bin", module_name);
            }
            Ok(output) => {
                return Err(ViperError::driver(format!(
                    "llc failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
            Err(e) => {
                return Err(ViperError::Io(e));
            }
        }
    } else {
        return Err(ViperError::driver("Optimization failed - no output"));
    }

    Ok(())
}
