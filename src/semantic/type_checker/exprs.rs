use crate::ast::{BinOp, Expr, Type};
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
                            // List concatenation: List + List is allowed
                            match (&lt, &rt) {
                                (Type::List(_), Type::List(_)) => {
                                    // List concatenation is valid
                                }
                                _ => {
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
                if let Expr::Ident(name, _) = func.as_ref() {
                    if let Some(symbol) = self.symbol_table.lookup(name) {
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
                    }
                }

                // Check argument types
                for arg in args {
                    self.check_expr(arg);
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
            // ... (other validations as needed)
            _ => {}
        }

        expr_type
    }
}
