mod ast;
mod cli;
mod codegen;
mod driver;
mod jit_stubs;
mod lexer;
mod parser;
mod semantic;
mod utils;

use clap::Parser;
use cli::args::{Args, Commands};
use driver::*;

fn main() {
    // Check basic prerequisites (LLVM, GCC)
    if let Err(e) = check_basic_prerequisites() {
        eprintln!("Error: {}", e);
        eprintln!();
        eprintln!("Please ensure:");
        eprintln!("  1. LLVM 20.x is installed and in PATH");
        eprintln!("  2. GCC is installed for linking AOT binaries");
        eprintln!();
        std::process::exit(1);
    }

    let args = Args::parse();

    match args.command {
        Commands::Build {
            input,
            output,
            optimize,
            lto,
            emit_llvm,
            pgo,
        } => {
            // Check runtime library for AOT compilation
            if let Err(e) = check_runtime_library() {
                eprintln!("Error: {}", e);
                eprintln!();
                eprintln!("To build the runtime library:");
                eprintln!("  cd runtime && make");
                std::process::exit(1);
            }
            if let Err(e) = compile_file_aot(
                &input,
                optimize,
                output.as_deref(),
                lto,
                emit_llvm,
                pgo.as_deref(),
            ) {
                eprintln!("Compilation failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run {
            input,
            optimize,
            lto: _,
            emit_llvm: _,
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
        Commands::Bench { file, iterations } => {
            let args = cli::bench::BenchArgs::new(file, iterations);
            if let Err(e) = cli::bench::run_bench(&args) {
                eprintln!("Bench failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Fmt { input, output } => {
            let args = cli::fmt::FmtArgs::new(input, output);
            if let Err(e) = cli::fmt::run_fmt(&args) {
                eprintln!("Format failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Lint { input, warnings } => {
            let args = cli::lint::LintArgs::new(input, warnings);
            if let Err(e) = cli::lint::run_lint(&args) {
                eprintln!("Lint failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Repl => {
            if let Err(e) = cli::repl::run_repl() {
                eprintln!("REPL error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Doc { input, output } => {
            let args = cli::doc::DocArgs::new(input, output);
            if let Err(e) = cli::doc::run_doc(&args) {
                eprintln!("Doc generation failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}
