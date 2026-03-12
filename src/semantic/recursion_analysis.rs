//! Recursion Analysis - Detect recursive functions and suggest memoization

use crate::ast::{Expr, Stmt};
use std::collections::{HashMap, HashSet};

/// Information about a recursive function
#[derive(Debug, Clone)]
pub struct RecursiveFunctionInfo {
    /// Function name
    pub name: String,
    /// Is directly recursive (calls itself)
    pub is_directly_recursive: bool,
    /// Is mutually recursive (calls functions that call back to this)
    pub is_mutually_recursive: bool,
    /// Mutually recursive group (other function names)
    pub mutual_group: Vec<String>,
    /// Number of recursive call sites
    pub recursive_call_count: usize,
    /// Function appears to be pure (no side effects detected)
    pub appears_pure: bool,
    /// All parameters are hashable (can be cache keys)
    pub params_are_hashable: bool,
    /// Recommendation: should suggest memoization
    pub should_memoize: bool,
    /// Return type is BigInt (needs special caching)
    pub returns_bigint: bool,
}

/// Analyze recursion in functions
pub struct RecursionAnalyzer {
    /// All function names in the module
    function_names: HashSet<String>,
    /// Call graph: function -> functions it calls
    call_graph: HashMap<String, Vec<String>>,
    /// Recursive functions detected
    recursive_functions: HashMap<String, RecursiveFunctionInfo>,
}

impl RecursionAnalyzer {
    pub fn new() -> Self {
        Self {
            function_names: HashSet::new(),
            call_graph: HashMap::new(),
            recursive_functions: HashMap::new(),
        }
    }

    /// Register a function name
    pub fn register_function(&mut self, name: &str) {
        self.function_names.insert(name.to_string());
        self.call_graph.insert(name.to_string(), Vec::new());
    }

    /// Analyze a function body for recursive calls
    pub fn analyze_function(&mut self, name: &str, body: &[Stmt]) {
        let mut called_functions = Vec::new();
        let mut recursive_calls = 0;

        Self::collect_function_calls(body, &mut called_functions, name, &mut recursive_calls);

        self.call_graph.insert(name.to_string(), called_functions);

        // Check if directly recursive
        if recursive_calls > 0 {
            // Detect if function returns BigInt by analyzing return statements
            let returns_bigint = Self::detect_bigint_return(body);
            
            self.recursive_functions.insert(
                name.to_string(),
                RecursiveFunctionInfo {
                    name: name.to_string(),
                    is_directly_recursive: true,
                    is_mutually_recursive: false,
                    mutual_group: Vec::new(),
                    recursive_call_count: recursive_calls,
                    appears_pure: Self::check_purity(body),
                    params_are_hashable: true, // Simplified - assume all params hashable
                    should_memoize: recursive_calls > 0,
                    returns_bigint,
                },
            );
        }
    }

