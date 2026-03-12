//! Loop Invariant Code Motion (LICM) Optimization
//!
//! This module implements Loop Invariant Code Motion, which:
//! 1. Identifies expressions in loops whose values don't change between iterations
//! 2. Moves these expressions outside the loop (to the preheader)
//! 3. Reduces redundant computations inside hot loops
//!
//! Example transformation:
//! ```python
//! # Before
//! for i in range(1000):
//!     x = a + b  # invariant - doesn't depend on i
//!     y = x * i
//!
//! # After
//! x = a + b  # hoisted out of loop
//! for i in range(1000):
//!     y = x * i
//! ```

use crate::ast::{Expr, Stmt};
use std::collections::{HashMap, HashSet};

/// Represents a loop in the AST
#[derive(Debug, Clone)]
pub struct LoopInfo {
    /// Statement index of the loop
    pub stmt_idx: usize,
    /// Variables that are modified inside the loop body
    pub modified_vars: HashSet<String>,
    /// Variables that are read inside the loop body
    pub read_vars: HashSet<String>,
    /// Expressions that are loop-invariant
    pub invariant_exprs: Vec<InvariantExpr>,
}

/// Represents a loop-invariant expression
#[derive(Debug, Clone)]
pub struct InvariantExpr {
    /// Statement index containing the invariant expression
    pub stmt_idx: usize,
    /// The invariant expression
    pub expr: Expr,
    /// Variables this expression depends on
    pub dependencies: HashSet<String>,
}

/// Loop Invariant Code Motion optimizer
pub struct LicmOptimizer {
    /// Loops found in the current function
    loops: Vec<LoopInfo>,
    /// Statements to move (stmt_idx -> new_position)
    statements_to_hoist: HashMap<usize, usize>,
    /// Track which variables are defined at each statement
    var_definitions: HashMap<String, usize>,
}

impl LicmOptimizer {
    pub fn new() -> Self {
        Self {
            loops: Vec::new(),
            statements_to_hoist: HashMap::new(),
            var_definitions: HashMap::new(),
        }
    }

    /// Run LICM optimization on a list of statements
    pub fn optimize(&mut self, stmts: &mut Vec<Stmt>) {
        self.loops.clear();
        self.statements_to_hoist.clear();
        self.var_definitions.clear();

        // First pass: find all loops and analyze them
        self.find_loops(stmts);

        // Second pass: identify loop-invariant expressions
        for loop_info in &self.loops {
            self.find_invariant_expressions(stmts, loop_info);
        }

        // Third pass: hoist invariant expressions to preheader
        if !self.statements_to_hoist.is_empty() {
            self.hoist_invariants(stmts);
        }

        // Recursively optimize nested loops and control flow
        for stmt in stmts.iter_mut() {
            self.optimize_stmt(stmt);
        }
    }

