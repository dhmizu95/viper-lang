//! Escape Analysis for Viper Compiler
//!
//! This module implements escape analysis to determine whether variables can be
//! stack-allocated instead of heap-allocated. The analysis tracks whether a
//! variable's value "escapes" its local scope.
//!
//! # Escape States
//!
//! - `None`: Variable does not escape, can be stack-allocated
//! - `MayEscape`: Variable might escape (conservative default)
//! - `Escapes`: Variable definitely escapes (returned, stored globally, etc.)
//!
//! # Analysis Rules
//!
//! A variable escapes if:
//! 1. It is returned from a function
//! 2. It is stored to a global variable
//! 3. It is passed to a function that may store it
//! 4. It is stored in a heap-allocated data structure
//! 5. It is captured by a closure that escapes

use crate::ast::{BinOp, Expr, Module, Stmt, Type};
use std::collections::{HashMap, HashSet};

/// Escape state for a variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EscapeState {
    /// Variable does not escape - safe for stack allocation
    None,
    /// Variable may escape - conservative estimate
    MayEscape,
    /// Variable definitely escapes - requires heap allocation
    Escapes,
}

impl EscapeState {
    /// Merge two escape states (take the more conservative one)
    pub fn merge(self, other: EscapeState) -> EscapeState {
        match (self, other) {
            (EscapeState::Escapes, _) | (_, EscapeState::Escapes) => EscapeState::Escapes,
            (EscapeState::MayEscape, _) | (_, EscapeState::MayEscape) => EscapeState::MayEscape,
            (EscapeState::None, EscapeState::None) => EscapeState::None,
        }
    }

    /// Check if this state allows stack allocation
    pub fn can_stack_allocate(&self) -> bool {
        matches!(self, EscapeState::None)
    }
}

/// Information about a variable's escape status
#[derive(Debug, Clone)]
pub struct VariableEscapeInfo {
    /// Current escape state
    pub escape_state: EscapeState,
    /// Variable type (for determining allocation strategy)
    pub var_type: Option<Type>,
    /// Whether the variable is mutable
    pub is_mutable: bool,
    /// Source location (line number)
    pub definition_line: usize,
    /// Whether this variable holds a reference type (needs ARC)
    pub is_reference_type: bool,
}

impl VariableEscapeInfo {
    pub fn new(var_type: Option<Type>, is_mutable: bool, definition_line: usize) -> Self {
        Self {
            escape_state: EscapeState::None,
            var_type,
            is_mutable,
            definition_line,
            is_reference_type: false,
        }
    }

    /// Create new info with reference type flag
    pub fn with_reference_type(mut self, is_ref: bool) -> Self {
        self.is_reference_type = is_ref;
        self
    }

    /// Check if this variable needs ARC operations
    pub fn needs_arc(&self) -> bool {
        self.is_reference_type && !self.escape_state.can_stack_allocate()
    }
}

/// Escape analysis context for a function
#[derive(Debug)]
pub struct FunctionEscapeContext {
    /// Variables tracked in this function
    pub variables: HashMap<String, VariableEscapeInfo>,
    /// Parameters that escape (may need heap allocation)
    pub escaping_params: HashSet<String>,
    /// Return value escapes (affects caller)
    pub return_escapes: bool,
    /// Variables that need ARC cleanup at function exit
    pub vars_needing_cleanup: HashSet<String>,
}

impl FunctionEscapeContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            escaping_params: HashSet::new(),
            return_escapes: false,
            vars_needing_cleanup: HashSet::new(),
        }
    }
}

impl Default for FunctionEscapeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main escape analyzer
pub struct EscapeAnalyzer {
    /// Analysis results per function
    pub function_contexts: HashMap<String, FunctionEscapeContext>,
    /// Global variables (always escape)
    pub global_vars: HashSet<String>,
    /// Current function being analyzed
    current_function: Option<String>,
    /// Track variables that are returned
    returned_vars: HashSet<String>,
}

impl EscapeAnalyzer {
    pub fn new() -> Self {
        Self {
            function_contexts: HashMap::new(),
            global_vars: HashSet::new(),
            current_function: None,
            returned_vars: HashSet::new(),
        }
    }

    /// Analyze a complete module
    pub fn analyze_module(&mut self, module: &Module) {
        // First pass: collect function definitions and global variables
        self.collect_globals(module);

        // Second pass: analyze each function
        for stmt in &module.statements {
            if let Stmt::Function { name, body, .. } = stmt {
                self.analyze_function(name, body);
            }
        }
    }

