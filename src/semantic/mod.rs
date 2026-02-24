// Semantic analysis module for Viper compiler
// Handles symbol table management and type checking

pub mod symbol_table;
pub mod type_checker;
pub mod escape_analysis;

#[allow(unused_imports)]
pub use symbol_table::{Symbol, SymbolKind, SymbolTable};
#[allow(unused_imports)]
pub use type_checker::TypeChecker;
#[allow(unused_imports)]
pub use escape_analysis::{EscapeAnalyzer, EscapeState, VariableEscapeInfo, FunctionEscapeContext};
