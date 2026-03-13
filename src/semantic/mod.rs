pub mod closure_analysis;
pub mod constant_folding;
pub mod escape_analysis;
pub mod monomorphization;
pub mod recursion_analysis;
pub mod symbol_table;
pub mod type_checker;

pub(crate) type Result<T> = crate::error::Result<T>;

pub(crate) fn semantic_err(message: impl Into<String>) -> crate::error::ViperError {
    crate::error::ViperError::type_error(message, crate::utils::Span::default())
}

pub(crate) fn semantic_error<T>(message: impl Into<String>) -> Result<T> {
    Err(semantic_err(message))
}

// Re-export commonly used items
pub use closure_analysis::{CapturedVarInfo, ClosureAnalyzer, ClosureInfo};
pub use constant_folding::ConstantFolder;
pub use escape_analysis::{EscapeAnalyzer, EscapeState, FunctionEscapeContext, VariableEscapeInfo};
pub use monomorphization::{Monomorphizer, MonomorphizedFunction};
pub use recursion_analysis::RecursionAnalyzer;
pub use symbol_table::{Symbol, SymbolKind, SymbolTable};
pub use type_checker::TypeChecker;
