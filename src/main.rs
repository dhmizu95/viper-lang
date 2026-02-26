use clap::Parser;
use viper_lang::cli::args::Args;
use viper_lang::cli::commands;
use viper_lang::driver::check_basic_prerequisites;

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

    if let Err(e) = commands::execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
