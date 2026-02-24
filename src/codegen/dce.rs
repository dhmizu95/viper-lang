//! Dead Code Elimination (DCE) Pass for Viper Compiler
//!
//! This module implements a simple dead code elimination optimization
//! that removes unused variable declarations and computations whose
//! results are never used.

use crate::ast::{Expr, Stmt, Module};
use std::collections::{HashMap, HashSet};

/// Dead Code Elimination optimizer
pub struct DeadCodeEliminator {
    /// Track which variables are used
    used_vars: HashSet<String>,
    /// Track which statements are dead
    dead_stmts: HashSet<usize>,
    /// Variable definitions: maps statement index to variable name
    var_defs: HashMap<usize, String>,
}

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self {
            used_vars: HashSet::new(),
            dead_stmts: HashSet::new(),
            var_defs: HashMap::new(),
        }
    }

    /// Run DCE on a module
    pub fn optimize(&mut self, module: &Module) -> Module {
        self.used_vars.clear();
        self.dead_stmts.clear();
        self.var_defs.clear();

        // First pass: collect all variable definitions
        self.collect_definitions(&module.statements);

        // Second pass: find all used variables (starting from side effects)
        self.find_used_vars(&module.statements);

        // Third pass: mark dead code
        self.mark_dead_code(&module.statements);

        // Fourth pass: remove dead code
        self.remove_dead(module)
    }

    /// Collect variable definitions
    fn collect_definitions(&mut self, stmts: &[Stmt]) {
        for (idx, stmt) in stmts.iter().enumerate() {
            match stmt {
                Stmt::Declare { name, .. } => {
                    self.var_defs.insert(idx, name.clone());
                }
                Stmt::Assign { target, .. } => {
                    if let Expr::Ident(name, _) = target.as_ref() {
                        self.var_defs.insert(idx, name.clone());
                    }
                }
                _ => {}
            }
        }
    }

    /// Find variables that are actually used (have side effects or are returned)
    fn find_used_vars(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.find_used_in_stmt(stmt);
        }
    }

    fn find_used_in_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            // Expression statement - check if it's a call (side effect)
            Stmt::Expr(expr) => {
                self.mark_expr_vars(expr);
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.mark_expr_vars(expr);
                }
            }
            Stmt::If { condition, body, elif_blocks, else_body, .. } => {
                self.mark_expr_vars(condition);
                self.find_used_vars(body);
                for (_, elif_body) in elif_blocks {
                    self.find_used_vars(elif_body);
                }
                if let Some(else_body) = else_body {
                    self.find_used_vars(else_body);
                }
            }
            Stmt::While { condition, body, .. } => {
                self.mark_expr_vars(condition);
                self.find_used_vars(body);
            }
            Stmt::For { iter, body, .. } => {
                self.mark_expr_vars(iter);
                self.find_used_vars(body);
            }
            Stmt::Assign { value, .. } => {
                self.mark_expr_vars(value);
            }
            Stmt::Declare { value, .. } => {
                if let Some(val) = value {
                    self.mark_expr_vars(val);
                }
            }
            Stmt::AugAssign { target, value, .. } => {
                self.mark_expr_vars(target);
                self.mark_expr_vars(value);
            }
            _ => {}
        }
    }

    /// Mark all variables used in an expression
    fn mark_expr_vars(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name, _) => {
                self.used_vars.insert(name.clone());
            }
            Expr::BinOp { left, right, .. } => {
                self.mark_expr_vars(left);
                self.mark_expr_vars(right);
            }
            Expr::UnaryOp { operand, .. } => {
                self.mark_expr_vars(operand);
            }
            Expr::Call { func, args, .. } => {
                self.mark_expr_vars(func);
                for arg in args {
                    self.mark_expr_vars(arg);
                }
            }
            Expr::Index { obj, index, .. } => {
                self.mark_expr_vars(obj);
                self.mark_expr_vars(index);
            }
            Expr::Attribute { obj, .. } => {
                self.mark_expr_vars(obj);
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.mark_expr_vars(elem);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.mark_expr_vars(key);
                    self.mark_expr_vars(value);
                }
            }
            Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.mark_expr_vars(elem);
                }
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.mark_expr_vars(condition);
                self.mark_expr_vars(then_expr);
                self.mark_expr_vars(else_expr);
            }
            _ => {}
        }
    }

    /// Mark statements as dead if they define unused variables
    fn mark_dead_code(&mut self, stmts: &[Stmt]) {
        for (idx, stmt) in stmts.iter().enumerate() {
            match stmt {
                Stmt::Declare { name, value, .. } => {
                    // Dead if variable is never used and value has no side effects
                    if !self.used_vars.contains(name) {
                        // Check if value has side effects
                        if let Some(val) = value {
                            if !self.has_side_effects(val) {
                                self.dead_stmts.insert(idx);
                            }
                        } else {
                            self.dead_stmts.insert(idx);
                        }
                    }
                }
                Stmt::Assign { target, value, .. } => {
                    if let Expr::Ident(name, _) = target.as_ref() {
                        if !self.used_vars.contains(name) {
                            if !self.has_side_effects(value) {
                                self.dead_stmts.insert(idx);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Check if an expression has side effects (function calls, etc.)
    fn has_side_effects(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Call { .. } => true,
            Expr::BinOp { left, right, .. } => {
                self.has_side_effects(left) || self.has_side_effects(right)
            }
            Expr::UnaryOp { operand, .. } => {
                self.has_side_effects(operand)
            }
            Expr::Index { obj, index, .. } => {
                self.has_side_effects(obj) || self.has_side_effects(index)
            }
            Expr::Attribute { obj, .. } => {
                self.has_side_effects(obj)
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.has_side_effects(condition)
                    || self.has_side_effects(then_expr)
                    || self.has_side_effects(else_expr)
            }
            _ => false,
        }
    }

    /// Remove dead code from module
    fn remove_dead(&self, module: &Module) -> Module {
        let new_statements: Vec<Stmt> = module.statements
            .iter()
            .enumerate()
            .filter_map(|(idx, stmt)| {
                if self.dead_stmts.contains(&idx) {
                    None
                } else {
                    Some(stmt.clone())
                }
            })
            .collect();

        Module {
            statements: new_statements,
            span: module.span,
        }
    }
}

impl Default for DeadCodeEliminator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Stmt, Module, Type};
    use crate::utils::Span;

    fn span() -> Span {
        Span::new(0, 0, 1, 1)
    }

    #[test]
    fn test_eliminate_unused_var() {
        let mut dce = DeadCodeEliminator::new();

        let module = Module {
            statements: vec![
                // Unused variable - should be eliminated
                Stmt::Declare {
                    name: "unused".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(42, span())),
                    mutable: false,
                    span: span(),
                },
                // Used variable - should be kept
                Stmt::Declare {
                    name: "used".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(10, span())),
                    mutable: false,
                    span: span(),
                },
                // Print uses 'used' - side effect
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident("print".to_string(), span())),
                    args: vec![Expr::Ident("used".to_string(), span())],
                    span: span(),
                }),
            ],
            span: span(),
        };

        let optimized = dce.optimize(&module);

        // Should have 2 statements (unused var eliminated)
        assert_eq!(optimized.statements.len(), 2);
    }

    #[test]
    fn test_keep_side_effects() {
        let mut dce = DeadCodeEliminator::new();

        let module = Module {
            statements: vec![
                // Unused but has side effect - should be kept
                Stmt::Declare {
                    name: "x".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Call {
                        func: Box::new(Expr::Ident("get_value".to_string(), span())),
                        args: vec![],
                        span: span(),
                    }),
                    mutable: false,
                    span: span(),
                },
            ],
            span: span(),
        };

        let optimized = dce.optimize(&module);

        // Should keep the statement (has side effect)
        assert_eq!(optimized.statements.len(), 1);
    }
}
