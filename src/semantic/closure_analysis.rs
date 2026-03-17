//! Closure Analysis for Viper Compiler
//!
//! This module implements closure analysis to determine:
//! 1. Which variables in an enclosing function are captured by nested functions
//! 2. Which variables are declared `nonlocal` in nested functions
//! 3. Building the closure environment for nested functions
//!
//! # Closure Cell Mechanism
//!
//! For variables that are shared between an enclosing function and nested functions:
//! - The enclosing function allocates a "cell" (heap-allocated box containing a pointer)
//! - The cell is passed as a hidden parameter to nested functions
//! - `nonlocal` declarations in nested functions resolve to these cells
//! - All reads/writes go through the cell pointer

use crate::ast::{Expr, Module, Stmt};
use std::collections::{HashMap, HashSet};

/// Information about a captured variable
#[derive(Debug, Clone)]
pub struct CapturedVarInfo {
    /// Name of the captured variable
    pub name: String,
    /// The function that defines the variable (enclosing function)
    pub defining_function: String,
    /// Functions that capture this variable (nested functions)
    pub captured_by: HashSet<String>,
    /// Whether the variable is declared nonlocal in any nested function
    pub is_nonlocal: bool,
    /// Whether the variable is mutated (needs write access through cell)
    pub is_mutated: bool,
}

/// Closure information for a function
#[derive(Debug, Clone, Default)]
pub struct ClosureInfo {
    /// Variables from this function that are captured by nested functions
    pub captured_vars: HashSet<String>,
    /// Variables from enclosing functions that this function captures (nonlocal)
    pub nonlocal_vars: HashSet<String>,
    /// Nested functions defined within this function
    pub nested_functions: HashSet<String>,
    /// The enclosing function (if this is a nested function)
    pub enclosing_function: Option<String>,
}

/// Main closure analyzer
pub struct ClosureAnalyzer {
    /// Closure info per function
    pub function_closures: HashMap<String, ClosureInfo>,
    /// Captured variable details
    pub captured_vars: HashMap<String, CapturedVarInfo>,
    /// Current function being analyzed
    current_function: Option<String>,
    /// Stack of enclosing functions (for nested function tracking)
    enclosing_stack: Vec<String>,
    /// Variables defined in current function
    current_func_vars: HashSet<String>,
    /// Nonlocal declarations in current function
    current_nonlocals: HashSet<String>,
}

impl ClosureAnalyzer {
    pub fn new() -> Self {
        Self {
            function_closures: HashMap::new(),
            captured_vars: HashMap::new(),
            current_function: None,
            enclosing_stack: Vec::new(),
            current_func_vars: HashSet::new(),
            current_nonlocals: HashSet::new(),
        }
    }

    /// Analyze a complete module
    pub fn analyze_module(&mut self, module: &Module) {
        // First pass: collect function definitions and their structure
        self.collect_function_structure(&module.statements, None);

        // Second pass: analyze each function for captures
        for stmt in &module.statements {
            if let Stmt::Function { name, body, .. } = stmt {
                self.analyze_function(name, body);
            }
        }
    }

    /// Collect function structure (nesting relationships)
    fn collect_function_structure(&mut self, stmts: &[Stmt], enclosing: Option<&str>) {
        for stmt in stmts {
            if let Stmt::Function { name, body, .. } = stmt {
                // Record nesting relationship
                let closure_info = self.function_closures.entry(name.clone()).or_default();
                if let Some(enc) = enclosing {
                    closure_info.enclosing_function = Some(enc.to_string());
                    closure_info.nonlocal_vars = HashSet::new();
                }

                // Record this function as nested in the enclosing function
                if let Some(enc) = enclosing {
                    let enc_info = self.function_closures.entry(enc.to_string()).or_default();
                    enc_info.nested_functions.insert(name.clone());
                }

                // Recursively process nested functions
                self.collect_function_structure(body, Some(name));
            } else {
                // Process other compound statements that may contain nested functions
                self.collect_function_structure_in_stmt(stmt, enclosing);
            }
        }
    }

