use crate::ast::{BinOp, Expr, Type, UnaryOp};
use crate::semantic::symbol_table::SymbolKind;
use crate::semantic::type_checker::TypeChecker;

impl TypeChecker {
    /// Get the inferred type of an expression
    pub fn get_expr_type(&self, expr: &Expr) -> Option<Type> {
        // Use span as a rough identifier for the expression
        let span = expr.span();
        // Try to find in our type map or infer from the expression itself
        self.expr_types.get(&(span.start as usize)).cloned().or_else(|| self.infer_expr_type(expr))
    }

    /// Infer the type of an expression
    pub(crate) fn infer_expr_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Int(_, _) => Some(Type::I64),
            Expr::Float(_, _) => Some(Type::F64),
            Expr::Bool(_, _) => Some(Type::Bool),
            Expr::Str(_, _) | Expr::FString(_, _) => Some(Type::Str),
            Expr::Bytes(_, _) => Some(Type::Bytes),
            Expr::None(_) => Some(Type::None),
            Expr::Ident(name, _) => self.symbol_table.lookup(name).and_then(|s| s.get_type()),
            Expr::List { elements, .. } => {
                if elements.is_empty() {
                    Some(Type::List(Box::new(Type::Infer)))
                } else {
                    self.infer_expr_type(&elements[0]).map(|t| Type::List(Box::new(t)))
                }
            }
            Expr::Array { elements, size, .. } => {
                if elements.is_empty() {
                    Some(Type::Array(Box::new(Type::Infer), size.unwrap_or(0)))
                } else {
                    self.infer_expr_type(&elements[0])
                        .map(|t| Type::Array(Box::new(t), size.unwrap_or(elements.len())))
                }
            }
            Expr::Tuple { elements, .. } => {
                let types: Vec<Type> =
                    elements.iter().filter_map(|e| self.infer_expr_type(e)).collect();
                if types.len() == elements.len() {
                    Some(Type::Tuple(types))
                } else {
                    None
                }
            }
            Expr::Dict { .. } => Some(Type::Var("dict".to_string())),
            Expr::Await { future, .. } => {
                // Await returns the type of the future
                self.infer_expr_type(future)
            }
            Expr::AssignmentExpr { value, .. } => {
                // Walrus operator returns the type of the value
                self.infer_expr_type(value)
            }
            Expr::Call { func, args: _, span: _ } => {
                if let Expr::Ident(name, _) = func.as_ref() {
                    // Handle concurrency builtins (Phase 3)
                    match name.as_str() {
                        "chan" => Some(Type::Infer), // Chan element type inferred from usage
                        "recv" => Some(Type::Infer), // Returns channel element type
                        "WaitGroup" => Some(Type::WaitGroup),
                        "send" | "add" | "done" | "wait" => Some(Type::None),
                        _ => {
                            if let Some(symbol) = self.symbol_table.lookup(name) {
                                if let SymbolKind::Function { return_type, .. } = &symbol.kind {
                                    return_type.clone()
                                } else if let SymbolKind::Builtin { signature } = &symbol.kind {
                                    // Handle builtin return types
                                    match signature {
                                        crate::semantic::symbol_table::BuiltinSignature::Print => {
                                            Some(Type::None)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::Range => {
                                            Some(Type::List(Box::new(Type::I64)))
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::Len => {
                                            Some(Type::I64)
                                        }
                                        _ => Some(Type::Infer),
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            }
            Expr::BinOp { op, left, right, .. } => match op {
                // Arithmetic: if either operand is BigInt, result is BigInt
                // True division (/) always returns float
                BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Mod
                | BinOp::FloorDiv
                | BinOp::Pow => {
                    let left_type = self.infer_expr_type(left);
                    let right_type = self.infer_expr_type(right);
                    // If either is BigInt, result is BigInt
                    if left_type == Some(Type::BigInt) || right_type == Some(Type::BigInt) {
                        Some(Type::BigInt)
                    } else {
                        left_type.or_else(|| right_type)
                    }
                }
                BinOp::Div => Some(Type::F64),
                // Comparisons return bool
                BinOp::Eq
                | BinOp::NotEq
                | BinOp::Lt
                | BinOp::LtEq
                | BinOp::Gt
                | BinOp::GtEq
                | BinOp::And
                | BinOp::Or => Some(Type::Bool),
                // Bitwise: if either is BigInt, result is BigInt
                BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor | BinOp::LShift | BinOp::RShift => {
                    let left_type = self.infer_expr_type(left);
                    let right_type = self.infer_expr_type(right);
                    if left_type == Some(Type::BigInt) || right_type == Some(Type::BigInt) {
                        Some(Type::BigInt)
                    } else {
                        Some(Type::I64)
                    }
                }
                BinOp::NullCoalesce => {
                    self.infer_expr_type(left).or_else(|| self.infer_expr_type(right))
                }
                BinOp::Is | BinOp::IsNot | BinOp::In | BinOp::NotIn => Some(Type::Bool),
            },
            Expr::UnaryOp { op, operand, .. } => match op {
                UnaryOp::Neg | UnaryOp::Pos => {
                    let operand_type = self.infer_expr_type(operand);
                    if operand_type == Some(Type::BigInt) {
                        Some(Type::BigInt)
                    } else {
                        operand_type
                    }
                }
                UnaryOp::Not => Some(Type::Bool),
                UnaryOp::Invert => {
                    // Bitwise NOT - if operand is BigInt, result is BigInt
                    let operand_type = self.infer_expr_type(operand);
                    if operand_type == Some(Type::BigInt) {
                        Some(Type::BigInt)
                    } else {
                        Some(Type::I64)
                    }
                }
                _ => Some(Type::I64),
            },
            Expr::Index { obj, .. } => {
                if let Some(obj_type) = self.infer_expr_type(obj) {
                    match obj_type {
                        Type::List(elem_type) => Some(*elem_type),
                        Type::Var(s) if s.starts_with("dict") => Some(Type::Infer),
                        _ => Some(Type::I64),
                    }
                } else {
                    None
                }
            }
            Expr::Attribute { .. } => Some(Type::Infer),
            Expr::Conditional { then_expr, .. } => self.infer_expr_type(then_expr),
            Expr::Lambda { .. } => Some(Type::Var("lambda".to_string())),
            Expr::ListComprehension { .. } => Some(Type::List(Box::new(Type::Infer))),
            Expr::Slice { .. } => Some(Type::List(Box::new(Type::Infer))),
        }
    }
}
