//! Core code generation module
//!
//! This module contains the main CodeGen struct and its implementation,
//! split into domain-specific submodules for better organization.

pub mod classes;
pub mod constants;
pub mod context;
pub mod functions;
pub mod module_gen;
pub mod utils;

// Re-export CodeGen struct for backward compatibility
pub use context::CodeGen;