    /// Recursively optimize statements
    fn optimize_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::If { body, elif_blocks, else_body, .. } => {
                self.optimize(body);
                for (_, elif_body) in elif_blocks.iter_mut() {
                    self.optimize(elif_body);
                }
                if let Some(else_body) = else_body {
                    self.optimize(else_body);
                }
            }
            Stmt::While { body, .. } | Stmt::For { body, .. } => {
                // Recursively optimize loop body
                self.optimize(body);
            }
            _ => {}
        }
    }

    /// Find all loops in the statement list
    fn find_loops(&mut self, stmts: &[Stmt]) {
        for (idx, stmt) in stmts.iter().enumerate() {
            match stmt {
                Stmt::While { body, .. } | Stmt::For { body, .. } => {
                    let mut loop_info = LoopInfo {
                        stmt_idx: idx,
                        modified_vars: HashSet::new(),
                        read_vars: HashSet::new(),
                        invariant_exprs: Vec::new(),
                    };

                    // Analyze loop body
                    self.analyze_loop_body(body, &mut loop_info);
                    
                    // Find invariant expressions for this loop
                    let invariants = self.find_invariant_expressions(stmts, &loop_info);
                    
                    // Store loop info with invariants
                    self.loops.push(LoopInfo {
                        invariant_exprs: invariants,
                        ..loop_info
                    });
                }
                _ => {}
            }
        }
    }

    /// Analyze a loop body to find modified and read variables
    fn analyze_loop_body(&mut self, body: &[Stmt], loop_info: &mut LoopInfo) {
        for stmt in body {
            self.analyze_stmt_for_vars(stmt, loop_info);
        }
    }

    /// Analyze a statement for variable reads and writes
    fn analyze_stmt_for_vars(&mut self, stmt: &Stmt, loop_info: &mut LoopInfo) {
        match stmt {
            Stmt::Assign { target, value, .. } => {
                // Target is modified
                if let Expr::Ident(name, _) = target.as_ref() {
                    loop_info.modified_vars.insert(name.clone());
                }
                // RHS is read
                self.analyze_expr_for_vars(value, loop_info);
            }
            Stmt::Declare { name, value, .. } => {
                loop_info.modified_vars.insert(name.clone());
                if let Some(val) = value {
                    self.analyze_expr_for_vars(val, loop_info);
                }
            }
            Stmt::Expr(expr) => {
                self.analyze_expr_for_vars(expr, loop_info);
            }
            Stmt::Return { value, .. } => {
                if let Some(val) = value {
                    self.analyze_expr_for_vars(val, loop_info);
                }
            }
            Stmt::If { condition, body, elif_blocks, else_body, .. } => {
                self.analyze_expr_for_vars(condition, loop_info);
                self.analyze_loop_body(body, loop_info);
                for (_, elif_body) in elif_blocks {
                    self.analyze_loop_body(elif_body, loop_info);
                }
                if let Some(else_body) = else_body {
                    self.analyze_loop_body(else_body, loop_info);
                }
            }
            Stmt::While { condition, body, .. } => {
                self.analyze_expr_for_vars(condition, loop_info);
                // Nested loop - analyze but don't include in outer loop's vars
                for nested_stmt in body {
                    self.analyze_stmt_for_vars(nested_stmt, loop_info);
                }
            }
            Stmt::For { iter: _, body, .. } => {
                // For loops don't have a condition expression to analyze
                // Nested loop - analyze but don't include in outer loop's vars
                for nested_stmt in body {
                    self.analyze_stmt_for_vars(nested_stmt, loop_info);
                }
            }
            Stmt::AugAssign { target, value, .. } => {
                // Both target and value are read, target is modified
                self.analyze_expr_for_vars(target, loop_info);
                self.analyze_expr_for_vars(value, loop_info);
                if let Expr::Ident(name, _) = target.as_ref() {
                    loop_info.modified_vars.insert(name.clone());
                }
            }
            _ => {}
        }
    }

    /// Analyze an expression for variable reads
    fn analyze_expr_for_vars(&mut self, expr: &Expr, loop_info: &mut LoopInfo) {
        match expr {
            Expr::Ident(name, _) => {
                loop_info.read_vars.insert(name.clone());
            }
            Expr::BinOp { left, right, .. } => {
                self.analyze_expr_for_vars(left, loop_info);
                self.analyze_expr_for_vars(right, loop_info);
            }
            Expr::UnaryOp { operand, .. } => {
                self.analyze_expr_for_vars(operand, loop_info);
            }
            Expr::Call { func, args, .. } => {
                self.analyze_expr_for_vars(func, loop_info);
                for arg in args {
                    self.analyze_expr_for_vars(arg, loop_info);
                }
            }
            Expr::Index { obj, index, .. } => {
                self.analyze_expr_for_vars(obj, loop_info);
                self.analyze_expr_for_vars(index, loop_info);
            }
            Expr::Slice { obj, start, end, step, .. } => {
                self.analyze_expr_for_vars(obj, loop_info);
                if let Some(s) = start {
                    self.analyze_expr_for_vars(s, loop_info);
                }
                if let Some(e) = end {
                    self.analyze_expr_for_vars(e, loop_info);
                }
                if let Some(s) = step {
                    self.analyze_expr_for_vars(s, loop_info);
                }
            }
            Expr::Attribute { obj, .. } => {
                self.analyze_expr_for_vars(obj, loop_info);
            }
            Expr::List { elements, .. } | Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.analyze_expr_for_vars(elem, loop_info);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.analyze_expr_for_vars(key, loop_info);
                    self.analyze_expr_for_vars(value, loop_info);
                }
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.analyze_expr_for_vars(condition, loop_info);
                self.analyze_expr_for_vars(then_expr, loop_info);
                self.analyze_expr_for_vars(else_expr, loop_info);
            }
            Expr::FString(elements, _) => {
                for elem in elements {
                    self.analyze_expr_for_vars(elem, loop_info);
                }
            }
            _ => {}
        }
    }

    /// Find loop-invariant expressions
    fn find_invariant_expressions(&self, stmts: &[Stmt], loop_info: &LoopInfo) -> Vec<InvariantExpr> {
        // Build set of variables defined before this loop
        let mut vars_defined_before_loop: HashSet<String> = HashSet::new();
        for (idx, stmt) in stmts.iter().enumerate() {
            if idx >= loop_info.stmt_idx {
                break;
            }
            // Use a local method that doesn't need &mut self
            Self::collect_definitions_static(stmt, &mut vars_defined_before_loop);
        }

        // Find statements inside loop that are invariant
        let mut invariants = Vec::new();
        for (idx, stmt) in stmts.iter().enumerate() {
            if idx <= loop_info.stmt_idx {
                continue;
            }

            if let Some(invariant) = self.check_invariant(stmt, &loop_info.modified_vars, &vars_defined_before_loop) {
                invariants.push(invariant);
            }
        }
        invariants
    }

    /// Collect variable definitions from a statement (static version)
    fn collect_definitions_static(stmt: &Stmt, defined: &mut HashSet<String>) {
        match stmt {
            Stmt::Assign { target, .. } => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    defined.insert(name.clone());
                }
            }
            Stmt::Declare { name, .. } => {
                defined.insert(name.clone());
            }
            _ => {}
        }
    }

    /// Check if a statement is loop-invariant
    fn check_invariant(
        &self,
        stmt: &Stmt,
        modified_in_loop: &HashSet<String>,
        defined_before_loop: &HashSet<String>,
    ) -> Option<InvariantExpr> {
        match stmt {
            Stmt::Assign { target: _, value, .. } => {
                // Check if RHS is invariant
                let deps = self.get_expr_dependencies(value);
                if deps.iter().all(|d| !modified_in_loop.contains(d) && defined_before_loop.contains(d)) {
                    // This assignment is invariant
                    Some(InvariantExpr {
                        stmt_idx: 0, // Will be set by caller
                        expr: value.as_ref().clone(),
                        dependencies: deps,
                    })
                } else {
                    None
                }
            }
            Stmt::Declare { name: _, value, .. } => {
                if let Some(val) = value {
                    let deps = self.get_expr_dependencies(val);
                    if deps.iter().all(|d| !modified_in_loop.contains(d) && defined_before_loop.contains(d)) {
                        Some(InvariantExpr {
                            stmt_idx: 0,
                            expr: val.clone(),
                            dependencies: deps,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get all variable dependencies of an expression
    fn get_expr_dependencies(&self, expr: &Expr) -> HashSet<String> {
        let mut deps = HashSet::new();
        self.collect_deps(expr, &mut deps);
        deps
    }

    /// Recursively collect variable dependencies
    fn collect_deps(&self, expr: &Expr, deps: &mut HashSet<String>) {
        match expr {
            Expr::Ident(name, _) => {
                deps.insert(name.clone());
            }
            Expr::BinOp { left, right, .. } => {
                self.collect_deps(left, deps);
                self.collect_deps(right, deps);
            }
            Expr::UnaryOp { operand, .. } => {
                self.collect_deps(operand, deps);
            }
            Expr::Call { func, args, .. } => {
                self.collect_deps(func, deps);
                for arg in args {
                    self.collect_deps(arg, deps);
                }
            }
            Expr::Index { obj, index, .. } => {
                self.collect_deps(obj, deps);
                self.collect_deps(index, deps);
            }
            Expr::Slice { obj, start, end, step, .. } => {
                self.collect_deps(obj, deps);
                if let Some(s) = start {
                    self.collect_deps(s, deps);
                }
                if let Some(e) = end {
                    self.collect_deps(e, deps);
                }
                if let Some(s) = step {
                    self.collect_deps(s, deps);
                }
            }
            Expr::Attribute { obj, .. } => {
                self.collect_deps(obj, deps);
            }
            Expr::List { elements, .. } | Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.collect_deps(elem, deps);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.collect_deps(key, deps);
                    self.collect_deps(value, deps);
                }
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.collect_deps(condition, deps);
                self.collect_deps(then_expr, deps);
                self.collect_deps(else_expr, deps);
            }
            Expr::FString(elements, _) => {
                for elem in elements {
                    self.collect_deps(elem, deps);
                }
            }
            _ => {}
        }
    }

    /// Hoist invariant expressions to before the loop
    fn hoist_invariants(&mut self, stmts: &mut Vec<Stmt>) {
        // Sort by original position (descending) to avoid index shifting issues
        let mut to_hoist: Vec<_> = self.statements_to_hoist.iter().collect();
        to_hoist.sort_by(|a, b| b.0.cmp(a.0));

        for (&stmt_idx, &insert_pos) in to_hoist {
            if stmt_idx < stmts.len() {
                let stmt = stmts.remove(stmt_idx);
                // Adjust insert position if it's after the removed statement
                let actual_pos = if insert_pos > stmt_idx { insert_pos - 1 } else { insert_pos };
                stmts.insert(actual_pos, stmt);
            }
        }
    }
}

impl Default for LicmOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// LICM optimization pass for the AST
pub struct LicmPass {
    optimizer: LicmOptimizer,
}

impl LicmPass {
    pub fn new() -> Self {
        Self {
            optimizer: LicmOptimizer::new(),
        }
    }

    /// Run LICM optimization on a module
    pub fn run(&mut self, module: &mut crate::ast::Module) {
        self.optimizer.optimize(&mut module.statements);
    }
}

impl Default for LicmPass {
    fn default() -> Self {
        Self::new()
    }
}
