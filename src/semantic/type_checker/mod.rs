use crate::ast::{Module, Type};
use crate::semantic::symbol_table::{Symbol, SymbolTable};
use std::collections::HashMap;

pub mod compatibility;
pub mod exprs;
pub mod infer;
pub mod stmts;

/// Type error with location information
#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub span: crate::utils::Span,
}

impl TypeError {
    pub fn new(message: String, span: crate::utils::Span) -> Self {
        Self { message, span }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.message, self.span)
    }
}

/// Type checker for Viper programs
pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    pub errors: Vec<TypeError>,
    /// Map from expression to inferred type
    pub expr_types: HashMap<usize, Type>,
    /// Map from channel variable name to element type (for Chan[T] inference)
    #[allow(dead_code)]
    pub channel_types: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            expr_types: HashMap::new(),
            channel_types: HashMap::new(),
        }
    }

    /// Type check a complete module
    pub fn check(&mut self, module: &Module) -> Result<(), Vec<TypeError>> {
        self.errors.clear();

        // First pass: collect function declarations
        for stmt in &module.statements {
            if let crate::ast::Stmt::Function { name, params, return_type, span, type_params, .. } = stmt {
                let param_types: Vec<Type> =
                    params.iter().map(|p| p.type_ann.clone().unwrap_or(Type::Infer)).collect();

                // type_params is already Vec<String>
                let symbol = Symbol::new_function(
                    name.clone(),
                    param_types,
                    return_type.clone(),
                    *span,
                    self.symbol_table.current_scope_id(),
                    type_params.clone(),
                );
                if let Err(e) = self.symbol_table.insert(symbol) {
                    self.errors.push(TypeError::new(e, *span));
                }
            }
        }

        // Second pass: type check all statements
        // Module-level (scope 0) assignments create immutable constants
        for stmt in &module.statements {
            self.check_stmt_module(stmt);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Get collected errors
    pub fn errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// Get the symbol table
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
