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
    },
    /// Build and run a Viper source file
    Run {
        /// Source file to compile and run
        #[arg(value_name = "FILE")]
        input: String,

        /// Optimization level (0, 1, 2, or 3)
        #[arg(short = 'O', default_value = "0", value_name = "LEVEL")]
        optimize: u32,
    },
    /// Initialize a new Viper project
    Init {
        /// Project name
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
    /// Show compiler information
    Info,
}