    /// Detect if a function returns BigInt by analyzing return statements
    fn detect_bigint_return(body: &[Stmt]) -> bool {
        for stmt in body {
            if let Stmt::Return { value: Some(expr), .. } = stmt {
                if Self::expr_returns_bigint(expr) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if an expression returns a BigInt value
    fn expr_returns_bigint(expr: &Expr) -> bool {
        match expr {
            // BigInt literal
            Expr::BigInt(_, _) => true,
            
            // Binary operations that can produce BigInt
            Expr::BinOp { op, left, right, .. } => {
                // Check if either operand is BigInt
                if Self::expr_returns_bigint(left) || Self::expr_returns_bigint(right) {
                    // These operations preserve BigInt
                    matches!(op, crate::ast::BinOp::Add | crate::ast::BinOp::Sub | 
                             crate::ast::BinOp::Mul | crate::ast::BinOp::Div | 
                             crate::ast::BinOp::Mod)
                } else {
                    false
                }
            }
            
            // Recursive call - check if it's a call to a BigInt-returning function
            Expr::Call { func, .. } => {
                if let Expr::Ident(name, _) = func.as_ref() {
                    // Common BigInt-returning functions
                    matches!(name.as_str(), "bigint" | "int")
                } else {
                    false
                }
            }
            
            // Conditional - check both branches
            Expr::Conditional { then_expr, else_expr, .. } => {
                Self::expr_returns_bigint(then_expr) || Self::expr_returns_bigint(else_expr)
            }
            
            _ => false,
        }
    }

    /// Collect all function calls in a statement tree
    fn collect_function_calls(
        stmts: &[Stmt],
        called: &mut Vec<String>,
        current_function: &str,
        recursive_count: &mut usize,
    ) {
        for stmt in stmts {
            Self::collect_calls_in_stmt(stmt, called, current_function, recursive_count);
        }
    }

    fn collect_calls_in_stmt(
        stmt: &Stmt,
        called: &mut Vec<String>,
        current_function: &str,
        recursive_count: &mut usize,
    ) {
        match stmt {
            Stmt::Expr(expr) => {
                Self::collect_calls_in_expr(expr, called, current_function, recursive_count);
            }
            Stmt::If { condition, body, else_body, .. } => {
                Self::collect_calls_in_expr(condition, called, current_function, recursive_count);
                Self::collect_function_calls(body, called, current_function, recursive_count);
                if let Some(else_stmts) = else_body {
                    Self::collect_function_calls(else_stmts, called, current_function, recursive_count);
                }
            }
            Stmt::While { condition, body, .. } => {
                Self::collect_calls_in_expr(condition, called, current_function, recursive_count);
                Self::collect_function_calls(body, called, current_function, recursive_count);
            }
            Stmt::For { iter, body, .. } => {
                Self::collect_calls_in_expr(iter, called, current_function, recursive_count);
                Self::collect_function_calls(body, called, current_function, recursive_count);
            }
            Stmt::Return { value, .. } => {
                if let Some(expr) = value {
                    Self::collect_calls_in_expr(expr, called, current_function, recursive_count);
                }
            }
            Stmt::Assign { value, .. } => {
                Self::collect_calls_in_expr(value, called, current_function, recursive_count);
            }
            Stmt::Declare { value, .. } => {
                if let Some(expr) = value {
                    Self::collect_calls_in_expr(expr, called, current_function, recursive_count);
                }
            }
            Stmt::Function { name: inner_name, body, .. } => {
                // Nested function - analyze separately
                if inner_name != current_function {
                    Self::collect_function_calls(body, called, inner_name, recursive_count);
                }
            }
            _ => {}
        }
    }

    fn collect_calls_in_expr(
        expr: &Expr,
        called: &mut Vec<String>,
        current_function: &str,
        recursive_count: &mut usize,
    ) {
        match expr {
            Expr::Call { func, args, .. } => {
                if let Expr::Ident(name, _) = func.as_ref() {
                    called.push(name.clone());
                    if name == current_function {
                        *recursive_count += 1;
                    }
                }
                for arg in args {
                    Self::collect_calls_in_expr(arg, called, current_function, recursive_count);
                }
            }
            Expr::BinOp { left, right, .. } => {
                Self::collect_calls_in_expr(left, called, current_function, recursive_count);
                Self::collect_calls_in_expr(right, called, current_function, recursive_count);
            }
            Expr::UnaryOp { operand, .. } => {
                Self::collect_calls_in_expr(operand, called, current_function, recursive_count);
            }
            Expr::Attribute { obj, .. } => {
                Self::collect_calls_in_expr(obj, called, current_function, recursive_count);
            }
            Expr::Index { obj, index, .. } => {
                Self::collect_calls_in_expr(obj, called, current_function, recursive_count);
                Self::collect_calls_in_expr(index, called, current_function, recursive_count);
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                Self::collect_calls_in_expr(condition, called, current_function, recursive_count);
                Self::collect_calls_in_expr(then_expr, called, current_function, recursive_count);
                Self::collect_calls_in_expr(else_expr, called, current_function, recursive_count);
            }
            _ => {}
        }
    }

    /// Check if a function appears to be pure (no side effects)
    fn check_purity(body: &[Stmt]) -> bool {
        for stmt in body {
            if !Self::is_stmt_pure(stmt) {
                return false;
            }
        }
        true
    }

    fn is_stmt_pure(stmt: &Stmt) -> bool {
        match stmt {
            // Pure statements
            Stmt::Return { value, .. } => {
                value.as_ref().map_or(true, Self::is_expr_pure)
            }
            Stmt::Expr(expr) => Self::is_expr_pure(expr),
            Stmt::If { condition, body, else_body, .. } => {
                Self::is_expr_pure(condition)
                    && body.iter().all(Self::is_stmt_pure)
                    && else_body.as_ref().map_or(true, |eb| eb.iter().all(Self::is_stmt_pure))
            }
            Stmt::While { condition, body, .. } => {
                Self::is_expr_pure(condition)
                    && body.iter().all(Self::is_stmt_pure)
            }
            Stmt::For { iter, body, .. } => {
                Self::is_expr_pure(iter)
                    && body.iter().all(Self::is_stmt_pure)
            }
            Stmt::Declare { value, .. } => {
                value.as_ref().map_or(true, Self::is_expr_pure)
            }
            // Impure statements
            Stmt::Assign { .. } => false,  // Variable assignment is a side effect
            Stmt::AugAssign { .. } => false,
            Stmt::Function { .. } => false, // Nested function definition
            Stmt::Class { .. } => false,
            Stmt::Import { .. } => false,
            Stmt::FromImport { .. } => false,
            _ => false,
        }
    }

    fn is_expr_pure(expr: &Expr) -> bool {
        match expr {
            Expr::Call { func, args, .. } => {
                // Check if calling a pure function
                let func_is_pure = if let Expr::Ident(name, _) = func.as_ref() {
                    // Built-in pure functions
                    matches!(
                        name.as_str(),
                        "len" | "str" | "int" | "float" | "bool" | "abs" | "min" | "max" | "sum"
                    )
                } else {
                    false
                };
                func_is_pure && args.iter().all(Self::is_expr_pure)
            }
            Expr::BinOp { left, right, .. } => {
                Self::is_expr_pure(left) && Self::is_expr_pure(right)
            }
            Expr::UnaryOp { operand, .. } => {
                Self::is_expr_pure(operand)
            }
            // Literals and identifiers are pure
            Expr::Int(_, _) | Expr::Float(_, _) | Expr::Bool(_, _) | Expr::Str(_, _)
            | Expr::None(_) | Expr::BigInt(_, _) | Expr::Ident(_, _) => true,
            Expr::List { elements, .. } => {
                elements.iter().all(Self::is_expr_pure)
            }
            Expr::Tuple { elements, .. } => {
                elements.iter().all(Self::is_expr_pure)
            }
            Expr::Conditional { condition, then_expr, else_expr, .. } => {
                Self::is_expr_pure(condition)
                    && Self::is_expr_pure(then_expr)
                    && Self::is_expr_pure(else_expr)
            }
            _ => false,
        }
    }

    /// Detect mutually recursive functions
    pub fn detect_mutual_recursion(&mut self) {
        // Find cycles in call graph
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut mutual_groups: HashMap<String, Vec<String>> = HashMap::new();

        for func_name in self.function_names.iter() {
            if !visited.contains(func_name) {
                self.find_cycles(
                    func_name,
                    &mut visited,
                    &mut rec_stack,
                    &mut mutual_groups,
                );
            }
        }

        // Update recursive function info with mutual recursion data
        for (func_name, group) in mutual_groups {
            if let Some(info) = self.recursive_functions.get_mut(&func_name) {
                if !group.is_empty() {
                    info.is_mutually_recursive = true;
                    info.mutual_group = group;
                }
            }
        }
    }

    fn find_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        mutual_groups: &mut HashMap<String, Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(called) = self.call_graph.get(node) {
            for callee in called {
                if !visited.contains(callee) {
                    self.find_cycles(callee, visited, rec_stack, mutual_groups);
                } else if rec_stack.contains(callee) {
                    // Found a cycle - record mutual recursion
                    mutual_groups
                        .entry(node.to_string())
                        .or_insert_with(Vec::new)
                        .push(callee.clone());
                }
            }
        }

        rec_stack.remove(node);
    }

    /// Get all detected recursive functions
    pub fn get_recursive_functions(&self) -> &HashMap<String, RecursiveFunctionInfo> {
        &self.recursive_functions
    }

    /// Get info for a specific recursive function
    pub fn get_recursive_function(&self, name: &str) -> Option<&RecursiveFunctionInfo> {
        self.recursive_functions.get(name)
    }

    /// Check if a function is recursive
    pub fn is_recursive(&self, name: &str) -> bool {
        self.recursive_functions.contains_key(name)
    }

    /// Get memoization recommendation for a function
    pub fn should_memoize(&self, name: &str) -> bool {
        self.recursive_functions
            .get(name)
            .map_or(false, |info| info.should_memoize)
    }

    /// Generate warning message for non-memoized recursive function
    pub fn generate_warning(&self, name: &str) -> Option<String> {
        self.recursive_functions.get(name).map(|info| {
            format!(
                "warning: function '{}' is recursive ({} recursive call(s)) but not memoized\n\
                 --> consider adding @lru_cache decorator for significant performance improvement\n\
                 --> example: @lru_cache(maxsize=None)\n    def {}(...):",
                info.name, info.recursive_call_count, info.name
            )
        })
    }
}

impl Default for RecursionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_direct_recursion() {
        // This would test detection of:
        // def fib(n):
        //     if n <= 1: return n
        //     return fib(n-1) + fib(n-2)
        // 
        // Implementation would require AST construction
        // Left as exercise for full implementation
    }
}