    /// Collect global variables from module
    fn collect_globals(&mut self, module: &Module) {
        for stmt in &module.statements {
            match stmt {
                Stmt::Assign { target, .. } => {
                    if let Expr::Ident(name, _) = target.as_ref() {
                        self.global_vars.insert(name.clone());
                    }
                }
                Stmt::Declare { name, .. } => {
                    self.global_vars.insert(name.clone());
                }
                _ => {}
            }
        }
    }

    /// Analyze a single function
    pub fn analyze_function(&mut self, name: &str, body: &[Stmt]) {
        self.current_function = Some(name.to_string());
        let mut ctx = FunctionEscapeContext::new();

        // Analyze statements in function body
        for stmt in body {
            self.analyze_stmt(stmt, &mut ctx);
        }

        // Mark returned variables as escaping
        for var_name in &self.returned_vars {
            if let Some(var_info) = ctx.variables.get_mut(var_name) {
                var_info.escape_state = EscapeState::Escapes;
            }
            ctx.escaping_params.insert(var_name.clone());
        }

        self.returned_vars.clear();
        self.function_contexts.insert(name.to_string(), ctx);
        self.current_function = None;
    }

    /// Analyze a statement
    fn analyze_stmt(&mut self, stmt: &Stmt, ctx: &mut FunctionEscapeContext) {
        match stmt {
            Stmt::Expr(expr) => {
                self.analyze_expr(expr, ctx, EscapeState::None);
            }
            Stmt::Assign { target, value, .. } => {
                self.analyze_assign(target, value, ctx);
            }
            Stmt::AugAssign { target, value, .. } => {
                // Augmented assignment reads and writes the variable
                self.analyze_expr(target, ctx, EscapeState::MayEscape);
                self.analyze_expr(value, ctx, EscapeState::MayEscape);
            }
            Stmt::Declare { name, value, type_ann, mutable, span } => {
                ctx.variables.insert(
                    name.clone(),
                    VariableEscapeInfo::new(type_ann.clone(), *mutable, span.line),
                );
                if let Some(val) = value {
                    self.analyze_expr(val, ctx, EscapeState::None);
                }
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    self.analyze_return_expr(expr, ctx);
                }
            }
            Stmt::If { condition, body, elif_blocks, else_body, .. } => {
                self.analyze_expr(condition, ctx, EscapeState::None);
                for stmt in body {
                    self.analyze_stmt(stmt, ctx);
                }
                for (elif_cond, elif_body) in elif_blocks {
                    self.analyze_expr(elif_cond, ctx, EscapeState::None);
                    for stmt in elif_body {
                        self.analyze_stmt(stmt, ctx);
                    }
                }
                if let Some(else_body) = else_body {
                    for stmt in else_body {
                        self.analyze_stmt(stmt, ctx);
                    }
                }
            }
            Stmt::While { condition, body, .. } => {
                self.analyze_expr(condition, ctx, EscapeState::None);
                for stmt in body {
                    self.analyze_stmt(stmt, ctx);
                }
            }
            Stmt::For { target, iter, body, .. } => {
                self.analyze_expr(iter, ctx, EscapeState::MayEscape);
                // Target variable is assigned in loop
                if let Expr::Ident(name, _) = target.as_ref() {
                    ctx.variables
                        .entry(name.clone())
                        .or_insert_with(|| VariableEscapeInfo::new(None, true, 0));
                }
                for stmt in body {
                    self.analyze_stmt(stmt, ctx);
                }
            }
            Stmt::Function { name, body, .. } => {
                // Nested function - analyze separately
                self.analyze_function(name, body);
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Pass(_) => {
                // No effect on escape analysis
            }
            Stmt::Sync { body, .. } => {
                // Sync block - conservative: assume may escape
                for stmt in body {
                    self.analyze_stmt(stmt, ctx);
                }
            }
            Stmt::Task { call, .. } => {
                // Task spawns concurrent work - conservative
                self.analyze_expr(call, ctx, EscapeState::MayEscape);
            }
            Stmt::Import { .. }
            | Stmt::FromImport { .. }
            | Stmt::Class { .. }
            | Stmt::Try { .. }
            | Stmt::Extern { .. }
            | Stmt::Struct { .. } => {
                // Handle other statement types conservatively
            }
            // Concurrency-related statements - handle conservatively
            Stmt::Chan { .. }
            | Stmt::Send { .. }
            | Stmt::Recv { .. }
            | Stmt::WaitGroup { .. }
            | Stmt::WgAdd { .. }
            | Stmt::WgDone { .. }
            | Stmt::WgWait { .. }
            | Stmt::Match { .. }
            | Stmt::Select { .. } => {
                // Conservative: assume may escape for concurrency operations
            }
        }
    }

