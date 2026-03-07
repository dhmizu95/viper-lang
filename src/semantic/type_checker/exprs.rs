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
                            // String repetition: Str * int or int * Str is allowed
                            let is_list_repeat = match (&lt, &rt) {
                                (Type::List(_), Type::I64) | (Type::List(_), Type::Int) => true,
                                (Type::I64, Type::List(_)) | (Type::Int, Type::List(_)) => true,
                                (Type::Str, Type::I64) | (Type::Str, Type::Int) => true,
                                (Type::I64, Type::Str) | (Type::Int, Type::Str) => true,
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
                for (i, arg) in args.iter().enumerate() {
                    // Special case: isinstance's second argument is a type name, not a value
                    // We still check it as an expression but don't enforce type constraints
                    if let Expr::Ident(name, _) = func.as_ref() {
                        if name == "isinstance" && i == 1 {
                            // Second arg to isinstance is a type name - just validate it's an identifier or None
                            // Don't check it as a variable reference
                            continue;
                        }
                    }
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
                    
                    // Explicitly store the return type for the call expression
                    if let Some(symbol) = self.symbol_table.lookup(name) {
                        if let SymbolKind::Function { return_type, .. } = &symbol.kind {
                            if let Some(ret_ty) = return_type {
                                self.expr_types.insert(span.start as usize, ret_ty.clone());
                            }
                        } else if let SymbolKind::Builtin { signature } = &symbol.kind {
                            let builtin_ret = match signature {
                                crate::semantic::symbol_table::BuiltinSignature::Print => Some(Type::None),
                                crate::semantic::symbol_table::BuiltinSignature::Range => Some(Type::List(Box::new(Type::I64))),
                                crate::semantic::symbol_table::BuiltinSignature::Len => Some(Type::I64),
                                crate::semantic::symbol_table::BuiltinSignature::Str => Some(Type::Str),
                                crate::semantic::symbol_table::BuiltinSignature::Int => Some(Type::I64),
                                crate::semantic::symbol_table::BuiltinSignature::Float => Some(Type::F64),
                                crate::semantic::symbol_table::BuiltinSignature::Bool => Some(Type::Bool),
                                // Program control
                                crate::semantic::symbol_table::BuiltinSignature::Exit => Some(Type::None),
                                // BigInt functions - removed, use int type instead
                                // BigInt constructor removed
                                crate::semantic::symbol_table::BuiltinSignature::StrBigint => Some(Type::Str),
                                crate::semantic::symbol_table::BuiltinSignature::IntBigint => Some(Type::I64),
                                crate::semantic::symbol_table::BuiltinSignature::AbsBigint => Some(Type::BigInt),
                                crate::semantic::symbol_table::BuiltinSignature::PowBigint => Some(Type::BigInt),
                                crate::semantic::symbol_table::BuiltinSignature::SqrtBigint => Some(Type::BigInt),
                                crate::semantic::symbol_table::BuiltinSignature::MinBigint => Some(Type::BigInt),
                                crate::semantic::symbol_table::BuiltinSignature::MaxBigint => Some(Type::BigInt),
                                // Math builtins (not requiring import)
                                crate::semantic::symbol_table::BuiltinSignature::Abs => Some(Type::F64),
                                _ => None,
                            };
                            if let Some(ty) = builtin_ret {
                                self.expr_types.insert(span.start as usize, ty);
                            }
                        }
                    }
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
                            if it != Type::I64 && it != Type::Int {
                                self.errors.push(TypeError::new(
                                    format!("Index must be i64 or int, got {}", it),
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
                        // First, check the operand expression to store its type
                        self.check_expr(operand);
                        
                        // Now get the stored type for the operand
                        let operand_type = self.get_expr_type(operand);
                        
                        match &operand_type {
                            Some(Type::Result(_ok_type, _err_type)) => {
                                // Check if we're in a function that returns Result
                                // This is needed for error propagation to work
                                if !self.is_in_result_returning_function(operand_type.as_ref().unwrap()) {
                                    self.errors.push(TypeError::new(
                                        format!(
                                            "The `?` operator can only be used in functions that return Result"
                                        ),
                                        *span,
                                    ));
                                }
                            }
                            Some(other) => {
                                self.errors.push(TypeError::new(
                                    format!(
                                        "The `?` operator requires a Result type, got {}",
                                        other
                                    ),
                                    *span,
                                ));
                            }
                            None => {
                                // Type couldn't be inferred - this is an error
                                self.errors.push(TypeError::new(
                                    "The `?` operator requires a Result type, but type could not be inferred".to_string(),
                                    *span,
                                ));
                            }
                        }
                    }
                    _ => {
                        // For other unary ops, just check the operand
                        self.check_expr(operand);
                    }
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

    /// Check an expression allowing undefined exception classes (for raise statements)
    /// This is used for raise X from Y where X and Y may be exception classes not defined in the code
    pub(crate) fn check_expr_allow_undefined_class(&mut self, expr: &Expr) {
        match expr {
            Expr::Call { func: _, args, .. } => {
                // For exception class calls like ValueError("msg"), allow undefined classes
                // Just check the arguments
                for arg in args {
                    self.check_expr(arg);
                }
            }
            _ => {
                // For other expressions, use normal type checking
                self.check_expr(expr);
            }
        }
    }
}
