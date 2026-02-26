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
        }
    }

    pub fn execute_chunk(&mut self, source: &str) -> Result<(), String> {
        self.chunk_counter += 1;

        // 1. Wrap source with declarations for our shadowed variables
        let wrapped_source = self.build_wrapped_source(source);

        // 2. Tokenize and parse
        let mut lexer = crate::lexer::Lexer::new(&wrapped_source);
        let tokens = lexer.tokenize()?;

        let mut parser = crate::parser::Parser::new(tokens);
        let ast = parser.parse()?;

        // Extract variable assignments from AST to update shadow store
        // This happens BEFORE execution - we assume execution will succeed
        self.extract_assignments_from_ast(&ast);

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
            Expr::BigInt(_, _) => {
                // BigInt is a pointer type, just track existence
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
                // Function call - just track that variable exists with placeholder
                // The actual value will come from the wrapped source on next chunk
                self.int_vars.insert(name.to_string(), 0);
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
        // Prepend user's code with earlier variable state to preserve values across chunks
        let mut preamble = String::new();

        for (name, val) in &self.int_vars {
            preamble.push_str(&format!("{} = {}\n", name, val));
        }
        for (name, val) in &self.float_vars {
            preamble.push_str(&format!("{} = {}\n", name, val));
        }
        for (name, val) in &self.bool_vars {
            preamble.push_str(&format!("{} = {}\n", name, if *val { "True" } else { "False" }));
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

        result
    }
}
