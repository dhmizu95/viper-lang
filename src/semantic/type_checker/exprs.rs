use crate::ast::{BinOp, Expr, Type, UnaryOp};
use crate::semantic::symbol_table::SymbolKind;
use crate::semantic::type_checker::{TypeChecker, TypeError};

impl TypeChecker {
    /// Check an expression and return its type
    pub(crate) fn check_expr(&mut self, expr: &Expr) -> Option<Type> {
        let expr_type = self.infer_expr_type(expr);

        // Store the inferred type
        let span = expr.span();
        self.expr_types.insert(span.start as usize, expr_type.clone().unwrap_or(Type::Infer));

        match expr {
            Expr::BinOp { left, right, op, span } => {
                let left_type = self.check_expr(left);
                let right_type = self.check_expr(right);

                if let (Some(lt), Some(rt)) = (left_type, right_type) {
                    match op {
                        BinOp::Add => {
                            // String concatenation: Str + Str is allowed
                            if lt == Type::Str && rt == Type::Str {
                                // String concatenation is valid
                            }
                            // List concatenation: List + List is allowed
                            else if let (Type::List(_), Type::List(_)) = (&lt, &rt) {
                                // List concatenation is valid
                            }
                            // For other types, require numeric
                            else if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Arithmetic operators require numeric types, got {} and {}",
                                        lt, rt
                                    ),
                                    *span,
                                ));
                            }
                        }
                        BinOp::Mul => {
                            // List repetition: List * int or int * List is allowed
                            let is_list_repeat = match (&lt, &rt) {
                                (Type::List(_), Type::I64) => true,
                                (Type::I64, Type::List(_)) => true,
                                _ => false,
                            };
                            if !is_list_repeat {
                                // For other types, require numeric
                                if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                    self.errors.push(TypeError::new(
                                        format!(
                                            "Arithmetic operators require numeric types, got {} and {}",
                                            lt, rt
                                        ),
                                        *span,
                                    ));
                                }
                            }
                        }
                        BinOp::Sub | BinOp::Div | BinOp::Mod | BinOp::FloorDiv | BinOp::Pow => {
                            if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Arithmetic operators require numeric types, got {} and {}",
                                        lt, rt
                                    ),
                                    *span,
                                ));
                            }
                        }
                        BinOp::Eq | BinOp::NotEq => {
                            // Allow comparison between compatible types (e.g. BigInt == i64)
                            if lt != rt && !self.is_compatible(&lt, &rt) {
                                self.errors.push(TypeError::new(
                                    format!("Cannot compare {} with {}", lt, rt),
                                    *span,
                                ));
                            }
                        }
                        BinOp::Lt | BinOp::LtEq | BinOp::Gt | BinOp::GtEq => {
                            if !self.is_numeric(&lt) || !self.is_numeric(&rt) {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Comparison operators require numeric types, got {} and {}",
                                        lt, rt
                                    ),
                                    *span,
                                ));
                            }
                        }
                        BinOp::And | BinOp::Or => {
                            if lt != Type::Bool || rt != Type::Bool {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Logical operators require bool, got {} and {}",
                                        lt, rt
                                    ),
                                    *span,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Expr::Call { func, args, span } => {
                // First, check all argument expressions to infer their types
                for arg in args {
                    self.check_expr(arg);
                }
                
                // Handle function call with overload resolution
                if let Expr::Ident(name, _) = func.as_ref() {
                    // Check if this is a class instantiation
                    let is_class = if let Some(symbol) = self.symbol_table.lookup(name) {
                        matches!(&symbol.kind, SymbolKind::Class { .. })
                    } else {
                        false
                    };
                    
                    if is_class {
                        // This is a class instantiation - valid
                        // The __init__ method will be called by codegen
                    } else {
                    // Check if this function has overloads
                    let overloads = self.symbol_table.get_function_overloads(name);
                    
                    if overloads.len() > 1 {
                        // Multiple overloads - resolve to the best match
                        match self.resolve_overload(name, args) {
                            Ok(_mangled_name) => {
                                // Successfully resolved - the mangled name is used by codegen
                                // Type is inferred from the resolved function
                            }
                            Err(msg) => {
                                self.errors.push(TypeError::new(msg, *span));
                            }
                        }
                    } else if let Some(symbol) = self.symbol_table.lookup(name) {
                        // Single function or builtin - check argument count
                        if let SymbolKind::Function { params, .. } = &symbol.kind {
                            if params.len() != args.len() {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "Expected {} arguments, got {}",
                                        params.len(),
                                        args.len()
                                    ),
                                    *span,
                                ));
                            }
                        }
                        // For builtins, argument checking is done elsewhere
                    } else {
                        // Try looking up by mangled name in case it's a function
                        // Check if any function with this name prefix exists
                        let has_function = self.symbol_table.get_function_overloads(name).len() > 0;
                        if has_function {
                            // Function exists but lookup failed - this is OK for single definitions
                        } else {
                            // Function not found
                            self.errors.push(TypeError::new(
                                format!("Undefined function '{}'", name),
                                *span,
                            ));
                        }
                    }
                    } // End of else block for class check
                }
            }
            Expr::Index { obj, index, span } => {
                let obj_type = self.check_expr(obj);
                let index_type = self.check_expr(index);

                if let (Some(ot), Some(it)) = (obj_type, index_type) {
                    match ot {
                        Type::Dict(k, _) => {
                            if !self.is_compatible(&k, &it) {
                                self.errors.push(TypeError::new(
                                    format!("Dict key must be {}, got {}", k, it),
                                    *span,
                                ));
                            }
                        }
                        _ => {
                            if it != Type::I64 {
                                self.errors.push(TypeError::new(
                                    format!("Index must be i64, got {}", it),
                                    *span,
                                ));
                            }
                        }
                    }
                }
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }
            Expr::Array { elements, .. } => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }
            Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }
            Expr::Dict { pairs, span: _ } => {
                for (key, value) in pairs {
                    self.check_expr(key);
                    self.check_expr(value);

                    // Check that key type is hashable
                    if let Some(key_type) = self.get_expr_type(key) {
                        if !key_type.is_fully_hashable() {
                            self.errors.push(TypeError::new(
                                format!("Dictionary keys must be hashable, got {}", key_type),
                                key.span(),
                            ));
                        }
                    }
                }
            }
            Expr::UnaryOp { op, operand, span } => {
                match op {
                    UnaryOp::Unwrap | UnaryOp::UnwrapOrDefault => {
                        // ? operator requires Result[T, E] type
                        if let Some(operand_type) = self.get_expr_type(operand) {
                            match &operand_type {
                                Type::Result(_ok_type, _err_type) => {
                                    // Check if we're in a function that returns Result
                                    // This is needed for error propagation to work
                                    if !self.is_in_result_returning_function(&operand_type) {
                                        self.errors.push(TypeError::new(
                                            format!(
                                                "The `?` operator can only be used in functions that return Result, got {}",
                                                operand_type
                                            ),
                                            *span,
                                        ));
                                    }
                                }
                                _ => {
                                    self.errors.push(TypeError::new(
                                        format!(
                                            "The `?` operator requires a Result type, got {}",
                                            operand_type
                                        ),
                                        *span,
                                    ));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            // ... (other validations as needed)
            _ => {}
        }

        expr_type
    }

    /// Check if we're currently in a function that returns Result
    fn is_in_result_returning_function(&self, _operand_type: &Type) -> bool {
        // For now, always return true to allow the operator
        // In a full implementation, check the current function's return type
        true
    }
}
