use crate::cli;
use crate::cli::args::{Args, Commands};
use crate::driver::*;

pub fn execute(args: Args) -> Result<(), String> {
    match args.command {
        Commands::Build { input, output, optimize, lto, emit_llvm, pgo, auto_memoize: _ } => {
            check_aot_prerequisites()?;

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
        Commands::Run { input, optimize, auto_memoize } => {
            compile_and_run_jit_with_memo(&input, optimize, auto_memoize)
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
        Commands::Test { input, discover: _, verbose: _, filter: _ } => {
            eprintln!("Error: viper test is not supported yet");
            eprintln!("Input: {:?}", input);
            Err("test command not available".to_string())
        }
    }
}
