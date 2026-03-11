pub mod closure_analysis;
pub mod constant_folding;
pub mod escape_analysis;
pub mod monomorphization;
pub mod symbol_table;
pub mod type_checker;

#[allow(unused_imports)]
pub use closure_analysis::{CapturedVarInfo, ClosureAnalyzer, ClosureInfo};
#[allow(unused_imports)]
pub use constant_folding::ConstantFolder;
#[allow(unused_imports)]
pub use escape_analysis::{EscapeAnalyzer, EscapeState, FunctionEscapeContext, VariableEscapeInfo};
#[allow(unused_imports)]
pub use monomorphization::{Monomorphizer, MonomorphizedFunction};
#[allow(unused_imports)]
pub use symbol_table::{Symbol, SymbolKind, SymbolTable};
#[allow(unused_imports)]
pub use type_checker::TypeChecker;
