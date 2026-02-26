//! Dead Code Elimination (DCE) Pass for Viper Compiler
//!
//! This module implements dead code elimination optimization that:
//! 1. Removes unused variable declarations and computations
//! 2. Eliminates dead stores (redundant assignments)
//! 3. Uses escape analysis information for better optimization

use crate::ast::{Expr, Module, Stmt};
use std::collections::{HashMap, HashSet};

/// Tracks the last definition of a variable before it's used or redefined
#[derive(Debug, Clone)]
struct VarDef {
    /// Statement index where variable was defined
    stmt_idx: usize,
    /// Whether this definition has been used
    is_used: bool,
    /// Whether the value has side effects
    has_side_effects: bool,
}

/// Dead Code Elimination optimizer
pub struct DeadCodeEliminator {
    /// Track which variables are used
    used_vars: HashSet<String>,
    /// Track which statements are dead
    dead_stmts: HashSet<usize>,
    /// Variable definitions: maps variable name to last definition
    var_defs: HashMap<String, VarDef>,
    /// Track stores to each variable (for dead store elimination)
    var_stores: HashMap<String, Vec<usize>>,
}

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self {
            used_vars: HashSet::new(),
            dead_stmts: HashSet::new(),
            var_defs: HashMap::new(),
            var_stores: HashMap::new(),
        }
    }

    /// Run DCE on a module
    pub fn optimize(&mut self, module: &Module) -> Module {
        self.used_vars.clear();
        self.dead_stmts.clear();
        self.var_defs.clear();
        self.var_stores.clear();

        // First pass: collect all variable definitions and stores
        self.collect_definitions(&module.statements);

        // Second pass: find all used variables (backward analysis)
        self.find_used_vars_backward(&module.statements);

        // Third pass: mark dead stores (redundant assignments)
        self.mark_dead_stores(&module.statements);

        // Fourth pass: mark completely dead code
        self.mark_dead_code(&module.statements);

        // Fifth pass: remove dead code
        self.remove_dead(module)
    }

    /// Run DCE with escape analysis information
    pub fn optimize_with_escape_info(
        &mut self,
        module: &Module,
        escape_info: &HashMap<String, HashSet<String>>,
    ) -> Module {
        self.used_vars.clear();
        self.dead_stmts.clear();
        self.var_defs.clear();
        self.var_stores.clear();

        // First pass: collect all variable definitions and stores
        self.collect_definitions(&module.statements);

        // Second pass: find all used variables (backward analysis)
        self.find_used_vars_backward(&module.statements);

        // Third pass: use escape info to mark non-escaping vars as potentially dead
        self.mark_non_escaping_vars(escape_info);

        // Fourth pass: mark dead stores (redundant assignments)
        self.mark_dead_stores(&module.statements);

        // Fifth pass: mark completely dead code
        self.mark_dead_code(&module.statements);

        // Sixth pass: remove dead code
        self.remove_dead(module)
    }

    /// Collect variable definitions and track all stores
    fn collect_definitions(&mut self, stmts: &[Stmt]) {
        for (idx, stmt) in stmts.iter().enumerate() {
            match stmt {
                Stmt::Declare { name, value, .. } => {
                    let has_side_effects =
                        value.as_ref().map(|v| self.has_side_effects(v)).unwrap_or(false);
                    self.var_defs.insert(
                        name.clone(),
                        VarDef { stmt_idx: idx, is_used: false, has_side_effects },
                    );
                    self.var_stores.entry(name.clone()).or_insert_with(Vec::new).push(idx);
                }
                Stmt::Assign { target, value, .. } => {
                    if let Expr::Ident(name, _) = target.as_ref() {
                        let has_side_effects = self.has_side_effects(value);
                        self.var_defs.insert(
                            name.clone(),
                            VarDef { stmt_idx: idx, is_used: false, has_side_effects },
                        );
                        self.var_stores.entry(name.clone()).or_insert_with(Vec::new).push(idx);
                    }
                }
                _ => {}
            }
        }
    }

    /// Find variables that are actually used (backward analysis)
    fn find_used_vars_backward(&mut self, stmts: &[Stmt]) {
        // Process statements in reverse order
        for stmt in stmts.iter().rev() {
            self.find_used_in_stmt_backward(stmt);
        }
    }

    fn find_used_in_stmt_backward(&mut self, stmt: &Stmt) {
        match stmt {
            // Return statement - all variables in return value are used
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.mark_expr_vars(expr);
                }
            }
            // Expression statement - check if it's a call (side effect)
            Stmt::Expr(expr) => {
                self.mark_expr_vars(expr);
            }
            // Assignment - RHS variables are used
            Stmt::Assign { value, .. } => {
                self.mark_expr_vars(value);
            }
            Stmt::Declare { value, .. } => {
                if let Some(val) = value {
                    self.mark_expr_vars(val);
                }
            }
            Stmt::AugAssign { target, value, .. } => {
                // Both target (read) and value are used
                self.mark_expr_vars(target);
                self.mark_expr_vars(value);
            }
            // Control flow - analyze all branches
            Stmt::If { condition, body, elif_blocks, else_body, .. } => {
                self.mark_expr_vars(condition);
                self.find_used_vars_backward(body);
                for (_, elif_body) in elif_blocks {
                    self.find_used_vars_backward(elif_body);
                }
                if let Some(else_body) = else_body {
                    self.find_used_vars_backward(else_body);
                }
            }
            Stmt::While { condition, body, .. } => {
                self.mark_expr_vars(condition);
                self.find_used_vars_backward(body);
            }
            Stmt::For { iter, body, .. } => {
                self.mark_expr_vars(iter);
                self.find_used_vars_backward(body);
            }
            _ => {}
        }
    }

    /// Mark all variables used in an expression
    fn mark_expr_vars(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(name, _) => {
                self.used_vars.insert(name.clone());
                // Mark the current definition as used
                if let Some(var_def) = self.var_defs.get_mut(name) {
                    var_def.is_used = true;
                }
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
            Expr::Slice { obj, start, end, step, .. } => {
                self.mark_expr_vars(obj);
                if let Some(start) = start {
                    self.mark_expr_vars(start);
                }
                if let Some(end) = end {
                    self.mark_expr_vars(end);
                }
                if let Some(step) = step {
                    self.mark_expr_vars(step);
                }
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
            Expr::FString(elements, _) => {
                // Mark all variables in f-string elements
                for elem in elements {
                    self.mark_expr_vars(elem);
                }
            }
            _ => {}
        }
    }

    /// Mark variables that don't escape (from escape analysis) as potentially dead
    fn mark_non_escaping_vars(&mut self, escape_info: &HashMap<String, HashSet<String>>) {
        // For each function, mark non-escaping variables
        for (_func_name, non_escaping_vars) in escape_info {
            for var_name in non_escaping_vars {
                // Non-escaping variables that are never used can be eliminated
                if !self.used_vars.contains(var_name) {
                    if let Some(var_def) = self.var_defs.get(var_name) {
                        if !var_def.has_side_effects {
                            self.dead_stmts.insert(var_def.stmt_idx);
                        }
                    }
                }
            }
        }
    }

    /// Mark dead stores - stores that are overwritten before being used
    fn mark_dead_stores(&mut self, stmts: &[Stmt]) {
        // Clone store indices to avoid borrow checker issues
        let stores: Vec<_> = self.var_stores.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        // For each variable with multiple stores
        for (var_name, store_indices) in stores {
            if store_indices.len() <= 1 {
                continue; // Only check variables with multiple stores
            }

            // Check if variable is ever used
            let var_is_used = self.used_vars.contains(&var_name);

            if !var_is_used {
                // Variable is never used - all stores are dead (unless they have side effects)
                for &store_idx in &store_indices {
                    // Check if this specific store has side effects
                    let has_side_effects = stmts
                        .get(store_idx)
                        .map(|stmt| self.stmt_value_has_side_effects(stmt))
                        .unwrap_or(false);

                    if !has_side_effects {
                        self.dead_stmts.insert(store_idx);
                    }
                }
            } else {
                // Variable is used - mark stores that are overwritten before use as dead
                self.mark_redundant_stores(&var_name, &store_indices, stmts);
            }
        }
    }

    /// Check if the value being assigned in a statement has side effects
    fn stmt_value_has_side_effects(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Declare { value, .. } => {
                value.as_ref().map(|v| self.has_side_effects(v)).unwrap_or(false)
            }
            Stmt::Assign { value, .. } => self.has_side_effects(value),
            _ => false,
        }
    }

    /// Mark redundant stores that are overwritten before being read
    fn mark_redundant_stores(&mut self, var_name: &str, store_indices: &[usize], stmts: &[Stmt]) {
        // Find reads of the variable
        let mut read_indices = Vec::new();
        for (idx, stmt) in stmts.iter().enumerate() {
            if self.stmt_reads_var(stmt, var_name) {
                read_indices.push(idx);
            }
        }

        // For each store except the last one, check if it's dead
        for (i, &store_idx) in store_indices.iter().enumerate() {
            // Skip the last store - it might be the one that's used
            if i == store_indices.len() - 1 {
                continue;
            }

            // Find the next read after this store
            let next_read = read_indices.iter().find(|&&r| r > store_idx);

            // Find the next store after this store
            let next_store = store_indices.get(i + 1).copied();

            // Store is dead if next store comes before any read
            if let Some(ns) = next_store {
                let is_dead = match next_read {
                    None => true,       // No read after this store, and there's another store
                    Some(&r) => ns < r, // Next store comes before next read
                };

                if is_dead {
                    // Check if this store has side effects
                    let has_side_effects = stmts
                        .get(store_idx)
                        .map(|stmt| self.stmt_value_has_side_effects(stmt))
                        .unwrap_or(false);

                    if !has_side_effects {
                        self.dead_stmts.insert(store_idx);
                    }
                }
            }
        }
    }

    /// Check if a statement reads a variable
    fn stmt_reads_var(&self, stmt: &Stmt, var_name: &str) -> bool {
        match stmt {
            Stmt::Return { value, .. } => {
                value.as_ref().map(|e| self.expr_contains_var(e, var_name)).unwrap_or(false)
            }
            Stmt::Expr(expr) => self.expr_contains_var(expr, var_name),
            Stmt::Assign { value, .. } => self.expr_contains_var(value, var_name),
            Stmt::AugAssign { target, value, .. } => {
                // AugAssign reads the target
                self.expr_contains_var(target, var_name) || self.expr_contains_var(value, var_name)
            }
            Stmt::Declare { value, .. } => {
                value.as_ref().map(|e| self.expr_contains_var(e, var_name)).unwrap_or(false)
            }
            Stmt::If { condition, body, elif_blocks, else_body, .. } => {
                self.expr_contains_var(condition, var_name)
                    || self.block_reads_var(body, var_name)
                    || elif_blocks.iter().any(|(_, b)| self.block_reads_var(b, var_name))
                    || else_body.as_ref().map_or(false, |b| self.block_reads_var(b, var_name))
            }
            Stmt::While { condition, body, .. } => {
                self.expr_contains_var(condition, var_name) || self.block_reads_var(body, var_name)
            }
            Stmt::For { iter, body, .. } => {
                self.expr_contains_var(iter, var_name) || self.block_reads_var(body, var_name)
            }
            _ => false,
        }
    }

    /// Check if a block reads a variable
    fn block_reads_var(&self, stmts: &[Stmt], var_name: &str) -> bool {
        stmts.iter().any(|s| self.stmt_reads_var(s, var_name))
    }

    /// Check if an expression contains a variable reference
    fn expr_contains_var(&self, expr: &Expr, var_name: &str) -> bool {
        match expr {
            Expr::Ident(name, _) => name == var_name,
            Expr::BinOp { left, right, .. } => {
                self.expr_contains_var(left, var_name) || self.expr_contains_var(right, var_name)
            }
            Expr::UnaryOp { operand, .. } => self.expr_contains_var(operand, var_name),
            Expr::Call { func, args, .. } => {
                self.expr_contains_var(func, var_name)
                    || args.iter().any(|a| self.expr_contains_var(a, var_name))
            }
            Expr::Index { obj, index, .. } => {
                self.expr_contains_var(obj, var_name) || self.expr_contains_var(index, var_name)
            }
            Expr::Slice { obj, start, end, step, .. } => {
                let mut contains = self.expr_contains_var(obj, var_name);
                if let Some(start) = start {
                    contains = contains || self.expr_contains_var(start, var_name);
                }
                if let Some(end) = end {
                    contains = contains || self.expr_contains_var(end, var_name);
                }
                if let Some(step) = step {
                    contains = contains || self.expr_contains_var(step, var_name);
                }
                contains
            }
            Expr::Attribute { obj, .. } => self.expr_contains_var(obj, var_name),
            Expr::List { elements, .. } => {
                elements.iter().any(|e| self.expr_contains_var(e, var_name))
            }
            Expr::Dict { pairs, .. } => pairs.iter().any(|(k, v)| {
                self.expr_contains_var(k, var_name) || self.expr_contains_var(v, var_name)
            }),
            Expr::Tuple { elements, .. } => {
                elements.iter().any(|e| self.expr_contains_var(e, var_name))
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.expr_contains_var(condition, var_name)
                    || self.expr_contains_var(then_expr, var_name)
                    || self.expr_contains_var(else_expr, var_name)
            }
            Expr::FString(elements, _) => {
                // Check all elements in the f-string
                elements.iter().any(|e| self.expr_contains_var(e, var_name))
            }
            _ => false,
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
            Expr::UnaryOp { operand, .. } => self.has_side_effects(operand),
            Expr::Index { obj, index, .. } => {
                self.has_side_effects(obj) || self.has_side_effects(index)
            }
            Expr::Slice { obj, start, end, step, .. } => {
                let mut effects = self.has_side_effects(obj);
                if let Some(start) = start {
                    effects = effects || self.has_side_effects(start);
                }
                if let Some(end) = end {
                    effects = effects || self.has_side_effects(end);
                }
                if let Some(step) = step {
                    effects = effects || self.has_side_effects(step);
                }
                effects
            }
            Expr::Attribute { obj, .. } => self.has_side_effects(obj),
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.has_side_effects(condition)
                    || self.has_side_effects(then_expr)
                    || self.has_side_effects(else_expr)
            }
            Expr::FString(elements, _) => {
                // FString has side effects if any of its elements do
                elements.iter().any(|e| self.has_side_effects(e))
            }
            _ => false,
        }
    }

    /// Remove dead code from module
    fn remove_dead(&self, module: &Module) -> Module {
        let new_statements: Vec<Stmt> = module
            .statements
            .iter()
            .enumerate()
            .filter_map(
                |(idx, stmt)| {
                    if self.dead_stmts.contains(&idx) {
                        None
                    } else {
                        Some(stmt.clone())
                    }
                },
            )
            .collect();

        Module { statements: new_statements, span: module.span }
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
    use crate::ast::{BinOp, Expr, Module, Stmt, Type};
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

    #[test]
    fn test_dead_store_elimination() {
        let mut dce = DeadCodeEliminator::new();

        let module = Module {
            statements: vec![
                // First assignment to x - dead store
                Stmt::Declare {
                    name: "x".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(5, span())),
                    mutable: false,
                    span: span(),
                },
                // Second assignment to x - overwrites first
                Stmt::Assign {
                    target: Box::new(Expr::Ident("x".to_string(), span())),
                    value: Box::new(Expr::Int(10, span())),
                    span: span(),
                },
                // Third assignment - this is the one that's used
                Stmt::Assign {
                    target: Box::new(Expr::Ident("x".to_string(), span())),
                    value: Box::new(Expr::Int(15, span())),
                    span: span(),
                },
                // Use x
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident("print".to_string(), span())),
                    args: vec![Expr::Ident("x".to_string(), span())],
                    span: span(),
                }),
            ],
            span: span(),
        };

        let optimized = dce.optimize(&module);

        // Should eliminate the first two assignments (dead stores)
        // Keep only the last assignment and the print
        assert_eq!(optimized.statements.len(), 2);
    }

    #[test]
    fn test_dead_store_with_read_between() {
        let mut dce = DeadCodeEliminator::new();

        let module = Module {
            statements: vec![
                // First assignment to x
                Stmt::Declare {
                    name: "x".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(5, span())),
                    mutable: false,
                    span: span(),
                },
                // Read x (so first assignment is not dead)
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident("print".to_string(), span())),
                    args: vec![Expr::Ident("x".to_string(), span())],
                    span: span(),
                }),
                // Second assignment to x
                Stmt::Assign {
                    target: Box::new(Expr::Ident("x".to_string(), span())),
                    value: Box::new(Expr::Int(10, span())),
                    span: span(),
                },
                // Read x again (so second assignment is not dead)
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident("print".to_string(), span())),
                    args: vec![Expr::Ident("x".to_string(), span())],
                    span: span(),
                }),
            ],
            span: span(),
        };

        let optimized = dce.optimize(&module);

        // Should keep all statements (no dead stores)
        assert_eq!(optimized.statements.len(), 4);
    }

    #[test]
    fn test_multiple_dead_stores() {
        let mut dce = DeadCodeEliminator::new();

        let module = Module {
            statements: vec![
                // Dead store to x
                Stmt::Declare {
                    name: "x".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(1, span())),
                    mutable: false,
                    span: span(),
                },
                // Dead store to x
                Stmt::Assign {
                    target: Box::new(Expr::Ident("x".to_string(), span())),
                    value: Box::new(Expr::Int(2, span())),
                    span: span(),
                },
                // Dead store to x
                Stmt::Assign {
                    target: Box::new(Expr::Ident("x".to_string(), span())),
                    value: Box::new(Expr::Int(3, span())),
                    span: span(),
                },
                // Live store to x
                Stmt::Assign {
                    target: Box::new(Expr::Ident("x".to_string(), span())),
                    value: Box::new(Expr::Int(4, span())),
                    span: span(),
                },
                // Use x
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident("print".to_string(), span())),
                    args: vec![Expr::Ident("x".to_string(), span())],
                    span: span(),
                }),
            ],
            span: span(),
        };

        let optimized = dce.optimize(&module);

        // Should eliminate first 3 assignments (all dead stores)
        assert_eq!(optimized.statements.len(), 2);
    }

    #[test]
    fn test_optimize_with_escape_info() {
        let mut dce = DeadCodeEliminator::new();

        let module = Module {
            statements: vec![
                // Non-escaping variable (from escape analysis)
                Stmt::Declare {
                    name: "temp".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(42, span())),
                    mutable: false,
                    span: span(),
                },
                // Used variable
                Stmt::Declare {
                    name: "result".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(100, span())),
                    mutable: false,
                    span: span(),
                },
                // Use result
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident("print".to_string(), span())),
                    args: vec![Expr::Ident("result".to_string(), span())],
                    span: span(),
                }),
            ],
            span: span(),
        };

        // Escape info indicates "temp" doesn't escape and is never used
        let mut escape_info = HashMap::new();
        let mut non_escaping = HashSet::new();
        non_escaping.insert("temp".to_string());
        escape_info.insert("test_func".to_string(), non_escaping);

        let optimized = dce.optimize_with_escape_info(&module, &escape_info);

        // Should eliminate the non-escaping unused variable
        assert_eq!(optimized.statements.len(), 2);
    }

    #[test]
    fn test_augmented_assignment_not_dead() {
        let mut dce = DeadCodeEliminator::new();

        let module = Module {
            statements: vec![
                // Initialize x
                Stmt::Declare {
                    name: "x".to_string(),
                    type_ann: Some(Type::I64),
                    value: Some(Expr::Int(0, span())),
                    mutable: false,
                    span: span(),
                },
                // Augmented assignment (reads and writes x)
                Stmt::AugAssign {
                    target: Box::new(Expr::Ident("x".to_string(), span())),
                    op: BinOp::Add,
                    value: Box::new(Expr::Int(5, span())),
                    span: span(),
                },
                // Use x
                Stmt::Expr(Expr::Call {
                    func: Box::new(Expr::Ident("print".to_string(), span())),
                    args: vec![Expr::Ident("x".to_string(), span())],
                    span: span(),
                }),
            ],
            span: span(),
        };

        let optimized = dce.optimize(&module);

        // Should keep all statements (augmented assignment reads the value)
        assert_eq!(optimized.statements.len(), 3);
    }
}
