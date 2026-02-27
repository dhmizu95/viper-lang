use inkwell::context::Context;
use inkwell::targets::{InitializationConfig, Target};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr;

use crate::ast::{Expr, Stmt};
use crate::codegen::CodeGen;

pub struct ReplSession {
    chunk_counter: usize,
    // Persistent shadow store of REPL variables
    pub int_vars: HashMap<String, i64>,
    pub float_vars: HashMap<String, f64>,
    pub bool_vars: HashMap<String, bool>,
    str_vars: HashMap<String, *mut c_void>, // raw pointers for strings/lists
    // Persistent function definitions (stored as source text)
    function_sources: HashMap<String, String>,
    // Persistent variable assignments (stored as source text for re-execution)
    var_assignments: HashMap<String, String>,
}

unsafe impl Send for ReplSession {}
unsafe impl Sync for ReplSession {}

impl ReplSession {
    pub fn new() -> Self {
        Target::initialize_native(&InitializationConfig::default())
            .expect("Failed to initialize native target");

        Self {
            chunk_counter: 0,
            int_vars: HashMap::new(),
            float_vars: HashMap::new(),
            bool_vars: HashMap::new(),
            str_vars: HashMap::new(),
            function_sources: HashMap::new(),
            var_assignments: HashMap::new(),
        }
    }

    pub fn execute_chunk(&mut self, source: &str) -> Result<(), String> {
        self.chunk_counter += 1;

        // 1. Wrap source with declarations for our shadowed variables and function definitions
        // Note: We use the state from BEFORE this chunk to avoid duplicates
        let wrapped_source = self.build_wrapped_source(source);

        // 2. Tokenize and parse
        let mut lexer = crate::lexer::Lexer::new(&wrapped_source);
        let tokens = lexer.tokenize()?;

        let mut parser = crate::parser::Parser::new(tokens);
        let ast = parser.parse()?;

        // Extract variable assignments from AST to update shadow store
        // This happens BEFORE execution - we assume execution will succeed
        self.extract_assignments_from_ast(&ast);

        // Extract function definitions and variable assignments from source for FUTURE chunks
        self.extract_definitions_and_assignments(source);

        // Type checking
        let mut type_checker = crate::semantic::type_checker::TypeChecker::new();
        type_checker.check(&ast).map_err(|e| {
            format!(
                "Type errors found:\n{}",
                e.iter().map(|err| format!(" - {}", err)).collect::<Vec<_>>().join("\n")
            )
        })?;

        // 3. JIT Compile
        let context = Context::create();
        let module_name = format!("repl_chunk_{}", self.chunk_counter);

        let mut codegen = CodeGen::new(&context, &module_name);
        codegen.generate(&ast)?;
        codegen.verify()?;

        let execution_engine = codegen
            .module()
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| format!("Failed to create JIT engine: {}", e))?;

        crate::jit_stubs::register_stubs(&execution_engine, codegen.module());

        // 4. Run the code
        unsafe {
            let init_func_name = "__module_level__";
            if let Some(_func) = codegen.module().get_function(init_func_name) {
                let func_val = execution_engine
                    .get_function_value(init_func_name)
                    .map_err(|e| format!("Failed to find JIT init function: {}", e))?;

                execution_engine.run_function(func_val, &[]);
            } else if let Some(_func) = codegen.module().get_function("main") {
                let func_val = execution_engine
                    .get_function_value("main")
                    .map_err(|e| format!("Failed to find JIT main function: {}", e))?;

                execution_engine.run_function(func_val, &[]);
            }
        }

