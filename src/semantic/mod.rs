// Semantic analysis module for Viper compiler
// Handles symbol table management and type checking

pub mod escape_analysis;
pub mod symbol_table;
pub mod type_checker;

#[allow(unused_imports)]
pub use escape_analysis::{EscapeAnalyzer, EscapeState, FunctionEscapeContext, VariableEscapeInfo};
#[allow(unused_imports)]
pub use symbol_table::{Symbol, SymbolKind, SymbolTable};
#[allow(unused_imports)]
pub use type_checker::TypeChecker;
