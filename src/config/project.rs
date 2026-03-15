//! Project Configuration - viper.toml / vpm.toml support
//!
//! Reads configuration from viper.toml or vpm.toml in the current directory
//! to set compiler defaults like auto_memoize, optimization level, etc.

use serde::Deserialize;
use std::path::Path;

/// Project configuration from viper.toml or vpm.toml
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectConfig {
    /// Package metadata
    #[serde(default)]
    pub package: PackageConfig,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,

    /// Compiler configuration
    #[serde(default)]
    pub compiler: CompilerConfig,
}

/// Package metadata configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PackageConfig {
    pub name: Option<String>,
    pub version: Option<String>,
    pub edition: Option<String>,
}

/// Build configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BuildConfig {
    pub cargo_toml: Option<String>,
    pub runtime_dir: Option<String>,
    pub std_dir: Option<String>,
    pub binary_name: Option<String>,
    pub build_backend: Option<String>,
}

/// Compiler configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CompilerConfig {
    /// Automatically memoize pure recursive functions with exponential complexity
    #[serde(default)]
    pub auto_memoize: bool,

    /// Default optimization level (0-3)
    #[serde(default)]
    pub opt_level: Option<u32>,

    /// Enable Link-Time Optimization
    #[serde(default)]
    pub lto: bool,

    /// Enable Profile-Guided Optimization
    #[serde(default)]
    pub pgo: bool,
}

impl ProjectConfig {
    /// Load configuration from viper.toml or vpm.toml in the given directory
    pub fn load_from_dir(dir: &Path) -> Option<Self> {
        // Try viper.toml first, then vpm.toml
        let config_path = dir.join("viper.toml");
        let config_path = if config_path.exists() {
            config_path
        } else {
            let vpm_path = dir.join("vpm.toml");
            if vpm_path.exists() {
                vpm_path
            } else {
                return None;
            }
        };

        Self::load_from_path(&config_path)
    }

    /// Load configuration from a specific path
    pub fn load_from_path(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(path).ok()?;
        let config: ProjectConfig = toml::from_str(&content).ok()?;
        Some(config)
    }

    /// Load configuration from current directory or any parent directory
    pub fn load_from_current_dir() -> Option<Self> {
        std::env::current_dir().ok().and_then(|dir| {
            // Search current directory and parents for config
            for ancestor in dir.ancestors() {
                if let Some(config) = Self::load_from_dir(ancestor) {
                    return Some(config);
                }
            }
            None
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compiler_config() {
        let toml_str = r#"
[compiler]
auto_memoize = true
opt_level = 2
lto = true
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert!(config.compiler.auto_memoize);
        assert_eq!(config.compiler.opt_level, Some(2));
        assert!(config.compiler.lto);
    }

    #[test]
    fn test_parse_default_config() {
        let toml_str = r#"
[package]
name = "test-project"
version = "1.0.0"
"#;
        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.package.name, Some("test-project".to_string()));
        assert!(!config.compiler.auto_memoize);
        assert_eq!(config.compiler.opt_level, None);
    }
}
