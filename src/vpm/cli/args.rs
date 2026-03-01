//! Viper Package Manager (VPM) - Command Arguments
//!
//! CLI argument definitions for vpm commands.

use clap::Subcommand;

/// VPM CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new package
    Init {
        /// Package name
        #[arg(value_name = "NAME")]
        name: Option<String>,
    },
    /// Add a dependency
    Add {
        /// Package name (e.g., "requests" or "requests@1.0")
        #[arg(value_name = "PACKAGE")]
        package: String,

        /// Git repository URL
        #[arg(long, value_name = "URL")]
        git: Option<String>,

        /// Git branch
        #[arg(long, value_name = "BRANCH")]
        branch: Option<String>,

        /// Local path
        #[arg(long, value_name = "PATH")]
        path: Option<String>,
    },
    /// Remove a dependency
    Remove {
        /// Package name to remove
        #[arg(value_name = "PACKAGE")]
        package: String,
    },
    /// Install dependencies
    Install {
        /// Specific package to install
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,
    },
    /// Update dependencies
    Update {
        /// Specific package to update
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,

        /// Update to pre-release versions
        #[arg(long)]
        pre: bool,
    },
    /// Search for packages
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,
    },
    /// Show package information
    Show {
        /// Package name
        #[arg(value_name = "PACKAGE")]
        package: String,
    },
    /// List installed packages
    List {
        /// Show only direct dependencies
        #[arg(long)]
        top_level: bool,
    },
    /// Publish a package
    Publish {
        /// Bump version: major, minor, or patch
        #[arg(long, value_name = "LEVEL")]
        bump: Option<String>,

        /// Dry run (don't actually publish)
        #[arg(long)]
        dry_run: bool,
    },
    /// Clean the package cache
    Clean,
    /// Show package tree
    Tree {
        /// Maximum depth
        #[arg(long, short, default_value = "999")]
        depth: usize,
    },
}
