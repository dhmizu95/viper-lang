use crate::cli;
use crate::cli::args::{Args, Commands};
use crate::driver::*;

pub fn execute(args: Args) -> Result<(), String> {
    match args.command {
        Commands::Build { input, output, optimize, lto, emit_llvm, pgo } => {
            // Check runtime library for AOT compilation
            if let Err(e) = check_runtime_library() {
                eprintln!("Error: {}", e);
                eprintln!();
                eprintln!("To build the runtime library:");
                eprintln!("  cd runtime && make");
                std::process::exit(1);
            }
            compile_file_aot(&input, optimize, output.as_deref(), lto, emit_llvm, pgo.as_deref())
        }
        Commands::Run { input, optimize, lto: _, emit_llvm: _, pgo: _ } => {
            compile_and_run_jit(&input, optimize)
        }
        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| "viper_project".to_string());
            init_project(&project_name)
        }
        Commands::Info => {
            show_info();
            Ok(())
        }
        Commands::Bench { file, iterations } => {
            let args = cli::bench::BenchArgs::new(file, iterations);
            cli::bench::run_bench(&args)
        }
        Commands::Fmt { input, output } => {
            let args = cli::fmt::FmtArgs::new(input, output);
            cli::fmt::run_fmt(&args)
        }
        Commands::Lint { input, warnings } => {
            let args = cli::lint::LintArgs::new(input, warnings);
            cli::lint::run_lint(&args)
        }
        Commands::Repl => cli::repl::run_repl(),
        Commands::Doc { input, output } => {
            let args = cli::doc::DocArgs::new(input, output);
            cli::doc::run_doc(&args)
        }
        Commands::Test(test_args) => {
            cli::test::run_test_command(&test_args)
        }
    }
}
