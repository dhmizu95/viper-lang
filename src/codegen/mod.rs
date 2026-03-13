pub mod builder;
pub mod closure_cells;
pub mod control_flow;
pub mod core;
pub mod dce;
pub mod expressions;
pub mod functions;
pub mod inline_lists;
pub mod licm;
pub mod oop;
pub mod runtime;
pub mod state;
pub mod statements;
pub mod types;
pub mod variables;

pub use dce::DeadCodeEliminator;
pub use licm::LicmPass;
pub use core::CodeGen;

pub(crate) type Result<T> = crate::error::Result<T>;

pub(crate) fn codegen_err(message: impl Into<String>) -> crate::error::ViperError {
    crate::error::ViperError::codegen(message)
}

pub(crate) fn codegen_error<T>(message: impl Into<String>) -> Result<T> {
    Err(codegen_err(message))
}
