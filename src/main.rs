use clap::Parser;
use viper_lang::cli::args::Args;
use viper_lang::cli::commands;

fn main() {
    let args = Args::parse();

    if let Err(e) = commands::execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
