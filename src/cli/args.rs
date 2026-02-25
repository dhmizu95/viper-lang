use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "viper")]
#[command(author = "Viper Team")]
#[command(version = "0.2.3")]
#[command(about = "Viper Programming Language Compiler", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build a Viper source file
    Build {
        /// Source file to compile
        #[arg(value_name = "FILE")]
        input: String,

        /// Output file name
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,

        /// Optimization level (0, 1, 2, or 3)
        #[arg(short = 'O', default_value = "0", value_name = "LEVEL")]
        optimize: u32,

        /// Enable Link-Time Optimization
        #[arg(long)]
        lto: bool,

        /// Profile-Guided Optimization mode: "instrument" (phase 1) or "use" (phase 2)
        #[arg(long, value_name = "MODE", value_parser = parse_pgo_mode)]
        pgo: Option<String>,
    },
    /// Build and run a Viper source file
    Run {
        /// Source file to compile and run
        #[arg(value_name = "FILE")]
        input: String,

        /// Optimization level (0, 1, 2, or 3)
        #[arg(short = 'O', default_value = "0", value_name = "LEVEL")]
        optimize: u32,

        /// Enable Link-Time Optimization
        #[arg(long)]
        lto: bool,

        /// Profile-Guided Optimization mode: "instrument" (phase 1) or "use" (phase 2)
        #[arg(long, value_name = "MODE", value_parser = parse_pgo_mode)]
        pgo: Option<String>,
    },
    /// Initialize a new Viper project
    Init {
        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
    /// Show compiler information
    Info,
    /// Run benchmarks in benchmark/ directory
    Bench {
        /// Specific benchmark file to run
        #[arg(value_name = "FILE")]
        file: Option<String>,

        /// Number of iterations
        #[arg(short, long, default_value = "10")]
        iterations: u32,
    },
    /// Format and pretty-print a Viper source file
    Fmt {
        /// Source file to format
        #[arg(value_name = "FILE")]
        input: String,

        /// Output file (default: stdout)
        #[arg(short, long, value_name = "OUTPUT")]
        output: Option<String>,
    },
    /// Run semantic checks and print warnings
    Lint {
        /// Source file to lint
        #[arg(value_name = "FILE")]
        input: String,

        /// Enable all warnings
        #[arg(short, long)]
        warnings: bool,
    },
    /// Start an interactive REPL
    Repl,
    /// Generate documentation from docstrings
    Doc {
        /// Source file or directory
        #[arg(value_name = "FILE")]
        input: String,

        /// Output directory
        #[arg(short, long, default_value = "docs/generated")]
        output: String,
    },
}

fn parse_pgo_mode(s: &str) -> Result<String, String> {
    match s {
        "instrument" | "use" => Ok(s.to_string()),
        _ => Err("PGO mode must be 'instrument' or 'use'".to_string()),
    }
}
