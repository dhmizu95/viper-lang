pub mod builder;
pub mod closure_cells;
pub mod control_flow;
pub mod dce;
pub mod expressions;
pub mod functions;
pub mod generator;
pub mod inline_lists;
pub mod oop;
pub mod runtime;
pub mod state;
pub mod statements;
pub mod types;
pub mod variables;

pub use dce::DeadCodeEliminator;
pub use generator::CodeGen;