        Ok(())
    }

    /// Extract variable assignments from AST and update shadow store
    fn extract_assignments_from_ast(&mut self, ast: &crate::ast::Module) {
        for stmt in &ast.statements {
            self.extract_from_stmt(stmt);
        }
    }

    fn extract_from_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Assign { target, value, .. } => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    self.extract_value_to_shadow_store(name, value);
                }
            }
            Stmt::Declare { name, value, .. } => {
                if let Some(expr) = value {
                    self.extract_value_to_shadow_store(name, expr);
                }
            }
            // Handle other statement types if needed
            _ => {}
        }
    }

    /// Extract function definitions and variable assignments from source text
    fn extract_definitions_and_assignments(&mut self, source: &str) {
        // Extract function definitions
        self.extract_function_definitions(source);

        // Extract variable assignments (for re-execution in future chunks)
        self.extract_variable_assignments(source);
    }

    /// Extract function definitions from source text and store them
    fn extract_function_definitions(&mut self, source: &str) {
        // Simple regex-like extraction: find "def name(" patterns and extract the function
        let chars: Vec<char> = source.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            // Look for "def " at current position
            if i + 4 <= chars.len() && chars[i..i + 4].iter().collect::<String>() == "def " {
                let start = i;
                // Find the end of the function (matching indentation or end of source)
                let func_end = self.find_function_end(&chars, i);
                let func_source: String = chars[start..func_end].iter().collect();

                // Extract function name
                if let Some(name) = self.extract_function_name(&func_source) {
                    self.function_sources.insert(name.clone(), func_source);
                }

                i = func_end;
            } else {
                i += 1;
            }
        }
    }

    /// Find the end of a function definition based on indentation
    fn find_function_end(&self, chars: &[char], start: usize) -> usize {
        let mut i = start;
        let mut found_body = false;

        // First, find the colon and get the indentation of the def line
        let def_line_indent = self.get_line_indent(chars, start);

        while i < chars.len() {
            if chars[i] == ':' && !found_body {
                found_body = true;
                // Move to next line
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // Skip newline
                }
                continue;
            }

            if found_body {
                // Check indentation of current line
                let line_start = i;
                let line_indent = self.get_line_indent(chars, i);

                // Skip empty lines
                if line_indent == chars.len() - line_start
                    || chars[line_start..].iter().take_while(|&&c| c == '\n').count() > 0
                {
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1;
                    }
                    continue;
                }

                // If indentation is less than or equal to def line and not empty, function ended
                if line_indent <= def_line_indent && line_indent < chars.len() - line_start {
                    // Check if this is a new def, class, or other top-level statement
                    let rest: String = chars[line_start..].iter().take(10).collect();
                    if rest.trim().starts_with("def ")
                        || rest.trim().starts_with("class ")
                        || rest.trim().starts_with("struct ")
                        || rest.trim().starts_with("type ")
                    {
                        return line_start;
                    }
                }
            }

            i += 1;
        }

        i
    }

    /// Get indentation level of a line starting at position
    fn get_line_indent(&self, chars: &[char], start: usize) -> usize {
        let mut indent = 0;
        let mut i = start;
        while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
            indent += 1;
            i += 1;
        }
        indent
    }

    /// Extract function name from a function definition
    fn extract_function_name(&self, func_source: &str) -> Option<String> {
        // Find "def " and extract name until "("
        if let Some(def_pos) = func_source.find("def ") {
            let after_def = &func_source[def_pos + 4..];
            if let Some(paren_pos) = after_def.find('(') {
                let name = after_def[..paren_pos].trim();
                return Some(name.to_string());
            }
        }
        None
    }

    /// Extract variable assignments from source text for re-execution
    fn extract_variable_assignments(&mut self, source: &str) {
        // Simple extraction: find "name = expr" patterns at the top level (no indentation)
        let lines: Vec<&str> = source.lines().collect();
        let mut in_function = false;
        let mut function_indent = 0;

        for line in lines {
            // Calculate indentation
            let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
            let trimmed = line.trim();

            // Track function boundaries
            if trimmed.starts_with("def ") {
                in_function = true;
                function_indent = indent;
                continue;
            }

            // Check if we've exited the function (non-empty line at same or lower indent)
            if in_function
                && !trimmed.is_empty()
                && indent <= function_indent
                && !trimmed.starts_with('#')
            {
                in_function = false;
            }

            // Skip empty lines, comments, and lines inside functions
            if trimmed.is_empty() || trimmed.starts_with('#') || in_function {
                continue;
            }

            // Skip other top-level statements
            if trimmed.starts_with("def ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("return ")
                || trimmed.starts_with("print(")
                || trimmed.starts_with(":")
            {
                continue;
            }

            // Look for "name = " pattern at top level
            if let Some(eq_pos) = trimmed.find(" = ") {
                let name = trimmed[..eq_pos].trim();
                // Check if name is a valid identifier (simple check)
                if !name.is_empty() && name.chars().next().unwrap().is_alphabetic() {
                    // Store the assignment for re-execution
                    self.var_assignments.insert(name.to_string(), trimmed.to_string());
                }
            }
        }
    }

    fn extract_value_to_shadow_store(&mut self, name: &str, expr: &Expr) {
        match expr {
            Expr::Int(val, _) => {
                self.int_vars.insert(name.to_string(), *val);
            }
            Expr::Float(val, _) => {
                self.float_vars.insert(name.to_string(), *val);
            }
            Expr::Bool(val, _) => {
                self.bool_vars.insert(name.to_string(), *val);
            }
            Expr::Str(_, _) => {
                // For strings, we just track that the variable exists
                // We can't easily store the string value in the shadow store
                self.str_vars.insert(name.to_string(), ptr::null_mut());
            }
            Expr::List { .. } | Expr::ListComprehension { .. } => {
                // List type, just track existence
                self.str_vars.insert(name.to_string(), ptr::null_mut());
            }
            // For expressions we can't statically evaluate, we still need to track the variable
            // but we'll use a default value. The actual value will be used during execution.
            Expr::BinOp { left, op, right, .. } => {
                // Try to evaluate simple binary operations
                self.extract_binop_to_shadow(name, op, left.as_ref(), right.as_ref());
            }
            Expr::UnaryOp { op, operand, .. } => {
                self.extract_unaryop_to_shadow(name, op, operand.as_ref());
            }
            Expr::Call { .. } => {
                // Function call - don't insert a placeholder.
                // The variable will be set during execution and extracted after.
                // We skip this to avoid overriding the actual value with a placeholder.
            }
            Expr::Ident(other, _) => {
                // Assignment from another variable - copy the value from shadow store
                if let Some(val) = self.int_vars.get(other) {
                    self.int_vars.insert(name.to_string(), *val);
                } else if let Some(val) = self.float_vars.get(other) {
                    self.float_vars.insert(name.to_string(), *val);
                } else if let Some(val) = self.bool_vars.get(other) {
                    self.bool_vars.insert(name.to_string(), *val);
                }
            }
            // For other expression types, insert a placeholder
            _ => {
                // Default to int 0 for unknown types
                self.int_vars.insert(name.to_string(), 0);
            }
        }
    }

    fn extract_binop_to_shadow(
        &mut self,
        name: &str,
        op: &crate::ast::BinOp,
        left: &Expr,
        right: &Expr,
    ) {
        // Recursively evaluate left and right operands
        let left_val = self.evaluate_expr_int(left);
        let right_val = self.evaluate_expr_int(right);

        if let (Some(l), Some(r)) = (left_val, right_val) {
            let result = match op {
                crate::ast::BinOp::Add => l + r,
                crate::ast::BinOp::Sub => l - r,
                crate::ast::BinOp::Mul => l * r,
                crate::ast::BinOp::Div => {
                    if r != 0 {
                        l / r
                    } else {
                        0
                    }
                }
                crate::ast::BinOp::Mod => {
                    if r != 0 {
                        l % r
                    } else {
                        0
                    }
                }
                crate::ast::BinOp::Pow => l.pow(r as u32),
                _ => l,
            };
            self.int_vars.insert(name.to_string(), result);
            return;
        }

        // Try float operations
        let left_float = self.evaluate_expr_float(left);
        let right_float = self.evaluate_expr_float(right);

        if let (Some(l), Some(r)) = (left_float, right_float) {
            let result = match op {
                crate::ast::BinOp::Add => l + r,
                crate::ast::BinOp::Sub => l - r,
                crate::ast::BinOp::Mul => l * r,
                crate::ast::BinOp::Div => {
                    if r != 0.0 {
                        l / r
                    } else {
                        0.0
                    }
                }
                _ => l,
            };
            self.float_vars.insert(name.to_string(), result);
            return;
        }

        // Mixed int/float - promote to float
        if let Some(l) = self.evaluate_expr_int(left) {
            if let Some(r) = self.evaluate_expr_float(right) {
                let result = match op {
                    crate::ast::BinOp::Add => l as f64 + r,
                    crate::ast::BinOp::Sub => l as f64 - r,
                    crate::ast::BinOp::Mul => l as f64 * r,
                    crate::ast::BinOp::Div => {
                        if r != 0.0 {
                            l as f64 / r
                        } else {
                            0.0
                        }
                    }
                    _ => l as f64,
                };
                self.float_vars.insert(name.to_string(), result);
                return;
            }
        }
        if let Some(l) = self.evaluate_expr_float(left) {
            if let Some(r) = self.evaluate_expr_int(right) {
                let result = match op {
                    crate::ast::BinOp::Add => l + r as f64,
                    crate::ast::BinOp::Sub => l - r as f64,
                    crate::ast::BinOp::Mul => l * r as f64,
                    crate::ast::BinOp::Div => {
                        if r != 0 {
                            l / r as f64
                        } else {
                            0.0
                        }
                    }
                    _ => l,
                };
                self.float_vars.insert(name.to_string(), result);
                return;
            }
        }

        // Can't statically evaluate, use placeholder
        self.int_vars.insert(name.to_string(), 0);
    }

    /// Recursively evaluate expression to int value
    fn evaluate_expr_int(&self, expr: &Expr) -> Option<i64> {
        match expr {
            Expr::Int(val, _) => Some(*val),
            Expr::Ident(var_name, _) => self.int_vars.get(var_name).copied(),
            Expr::BinOp { left, op, right, .. } => {
                let l = self.evaluate_expr_int(left)?;
                let r = self.evaluate_expr_int(right)?;
                match op {
                    crate::ast::BinOp::Add => Some(l + r),
                    crate::ast::BinOp::Sub => Some(l - r),
                    crate::ast::BinOp::Mul => Some(l * r),
                    crate::ast::BinOp::Div => {
                        if r != 0 {
                            Some(l / r)
                        } else {
                            Some(0)
                        }
                    }
                    crate::ast::BinOp::Mod => {
                        if r != 0 {
                            Some(l % r)
                        } else {
                            Some(0)
                        }
                    }
                    _ => Some(l),
                }
            }
            _ => None,
        }
    }

    /// Recursively evaluate expression to float value
    fn evaluate_expr_float(&self, expr: &Expr) -> Option<f64> {
        match expr {
            Expr::Float(val, _) => Some(*val),
            Expr::Ident(var_name, _) => {
                // Try float vars first, then int vars (promoted to float)
                if let Some(v) = self.float_vars.get(var_name) {
                    Some(*v)
                } else {
                    self.int_vars.get(var_name).map(|&i| i as f64)
                }
            }
            Expr::Int(val, _) => Some(*val as f64),
            Expr::BinOp { left, op, right, .. } => {
                // Try float operations first
                if let (Some(l), Some(r)) =
                    (self.evaluate_expr_float(left), self.evaluate_expr_float(right))
                {
                    return match op {
                        crate::ast::BinOp::Add => Some(l + r),
                        crate::ast::BinOp::Sub => Some(l - r),
                        crate::ast::BinOp::Mul => Some(l * r),
                        crate::ast::BinOp::Div => {
                            if r != 0.0 {
                                Some(l / r)
                            } else {
                                Some(0.0)
                            }
                        }
                        _ => Some(l),
                    };
                }
                // Try mixed int/float
                if let (Some(l), Some(r)) =
                    (self.evaluate_expr_int(left), self.evaluate_expr_float(right))
                {
                    return match op {
                        crate::ast::BinOp::Add => Some(l as f64 + r),
                        crate::ast::BinOp::Sub => Some(l as f64 - r),
                        crate::ast::BinOp::Mul => Some(l as f64 * r),
                        crate::ast::BinOp::Div => {
                            if r != 0.0 {
                                Some(l as f64 / r)
                            } else {
                                Some(0.0)
                            }
                        }
                        _ => Some(l as f64),
                    };
                }
                if let (Some(l), Some(r)) =
                    (self.evaluate_expr_float(left), self.evaluate_expr_int(right))
                {
                    return match op {
                        crate::ast::BinOp::Add => Some(l + r as f64),
                        crate::ast::BinOp::Sub => Some(l - r as f64),
                        crate::ast::BinOp::Mul => Some(l * r as f64),
                        crate::ast::BinOp::Div => {
                            if r != 0 {
                                Some(l / r as f64)
                            } else {
                                Some(0.0)
                            }
                        }
                        _ => Some(l),
                    };
                }
                None
            }
            _ => None,
        }
    }

    /// Get integer value from expression (handles literals and variable references)
    fn get_expr_int_value(&self, expr: &Expr) -> Option<i64> {
        match expr {
            Expr::Int(val, _) => Some(*val),
            Expr::Ident(var_name, _) => self.int_vars.get(var_name).copied(),
            _ => None,
        }
    }

    /// Get float value from expression (handles literals and variable references)
    fn get_expr_float_value(&self, expr: &Expr) -> Option<f64> {
        match expr {
            Expr::Float(val, _) => Some(*val),
            Expr::Ident(var_name, _) => self.float_vars.get(var_name).copied(),
            _ => None,
        }
    }

    fn extract_unaryop_to_shadow(&mut self, name: &str, op: &crate::ast::UnaryOp, operand: &Expr) {
        match op {
            crate::ast::UnaryOp::Neg => {
                if let Expr::Int(val, _) = operand {
                    self.int_vars.insert(name.to_string(), -val);
                } else if let Expr::Float(val, _) = operand {
                    self.float_vars.insert(name.to_string(), -val);
                }
            }
            crate::ast::UnaryOp::Not => {
                if let Expr::Bool(val, _) = operand {
                    self.bool_vars.insert(name.to_string(), !val);
                }
            }
            // Other unary ops (Pos, Invert, Inc/Dec) - just use operand value
            _ => {
                // For other unary operations, copy operand value if it's a literal
                if let Expr::Int(val, _) = operand {
                    self.int_vars.insert(name.to_string(), *val);
                } else if let Expr::Float(val, _) = operand {
                    self.float_vars.insert(name.to_string(), *val);
                }
            }
        }
    }

    fn build_wrapped_source(&self, source: &str) -> String {
        // Prepend user's code with earlier variable state and function definitions
        let mut preamble = String::new();

        // First, include all function definitions
        for (_name, func_source) in &self.function_sources {
            preamble.push_str(func_source);
            preamble.push('\n');
        }

        // Then, include variable assignments for re-execution
        // These need to come after function definitions but before literal values
        for (_name, assignment) in &self.var_assignments {
            preamble.push_str(assignment);
            preamble.push('\n');
        }

        // Finally, include literal variable state (for values that weren't from assignments)
        for (name, val) in &self.int_vars {
            // Skip if this variable has an assignment (will be re-executed)
            if !self.var_assignments.contains_key(name) {
                preamble.push_str(&format!("{} = {}\n", name, val));
            }
        }
        for (name, val) in &self.float_vars {
            if !self.var_assignments.contains_key(name) {
                preamble.push_str(&format!("{} = {}\n", name, val));
            }
        }
        for (name, val) in &self.bool_vars {
            if !self.var_assignments.contains_key(name) {
                preamble.push_str(&format!("{} = {}\n", name, if *val { "True" } else { "False" }));
            }
        }

        preamble.push_str(source);
        preamble
    }

    pub fn reset(&mut self) {
        self.chunk_counter = 0;
        self.int_vars.clear();
        self.float_vars.clear();
        self.bool_vars.clear();
        self.str_vars.clear();
        self.function_sources.clear();
        self.var_assignments.clear();
    }

    /// Get a summary of all variables for the :vars command
    pub fn vars_summary(&self) -> Vec<String> {
        let mut result = Vec::new();

        for (name, val) in &self.int_vars {
            result.push(format!("{}: i64 = {}", name, val));
        }
        for (name, val) in &self.float_vars {
            result.push(format!("{}: f64 = {}", name, val));
        }
        for (name, val) in &self.bool_vars {
            result.push(format!("{}: bool = {}", name, val));
        }
        for (name, _val) in &self.str_vars {
            result.push(format!("{}: ptr = <reference>", name));
        }

        // Add function definitions
        for (name, _) in &self.function_sources {
            result.push(format!("{}: fn", name));
        }

        // Add variable assignments (for re-execution)
        for (name, assignment) in &self.var_assignments {
            result.push(format!("{}: {}", name, assignment));
        }

        result
    }
}