    /// Analyze an assignment
    fn analyze_assign(&mut self, target: &Expr, value: &Expr, ctx: &mut FunctionEscapeContext) {
        match target {
            Expr::Ident(name, _) => {
                // Check if assigning to a global
                if self.global_vars.contains(name) {
                    // Value escapes to global scope
                    self.analyze_expr(value, ctx, EscapeState::Escapes);
                } else {
                    // Local assignment - analyze value
                    self.analyze_expr(value, ctx, EscapeState::None);

                    // Ensure variable exists in context
                    if !ctx.variables.contains_key(name) {
                        ctx.variables.insert(name.clone(), VariableEscapeInfo::new(None, true, 0));
                    }
                }
            }
            Expr::Index { obj, index, .. } => {
                // Indexing into a data structure - the value may escape to heap
                self.analyze_expr(obj, ctx, EscapeState::MayEscape);
                self.analyze_expr(index, ctx, EscapeState::None);
                self.analyze_expr(value, ctx, EscapeState::MayEscape);
            }
            Expr::Attribute { obj, .. } => {
                // Attribute assignment - conservative
                self.analyze_expr(obj, ctx, EscapeState::MayEscape);
                self.analyze_expr(value, ctx, EscapeState::MayEscape);
            }
            _ => {
                // Other targets - conservative
                self.analyze_expr(target, ctx, EscapeState::MayEscape);
                self.analyze_expr(value, ctx, EscapeState::MayEscape);
            }
        }
    }

