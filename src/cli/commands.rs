use crate::cli;
use crate::cli::args::{Args, Commands};
use crate::config::ProjectConfig;
use crate::driver::*;
use crate::error::{Result, ViperError};
use std::path::Path;

/// Load project config and merge with CLI args (CLI args take precedence)
fn load_config_with_overrides(
    input_path: &str,
    cli_auto_memoize: bool,
    cli_opt: u32,
    cli_lto: bool,
) -> (bool, u32, bool) {
    // Get the directory containing the input file
    let input_dir = Path::new(input_path).parent().unwrap_or(Path::new("."));

    // Try to load config from input directory or parents
    let config =
        ProjectConfig::load_from_dir(input_dir).or_else(|| ProjectConfig::load_from_current_dir());

    if let Some(config) = config {
        // CLI args override config, config overrides defaults
        let auto_memoize = cli_auto_memoize || config.compiler.auto_memoize;
        let opt_level = if cli_opt != 2 { cli_opt } else { config.compiler.opt_level.unwrap_or(2) };
        let lto = cli_lto || config.compiler.lto;
        (auto_memoize, opt_level, lto)
    } else {
        (cli_auto_memoize, cli_opt, cli_lto)
    }
}

pub fn execute(args: Args) -> Result<()> {
    match args.command {
        Commands::Build { input, output, optimize, lto, emit_llvm, pgo, auto_memoize } => {
            check_aot_prerequisites()?;

            // Check runtime library for AOT compilation
            if let Err(e) = check_runtime_library() {
                eprintln!("Error: {}", e);
                eprintln!();
                eprintln!("To build the runtime library:");
                eprintln!("  cd runtime && make");
                std::process::exit(1);
            }

            // Load config and merge with CLI args
            let (auto_memoize, opt_level, lto) =
                load_config_with_overrides(&input, auto_memoize, optimize, lto);

            compile_file_aot(
                &input,
                opt_level,
                output.as_deref(),
                lto,
                emit_llvm,
                pgo.as_deref(),
                auto_memoize,
            )
        }
        Commands::Run { input, optimize, auto_memoize } => {
            let current_exe = std::env::current_exe().map_err(ViperError::Io)?;

            // Load config and merge with CLI args
            let (auto_memoize, opt_level, _) =
                load_config_with_overrides(&input, auto_memoize, optimize, false);

            compile_and_run_jit_isolated(&current_exe, &input, opt_level, auto_memoize)
        }
        Commands::RunInternal { input, optimize, auto_memoize } => {
            // For internal run, also check config
            let (auto_memoize, opt_level, _) =
                load_config_with_overrides(&input, auto_memoize, optimize, false);
            compile_and_run_jit_with_memo(&input, opt_level, auto_memoize)
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
            Err(ViperError::cli("test command not available"))
        }
    }
}
