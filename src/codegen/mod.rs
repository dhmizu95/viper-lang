//! CodeGen module for Viper - Main coordinator
//!
//! This module provides the main code generator that translates Viper AST to LLVM IR.
//! The implementation is split across multiple domain-specific modules:
//!
//! - `types`: Type definitions and LLVM type mapping
//! - `variables`: Variable management (VarInfo, LoopContext)
//! - `expressions`: Expression code generation
//! - `statements`: Statement code generation
//! - `runtime`: Runtime function declarations
//! - `functions`: Function declaration
//! - `control_flow`: If/while/for/return handling
//! - `builder`: IR builder helpers
//! - `dce`: Dead code elimination
//! - `state`: Common state for code generation
//! - `generator`: Main CodeGen struct

mod builder;
mod control_flow;
pub mod dce;
mod expressions;
mod functions;
mod generator;
mod runtime;
pub mod state;
mod statements;
pub mod types;
pub mod variables;

pub use dce::DeadCodeEliminator;
pub use generator::CodeGen;