    /// Analyze an expression for return (special handling)
    fn analyze_return_expr(&mut self, expr: &Expr, ctx: &mut FunctionEscapeContext) {
        match expr {
            Expr::Ident(name, _) => {
                // Returning a variable - mark as escaping
                self.returned_vars.insert(name.clone());
                if let Some(var_info) = ctx.variables.get_mut(name) {
                    var_info.escape_state = EscapeState::Escapes;
                }
            }
            Expr::List { elements, .. } => {
                // List literal being returned - elements escape
                for elem in elements {
                    self.analyze_return_expr(elem, ctx);
                }
            }
            Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.analyze_return_expr(elem, ctx);
                }
            }
            Expr::Call { func, args, .. } => {
                // Function call result being returned
                self.analyze_expr(func, ctx, EscapeState::None);
                for arg in args {
                    self.analyze_expr(arg, ctx, EscapeState::None);
                }
            }
            Expr::BinOp { left, right, .. } => {
                self.analyze_expr(left, ctx, EscapeState::None);
                self.analyze_expr(right, ctx, EscapeState::None);
            }
            Expr::UnaryOp { operand, .. } => {
                self.analyze_expr(operand, ctx, EscapeState::None);
            }
            Expr::Conditional { then_expr, else_expr, .. } => {
                self.analyze_return_expr(then_expr, ctx);
                self.analyze_return_expr(else_expr, ctx);
            }
            _ => {
                // Other expressions (literals, etc.) don't cause escapes
                self.analyze_expr(expr, ctx, EscapeState::None);
            }
        }
    }

    /// Analyze an expression
    fn analyze_expr(&mut self, expr: &Expr, ctx: &mut FunctionEscapeContext, state: EscapeState) {
        match expr {
            Expr::Int(_, _)
            | Expr::Float(_, _)
            | Expr::Str(_, _)
            | Expr::Bool(_, _)
            | Expr::None(_)
            | Expr::BigInt(_, _) => {
                // Literals don't escape
            }
            Expr::Ident(name, _) => {
                // Variable reference
                if let Some(var_info) = ctx.variables.get_mut(name) {
                    var_info.escape_state = var_info.escape_state.merge(state);
                }
                // Check if it's a parameter
                if state != EscapeState::None {
                    ctx.escaping_params.insert(name.clone());
                }
            }
            Expr::List { elements, .. } => {
                // List creation - elements may escape if list escapes
                for elem in elements {
                    self.analyze_expr(elem, ctx, state);
                }
            }
            Expr::Array { elements, .. } => {
                // Array creation - elements may escape if array escapes
                for elem in elements {
                    self.analyze_expr(elem, ctx, state);
                }
            }
            Expr::FString(elements, _) => {
                for elem in elements {
                    self.analyze_expr(elem, ctx, state);
                }
            }
            Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.analyze_expr(elem, ctx, state);
                }
            }
            Expr::Dict { pairs, .. } => {
                for (key, value) in pairs {
                    self.analyze_expr(key, ctx, state);
                    self.analyze_expr(value, ctx, state);
                }
            }
            Expr::Index { obj, index, .. } => {
                self.analyze_expr(obj, ctx, state);
                self.analyze_expr(index, ctx, EscapeState::None);
            }
            Expr::Attribute { obj, .. } => {
                self.analyze_expr(obj, ctx, state);
            }
            Expr::Await { future, .. } => {
                // Await - the future escapes
                self.analyze_expr(future, ctx, state);
            }
            Expr::BinOp { left, right, op, .. } => {
                // For membership operators, the container may be accessed
                let container_state = if matches!(op, BinOp::In | BinOp::NotIn) {
                    EscapeState::MayEscape
                } else {
                    state
                };
                self.analyze_expr(left, ctx, container_state);
                self.analyze_expr(right, ctx, state);
            }
            Expr::UnaryOp { operand, .. } => {
                self.analyze_expr(operand, ctx, state);
            }
            Expr::Call { func, args, .. } => {
                self.analyze_expr(func, ctx, EscapeState::None);
                // Arguments may escape to called function
                for arg in args {
                    self.analyze_expr(arg, ctx, EscapeState::MayEscape);
                }
            }
            Expr::Lambda { body, .. } => {
                // Lambda - conservative: assume captured vars may escape
                self.analyze_expr(body, ctx, EscapeState::MayEscape);
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                self.analyze_expr(condition, ctx, EscapeState::None);
                self.analyze_expr(then_expr, ctx, state);
                self.analyze_expr(else_expr, ctx, state);
            }
            Expr::ListComprehension { .. } => {
                // List comprehension - not yet fully implemented
            }
            Expr::Slice { obj, start, end, step, .. } => {
                self.analyze_expr(obj, ctx, state);
                if let Some(start) = start {
                    self.analyze_expr(start, ctx, state);
                }
                if let Some(end) = end {
                    self.analyze_expr(end, ctx, state);
                }
                if let Some(step) = step {
                    self.analyze_expr(step, ctx, state);
                }
            }
        }
    }

    /// Get escape info for a variable in a function
    pub fn get_variable_escape_info(
        &self,
        function_name: &str,
        var_name: &str,
    ) -> Option<&VariableEscapeInfo> {
        self.function_contexts.get(function_name).and_then(|ctx| ctx.variables.get(var_name))
    }

    /// Check if a variable can be stack-allocated
    pub fn can_stack_allocate(&self, function_name: &str, var_name: &str) -> bool {
        self.get_variable_escape_info(function_name, var_name)
            .map(|info| info.escape_state.can_stack_allocate())
            .unwrap_or(true) // Unknown variables default to stack allocation
    }

    /// Get all escaping parameters for a function
    pub fn get_escaping_params(&self, function_name: &str) -> Option<&HashSet<String>> {
        self.function_contexts.get(function_name).map(|ctx| &ctx.escaping_params)
    }

    /// Check if a function's return value escapes
    pub fn return_escapes(&self, function_name: &str) -> bool {
        self.function_contexts.get(function_name).map(|ctx| ctx.return_escapes).unwrap_or(false)
    }

    /// Check if a variable needs ARC retain/release operations
    /// Returns true if the variable is a reference type that escapes
    pub fn needs_arc(&self, function_name: &str, var_name: &str) -> bool {
        self.get_variable_escape_info(function_name, var_name)
            .map(|info| info.needs_arc())
            .unwrap_or(false)
    }

    /// Check if a variable needs ARC cleanup at function exit
    pub fn needs_arc_cleanup(&self, function_name: &str, var_name: &str) -> bool {
        self.function_contexts
            .get(function_name)
            .map(|ctx| ctx.vars_needing_cleanup.contains(var_name))
            .unwrap_or(false)
    }

    /// Get all variables that need ARC cleanup at function exit
    pub fn get_vars_needing_cleanup(&self, function_name: &str) -> Vec<&String> {
        self.function_contexts
            .get(function_name)
            .map(|ctx| ctx.vars_needing_cleanup.iter().collect())
            .unwrap_or_default()
    }

    /// Mark a variable as needing ARC cleanup at function exit
    pub fn mark_needs_cleanup(&mut self, function_name: &str, var_name: &str) {
        if let Some(ctx) = self.function_contexts.get_mut(function_name) {
            ctx.vars_needing_cleanup.insert(var_name.to_string());
        }
    }

    /// Set reference type flag for a variable
    pub fn set_reference_type(&mut self, function_name: &str, var_name: &str, is_ref: bool) {
        if let Some(ctx) = self.function_contexts.get_mut(function_name) {
            if let Some(var_info) = ctx.variables.get_mut(var_name) {
                var_info.is_reference_type = is_ref;
                // If it's a reference type and doesn't escape, it doesn't need cleanup
                if is_ref && !var_info.escape_state.can_stack_allocate() {
                    ctx.vars_needing_cleanup.insert(var_name.to_string());
                } else if !is_ref {
                    ctx.vars_needing_cleanup.remove(var_name);
                }
            }
        }
    }
}

