//! Core statement generation module.
//!
//! This module contains the main dispatch functions for generating LLVM IR
//! from Viper statements, as well as exception handling, import statements,
//! and context management (with statements).

mod dispatch;
mod exceptions;
mod imports;

// Re-export main dispatch functions
pub use dispatch::{generate_stmt, generate_stmt_with_closure, generate_stmt_with_escape};

// Re-export internal function for use within the statements module
pub(crate) use dispatch::generate_stmt_internal;

// Re-export exception handling functions
pub(crate) use exceptions::{generate_raise, generate_try_except};

// Re-export import and with statement handling functions
pub(crate) use imports::{
    generate_async_with, generate_from_import, generate_import, generate_sync_with,
};
