// Semantic analysis module for Viper compiler
// Handles symbol table management and type checking

pub mod symbol_table;
pub mod type_checker;

pub use symbol_table::{SymbolTable, Symbol, SymbolKind};
pub use type_checker::TypeChecker;
