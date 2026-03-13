//! Viper Package Manager (VPM) CLI
//!
//! A standalone package manager for Viper projects.
//! Usage: vpm <command> [options]

use clap::Parser;
use viper_lang::error::Result;

mod cli;

use cli::{args::Commands, commands::*};

#[derive(Parser, Debug)]
#[command(name = "vpm")]
#[command(author = "Viper Language Team")]
#[command(version = "0.4.5")]
#[command(about = "Viper Package Manager", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Args::parse();

    if let Err(e) = execute(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn execute(args: Args) -> Result<()> {
    match args.command {
        Commands::Init { name } => init_package(name),
        Commands::Add { package, git, branch, path } => {
            add_dependency(&package, git.as_deref(), branch.as_deref(), path.as_deref())
        }
        Commands::Remove { package } => remove_dependency(&package),
        Commands::Install { package } => install_dependencies(package.as_deref()),
        Commands::Update { package, pre } => update_dependencies(package.as_deref(), pre),
        Commands::Search { query } => search_packages(&query),
        Commands::Show { package } => show_package(&package),
        Commands::List { top_level } => list_packages(top_level),
        Commands::Publish { bump, dry_run } => publish_package(bump.as_deref(), dry_run),
        Commands::Clean => clean_cache(),
        Commands::Tree { depth } => show_tree(depth),
    }
}