    /// Collect function structure from compound statements
    fn collect_function_structure_in_stmt(&mut self, stmt: &Stmt, enclosing: Option<&str>) {
        match stmt {
            Stmt::If { body, elif_blocks, else_body, .. } => {
                self.collect_function_structure(body, enclosing);
                for (_cond, elif_body) in elif_blocks {
                    // cond is an Expr, not a Stmt - just process elif_body
                    self.collect_function_structure(elif_body, enclosing);
                }
                if let Some(else_body) = else_body {
                    self.collect_function_structure(else_body, enclosing);
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => {
                self.collect_function_structure(body, enclosing);
            }
            Stmt::Try { body, handlers, else_body, finally_body, .. } => {
                self.collect_function_structure(body, enclosing);
                for handler in handlers {
                    self.collect_function_structure(&handler.body, enclosing);
                }
                if let Some(else_body) = else_body {
                    self.collect_function_structure(else_body, enclosing);
                }
                if let Some(finally_body) = finally_body {
                    self.collect_function_structure(finally_body, enclosing);
                }
            }
            Stmt::With { body, .. } => {
                self.collect_function_structure(body, enclosing);
            }
            Stmt::Class { body, .. } => {
                // Methods inside classes - treat class as enclosing scope for methods
                self.collect_function_structure(body, enclosing);
            }
            _ => {}
        }
    }

    /// Analyze a single function for closure captures
    pub fn analyze_function(&mut self, name: &str, body: &[Stmt]) {
        self.current_function = Some(name.to_string());
        self.current_func_vars.clear();
        self.current_nonlocals.clear();

        // First pass: collect variable definitions and nonlocal declarations
        for stmt in body {
            self.collect_vars_and_nonlocals(stmt);
        }

        // Second pass: analyze captures in nested functions
        for stmt in body {
            if let Stmt::Function { name: nested_name, body: nested_body, .. } = stmt {
                self.analyze_nested_function(name, nested_name, nested_body);
            }
        }

        // Mark captured variables
        if let Some(closure_info) = self.function_closures.get_mut(name) {
            closure_info.captured_vars = self
                .captured_vars
                .iter()
                .filter(|(_, info)| info.defining_function == name)
                .map(|(name, _)| name.clone())
                .collect();
        }

        self.current_function = None;
    }

    /// Collect variable definitions and nonlocal declarations
    fn collect_vars_and_nonlocals(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Declare { name, .. } => {
                self.current_func_vars.insert(name.clone());
            }
            Stmt::Assign { target, .. } => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    self.current_func_vars.insert(name.clone());
                }
            }
            Stmt::Global { names: _, .. } => {
                // Global variables are not captured - they're module-level
            }
            Stmt::Nonlocal { names, .. } => {
                for name in names {
                    self.current_nonlocals.insert(name.clone());
                }
            }
            Stmt::For { target, .. } => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    self.current_func_vars.insert(name.clone());
                }
            }
            Stmt::Function { .. } => {
                // Nested function - handled separately
            }
            Stmt::If { body, elif_blocks, else_body, .. } => {
                for s in body {
                    self.collect_vars_and_nonlocals(s);
                }
                for (_cond, elif_body) in elif_blocks {
                    // cond is an Expr, not a Stmt - skip it, just process elif_body
                    for s in elif_body {
                        self.collect_vars_and_nonlocals(s);
                    }
                }
                if let Some(else_body) = else_body {
                    for s in else_body {
                        self.collect_vars_and_nonlocals(s);
                    }
                }
            }
            Stmt::While { body, .. } => {
                for s in body {
                    self.collect_vars_and_nonlocals(s);
                }
            }
            Stmt::Try { body, handlers, else_body, finally_body, .. } => {
                for s in body {
                    self.collect_vars_and_nonlocals(s);
                }
                for handler in handlers {
                    for s in &handler.body {
                        self.collect_vars_and_nonlocals(s);
                    }
                }
                if let Some(else_body) = else_body {
                    for s in else_body {
                        self.collect_vars_and_nonlocals(s);
                    }
                }
                if let Some(finally_body) = finally_body {
                    for s in finally_body {
                        self.collect_vars_and_nonlocals(s);
                    }
                }
            }
            _ => {}
        }
    }

    /// Analyze a nested function for captures
    fn analyze_nested_function(&mut self, enclosing_name: &str, nested_name: &str, body: &[Stmt]) {
        // Collect nonlocal declarations from the nested function
        let mut nested_nonlocals = HashSet::new();
        for stmt in body {
            if let Stmt::Nonlocal { names, .. } = stmt {
                for name in names {
                    nested_nonlocals.insert(name.clone());
                }
            }
        }

        // Store nonlocal vars in the nested function's closure info
        if let Some(closure_info) = self.function_closures.get_mut(nested_name) {
            for var_name in &nested_nonlocals {
                closure_info.nonlocal_vars.insert(var_name.clone());
            }
        }

        // Find variables used in nested function that are defined in enclosing function
        let mut used_vars = HashSet::new();
        self.collect_used_vars(body, &mut used_vars);

        // Nonlocal variables are also considered "used" even if only assigned to
        for var_name in &nested_nonlocals {
            used_vars.insert(var_name.clone());
        }

        // Check which used variables are from the enclosing scope
        for var_name in &used_vars {
            if self.current_func_vars.contains(var_name) {
                // This variable is captured from the enclosing function
                // First check mutation and nonlocal status before borrowing
                let is_mutated = self.is_var_mutated(body, var_name);
                let is_nonlocal = nested_nonlocals.contains(var_name);

                let captured_info =
                    self.captured_vars.entry(var_name.clone()).or_insert_with(|| CapturedVarInfo {
                        name: var_name.clone(),
                        defining_function: enclosing_name.to_string(),
                        captured_by: HashSet::new(),
                        is_nonlocal: false,
                        is_mutated: false,
                    });
                captured_info.captured_by.insert(nested_name.to_string());
                captured_info.is_mutated = is_mutated;
                captured_info.is_nonlocal = is_nonlocal;
            }
        }
    }

    /// Collect all variable uses in a function body
    fn collect_used_vars(&self, stmts: &[Stmt], used: &mut HashSet<String>) {
        for stmt in stmts {
            self.collect_used_vars_in_stmt(stmt, used);
        }
    }

    /// Collect variable uses from a statement
    fn collect_used_vars_in_stmt(&self, stmt: &Stmt, used: &mut HashSet<String>) {
        match stmt {
            Stmt::Expr(expr) => {
                self.collect_used_vars_in_expr(expr, used);
            }
            Stmt::Assign { target: _, value, .. } => {
                self.collect_used_vars_in_expr(value, used);
                // Target is being assigned to, not used
            }
            Stmt::AugAssign { target, value, .. } => {
                // Augmented assignment both reads and writes
                self.collect_used_vars_in_expr(target, used);
                self.collect_used_vars_in_expr(value, used);
            }
            Stmt::Declare { value, .. } => {
                if let Some(value) = value {
                    self.collect_used_vars_in_expr(value, used);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    self.collect_used_vars_in_expr(value, used);
                }
            }
            Stmt::If { condition, body, elif_blocks, else_body, .. } => {
                self.collect_used_vars_in_expr(condition, used);
                self.collect_used_vars(body, used);
                for (cond, elif_body) in elif_blocks {
                    self.collect_used_vars_in_expr(cond, used);
                    self.collect_used_vars(elif_body, used);
                }
                if let Some(else_body) = else_body {
                    self.collect_used_vars(else_body, used);
                }
            }
            Stmt::While { condition, body, .. } => {
                self.collect_used_vars_in_expr(condition, used);
                self.collect_used_vars(body, used);
            }
            Stmt::For { iter, body, .. } => {
                self.collect_used_vars_in_expr(iter, used);
                self.collect_used_vars(body, used);
            }
            Stmt::Function { body: _, .. } => {
                // Don't collect from nested function body here
                // (handled separately)
            }
            Stmt::Try { body, handlers, else_body, finally_body, .. } => {
                self.collect_used_vars(body, used);
                for handler in handlers {
                    self.collect_used_vars(&handler.body, used);
                }
                if let Some(else_body) = else_body {
                    self.collect_used_vars(else_body, used);
                }
                if let Some(finally_body) = finally_body {
                    self.collect_used_vars(finally_body, used);
                }
            }
            Stmt::With { items, body, .. } => {
                for item in items {
                    self.collect_used_vars_in_expr(&item.context_expr, used);
                }
                self.collect_used_vars(body, used);
            }
            _ => {}
        }
    }

    /// Collect variable uses from an expression
    fn collect_used_vars_in_expr(&self, expr: &Expr, used: &mut HashSet<String>) {
        match expr {
            Expr::Ident(name, _) => {
                used.insert(name.clone());
            }
            Expr::BinOp { left, right, .. } => {
                self.collect_used_vars_in_expr(left, used);
                self.collect_used_vars_in_expr(right, used);
            }
            Expr::UnaryOp { operand, .. } => {
                self.collect_used_vars_in_expr(operand, used);
            }
            Expr::Call { func, args, .. } => {
                self.collect_used_vars_in_expr(func, used);
                for arg in args {
                    self.collect_used_vars_in_expr(arg, used);
                }
            }
            Expr::Index { obj, index, .. } => {
                self.collect_used_vars_in_expr(obj, used);
                self.collect_used_vars_in_expr(index, used);
            }
            Expr::Attribute { obj, .. } => {
                self.collect_used_vars_in_expr(obj, used);
            }
            Expr::List { elements, .. }
            | Expr::Tuple { elements, .. }
            | Expr::Array { elements, .. } => {
                for elem in elements {
                    self.collect_used_vars_in_expr(elem, used);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.collect_used_vars_in_expr(key, used);
                    self.collect_used_vars_in_expr(value, used);
                }
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.collect_used_vars_in_expr(condition, used);
                self.collect_used_vars_in_expr(then_expr, used);
                self.collect_used_vars_in_expr(else_expr, used);
            }
            Expr::AssignmentExpr { target: _, value, .. } => {
                self.collect_used_vars_in_expr(value, used);
                // Target is being assigned
            }
            Expr::Lambda { body, .. } => {
                self.collect_used_vars_in_expr(body, used);
            }
            Expr::Await { future, .. } => {
                self.collect_used_vars_in_expr(future, used);
            }
            Expr::Slice { obj, start, end, step, .. } => {
                self.collect_used_vars_in_expr(obj, used);
                if let Some(start) = start {
                    self.collect_used_vars_in_expr(start, used);
                }
                if let Some(end) = end {
                    self.collect_used_vars_in_expr(end, used);
                }
                if let Some(step) = step {
                    self.collect_used_vars_in_expr(step, used);
                }
            }
            Expr::ListComprehension { iter, element, .. } => {
                self.collect_used_vars_in_expr(iter, used);
                self.collect_used_vars_in_expr(element, used);
            }
            // Literals don't use variables
            Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::Str(_, _)
            | Expr::FString(_, _)
            | Expr::FStringElement { .. }
            | Expr::Bytes(_, _)
            | Expr::BigInt(_, _)
            | Expr::Bool(_, _)
            | Expr::None(_)
            | Expr::Super(_) => {}
        }
    }

    /// Check if a variable is mutated in a function body
    fn is_var_mutated(&self, stmts: &[Stmt], var_name: &str) -> bool {
        for stmt in stmts {
            if self.is_var_mutated_in_stmt(stmt, var_name) {
                return true;
            }
        }
        false
    }

    /// Check if a variable is mutated in a statement
    fn is_var_mutated_in_stmt(&self, stmt: &Stmt, var_name: &str) -> bool {
        match stmt {
            Stmt::Assign { target, .. } => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    return name == var_name;
                }
            }
            Stmt::AugAssign { target, .. } => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    return name == var_name;
                }
            }
            Stmt::If { body, elif_blocks, else_body, .. } => {
                if self.is_var_mutated(body, var_name) {
                    return true;
                }
                for (_, elif_body) in elif_blocks {
                    if self.is_var_mutated(elif_body, var_name) {
                        return true;
                    }
                }
                if let Some(else_body) = else_body {
                    return self.is_var_mutated(else_body, var_name);
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => {
                return self.is_var_mutated(body, var_name);
            }
            Stmt::Try { body, handlers, else_body, finally_body, .. } => {
                if self.is_var_mutated(body, var_name) {
                    return true;
                }
                for handler in handlers {
                    if self.is_var_mutated(&handler.body, var_name) {
                        return true;
                    }
                }
                if let Some(else_body) = else_body {
                    return self.is_var_mutated(else_body, var_name);
                }
                if let Some(finally_body) = finally_body {
                    return self.is_var_mutated(finally_body, var_name);
                }
            }
            _ => {}
        }
        false
    }

    /// Get closure info for a function
    pub fn get_closure_info(&self, function_name: &str) -> Option<&ClosureInfo> {
        self.function_closures.get(function_name)
    }

    /// Get captured variable info
    pub fn get_captured_var_info(&self, var_name: &str) -> Option<&CapturedVarInfo> {
        self.captured_vars.get(var_name)
    }

    /// Check if a variable needs a closure cell
    pub fn needs_closure_cell(&self, function_name: &str, var_name: &str) -> bool {
        self.captured_vars.get(var_name).map_or(false, |info| {
            info.defining_function == function_name && !info.captured_by.is_empty()
        })
    }

    /// Check if a function uses nonlocal variables
    pub fn uses_nonlocal(&self, function_name: &str) -> bool {
        self.function_closures
            .get(function_name)
            .map_or(false, |info| !info.nonlocal_vars.is_empty())
    }

    /// Get the list of closure cells a function needs to receive as parameters
    pub fn get_closure_cell_params(&self, function_name: &str) -> Vec<String> {
        self.function_closures
            .get(function_name)
            .map(|info| info.nonlocal_vars.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get the list of closure cells a function needs to create
    pub fn get_closure_cells_to_create(&self, function_name: &str) -> Vec<String> {
        self.function_closures
            .get(function_name)
            .map(|info| info.captured_vars.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for ClosureAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