impl Default for EscapeAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Span;

    #[test]
    fn test_escape_state_merge() {
        assert_eq!(EscapeState::None.merge(EscapeState::None), EscapeState::None);
        assert_eq!(EscapeState::None.merge(EscapeState::MayEscape), EscapeState::MayEscape);
        assert_eq!(EscapeState::None.merge(EscapeState::Escapes), EscapeState::Escapes);
        assert_eq!(EscapeState::MayEscape.merge(EscapeState::Escapes), EscapeState::Escapes);
        assert_eq!(EscapeState::Escapes.merge(EscapeState::None), EscapeState::Escapes);
    }

    #[test]
    fn test_can_stack_allocate() {
        assert!(EscapeState::None.can_stack_allocate());
        assert!(!EscapeState::MayEscape.can_stack_allocate());
        assert!(!EscapeState::Escapes.can_stack_allocate());
    }

    #[test]
    fn test_simple_function_no_escape() {
        let mut analyzer = EscapeAnalyzer::new();

        // Create a simple function: def foo(): x = 5; return x
        let body = vec![
            Stmt::Declare {
                name: "x".to_string(),
                type_ann: Some(Type::I64),
                value: Some(Expr::Int(5, Span::empty(1, 0))),
                mutable: false,
                span: Span::empty(1, 0),
            },
            Stmt::Return {
                value: Some(Expr::Ident("x".to_string(), Span::empty(2, 0))),
                span: Span::empty(2, 0),
            },
        ];

        analyzer.analyze_function("foo", &body);

        // Variable x escapes because it's returned
        let info = analyzer.get_variable_escape_info("foo", "x").unwrap();
        assert_eq!(info.escape_state, EscapeState::Escapes);
    }

    #[test]
    fn test_local_variable_no_escape() {
        let mut analyzer = EscapeAnalyzer::new();

        // Function with local variable that doesn't escape: def foo(): x = 5; y = x + 1; return y
        let body = vec![
            Stmt::Declare {
                name: "x".to_string(),
                type_ann: Some(Type::I64),
                value: Some(Expr::Int(5, Span::empty(1, 0))),
                mutable: false,
                span: Span::empty(1, 0),
            },
            Stmt::Declare {
                name: "y".to_string(),
                type_ann: Some(Type::I64),
                value: Some(Expr::BinOp {
                    left: Box::new(Expr::Ident("x".to_string(), Span::empty(2, 0))),
                    op: BinOp::Add,
                    right: Box::new(Expr::Int(1, Span::empty(2, 0))),
                    span: Span::empty(2, 0),
                }),
                mutable: false,
                span: Span::empty(2, 0),
            },
            Stmt::Return {
                value: Some(Expr::Ident("y".to_string(), Span::empty(3, 0))),
                span: Span::empty(3, 0),
            },
        ];

        analyzer.analyze_function("foo", &body);

        // x is used locally but not returned directly
        let x_info = analyzer.get_variable_escape_info("foo", "x").unwrap();
        assert_eq!(x_info.escape_state, EscapeState::None);

        // y escapes because it's returned
        let y_info = analyzer.get_variable_escape_info("foo", "y").unwrap();
        assert_eq!(y_info.escape_state, EscapeState::Escapes);
    }
}
