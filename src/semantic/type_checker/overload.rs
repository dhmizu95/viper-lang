//! Function overload resolution for Viper
//!
//! This module implements overload resolution for function calls.
//! When multiple functions share the same name but have different parameter types,
//! the compiler selects the best match based on the argument types.

use crate::ast::{Expr, Type};
use crate::semantic::symbol_table::{Symbol, SymbolKind};
use crate::semantic::type_checker::TypeChecker;

impl TypeChecker {
    /// Resolve an overloaded function call
    ///
    /// Given a function name and argument expressions, finds the best matching overload.
    /// Returns the mangled name of the selected function.
    pub fn resolve_overload(&self, name: &str, args: &[Expr]) -> crate::semantic::Result<String> {
        let overloads = self.symbol_table.get_function_overloads(name);

        if overloads.is_empty() {
            return crate::semantic::semantic_error(format!("Function '{}' is not defined", name));
        }

        if overloads.len() == 1 {
            // Only one overload - use it
            if let SymbolKind::Function { mangled_name, params, .. } = &overloads[0].kind {
                // Check argument count matches
                if params.len() != args.len() {
                    return crate::semantic::semantic_error(format!(
                        "Expected {} arguments, got {}",
                        params.len(),
                        args.len()
                    ));
                }
                return Ok(mangled_name.clone());
            }
        }

        // Multiple overloads - find the best match
        let arg_types: Vec<Type> = args
            .iter()
            .map(|arg| {
                self.expr_types.get(&(arg.span().start as usize)).cloned().unwrap_or(Type::Infer)
            })
            .collect();

        let mut best_match: Option<(&Symbol, usize)> = None;
        let mut best_score = usize::MAX;

        for overload in &overloads {
            if let SymbolKind::Function { params, .. } = &overload.kind {
                // Check argument count
                if params.len() != arg_types.len() {
                    continue;
                }

                // Calculate match score
                let mut score = 0;
                let mut is_viable = true;

                for (param_type, arg_type) in params.iter().zip(arg_types.iter()) {
                    let match_score = self.type_match_score(param_type, arg_type);
                    if match_score == usize::MAX {
                        // Not compatible
                        is_viable = false;
                        break;
                    }
                    score += match_score;
                }

                if is_viable && score < best_score {
                    best_score = score;
                    best_match = Some((overload, score));
                }
            }
        }

        match best_match {
            Some((symbol, _)) => {
                if let SymbolKind::Function { mangled_name, .. } = &symbol.kind {
                    Ok(mangled_name.clone())
                } else {
                    crate::semantic::semantic_error(format!("'{}' is not a function", name))
                }
            }
            None => {
                // No viable overload found - provide helpful error
                let available: Vec<String> = overloads
                    .iter()
                    .filter_map(|s| {
                        if let SymbolKind::Function { params, .. } = &s.kind {
                            Some(format!(
                                "{}({})",
                                name,
                                params.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ")
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();

                crate::semantic::semantic_error(format!(
                    "No matching overload for '{}({})'. Available overloads: {}",
                    name,
                    arg_types.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", "),
                    available.join(", ")
                ))
            }
        }
    }

    /// Calculate match score between a parameter type and argument type
    ///
    /// Returns:
    /// - 0: Exact match
    /// - 1: Widening conversion (i64 -> f64, i64 -> Int)
    /// - 2: Narrowing conversion (Int -> i64)
    /// - 3: Compatible via type hierarchy
    /// - usize::MAX: Not compatible
    fn type_match_score(&self, param_type: &Type, arg_type: &Type) -> usize {
        // Exact match
        if param_type == arg_type {
            return 0;
        }

        // Infer matches anything with score 3 (will be resolved later)
        if matches!(param_type, Type::Infer) || matches!(arg_type, Type::Infer) {
            return 3;
        }

        // Error type matches anything (error propagation)
        if matches!(param_type, Type::Error) || matches!(arg_type, Type::Error) {
            return 3;
        }

        // Use existing is_compatible for basic compatibility check
        if !self.is_compatible(param_type, arg_type) {
            return usize::MAX;
        }

        // If compatible, determine score based on conversion kind
        match (param_type, arg_type) {
            // Widening conversions
            (Type::I64, Type::I8) | (Type::I64, Type::I16) | (Type::I64, Type::I32) => 1,
            (Type::F64, Type::F32) => 1,
            (Type::F64, Type::I64) => 1,
            (Type::Int, Type::I64) => 1,
            (Type::BigInt, Type::I64) => 1,
            (Type::Int, Type::I8) | (Type::Int, Type::I16) | (Type::Int, Type::I32) => 1,

            // Narrowing conversions
            (Type::Int, Type::BigInt) => 2,

            // List variance
            (Type::List(param_inner), Type::List(arg_inner)) => {
                self.type_match_score(param_inner, arg_inner)
            }

            // Optional: non-optional can match optional parameter
            (Type::Optional(inner), arg_type) if arg_type != &Type::None => {
                self.type_match_score(inner, arg_type)
            }

            // Union type: arg must match one of the variants
            (Type::Union(variants), _) => variants
                .iter()
                .map(|v| self.type_match_score(v, arg_type))
                .min()
                .unwrap_or(usize::MAX),

            // Arg is union: one variant must match param
            (_, Type::Union(variants)) => variants
                .iter()
                .map(|v| self.type_match_score(param_type, v))
                .min()
                .unwrap_or(usize::MAX),

            // Default for compatible types
            _ => 3,
        }
    }
}
