//! Module System for Viper
//!
//! This module provides:
//! - Module loading and caching
//! - Import statement handling
//! - Module search path management
//! - Export tracking

pub mod loader;
pub mod registry;

pub use loader::{LoadedModule, ModuleLoader, ModuleSearchPath};
pub use registry::{ExportedSymbol, ImportedModule, ModuleRegistry};
