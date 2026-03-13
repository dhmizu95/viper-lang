//! Viper Package Manager (VPM)
//!
//! A standalone package manager for Viper projects.

pub mod args;
pub mod commands;

pub(crate) type Result<T> = viper_lang::error::Result<T>;

pub(crate) fn cli_err(message: impl Into<String>) -> viper_lang::error::ViperError {
    viper_lang::error::ViperError::cli(message)
}

pub(crate) fn cli_error<T>(message: impl Into<String>) -> Result<T> {
    Err(cli_err(message))
}
