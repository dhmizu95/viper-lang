use crate::ast::{Module, Type};
use crate::semantic::symbol_table::{Symbol, SymbolTable};
use std::collections::HashMap;

pub mod compatibility;
pub mod exprs;
pub mod hindley_milner;
pub mod infer;
pub mod overload;
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
    /// Current function's return type (for context-sensitive inference)
    pub current_return_type: Option<Type>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            errors: Vec::new(),
            expr_types: HashMap::new(),
            channel_types: HashMap::new(),
            current_return_type: None,
        }
    }

    /// Type check a complete module
    pub fn check(&mut self, module: &Module) -> Result<(), Vec<TypeError>> {
        self.errors.clear();

        // First pass: collect function declarations
        for stmt in &module.statements {
            if let crate::ast::Stmt::Function { name, params, return_type, span, type_params, .. } = stmt {
                // Normalize parameter types
                let param_types: Vec<Type> =
                    params.iter()
                        .map(|p| {
                            p.type_ann.as_ref()
                                .map(|t| self.normalize_type(t))
                                .unwrap_or(Type::Infer)
                        })
                        .collect();

                // Normalize return type (convert GenericApp Result to Type::Result)
                let normalized_return_type = return_type.as_ref().map(|t| self.normalize_type(t));

                // type_params is already Vec<String>
                let symbol = Symbol::new_function(
                    name.clone(),
                    param_types,
                    normalized_return_type,
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

    /// Normalize a type - convert GenericApp Result[T, E] to Type::Result(T, E)
    fn normalize_type(&self, ty: &Type) -> Type {
        match ty {
            Type::GenericApp { name, type_args } => {
                if name == "Result" && type_args.len() == 2 {
                    Type::Result(
                        Box::new(self.normalize_type(&type_args[0])),
                        Box::new(self.normalize_type(&type_args[1])),
                    )
                } else if name == "List" && type_args.len() == 1 {
                    Type::List(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Dict" && type_args.len() == 2 {
                    Type::Dict(
                        Box::new(self.normalize_type(&type_args[0])),
                        Box::new(self.normalize_type(&type_args[1])),
                    )
                } else if name == "Optional" && type_args.len() == 1 {
                    Type::Optional(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Future" && type_args.len() == 1 {
                    Type::Future(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Chan" && type_args.len() == 1 {
                    Type::Chan(Box::new(self.normalize_type(&type_args[0])))
                } else if name == "Array" && type_args.len() == 2 {
                    // Array[T, N] - but N should be a constant, not a type
                    // For now, just normalize the element type
                    Type::Array(
                        Box::new(self.normalize_type(&type_args[0])),
                        0,  // Size would need special handling
                    )
                } else {
                    ty.clone()
                }
            }
            // Recursively normalize nested types
            Type::List(inner) => Type::List(Box::new(self.normalize_type(inner))),
            Type::Dict(k, v) => Type::Dict(
                Box::new(self.normalize_type(k)),
                Box::new(self.normalize_type(v)),
            ),
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| self.normalize_type(t)).collect()),
            Type::Fn(params, ret) => Type::Fn(
                params.iter().map(|p| self.normalize_type(p)).collect(),
                Box::new(self.normalize_type(ret)),
            ),
            Type::Union(variants) => Type::Union(variants.iter().map(|t| self.normalize_type(t)).collect()),
            Type::Optional(inner) => Type::Optional(Box::new(self.normalize_type(inner))),
            Type::Future(inner) => Type::Future(Box::new(self.normalize_type(inner))),
            Type::Chan(inner) => Type::Chan(Box::new(self.normalize_type(inner))),
            Type::Array(elem, size) => Type::Array(
                Box::new(self.normalize_type(elem)),
                *size,
            ),
            _ => ty.clone(),
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
