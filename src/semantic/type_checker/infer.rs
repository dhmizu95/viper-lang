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
            Expr::BigInt(_, _) => Some(Type::BigInt),
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
            Expr::Dict { pairs, .. } => {
                if pairs.is_empty() {
                    Some(Type::Dict(Box::new(Type::Infer), Box::new(Type::Infer)))
                } else {
                    let key_type = self.infer_expr_type(&pairs[0].0);
                    let value_type = self.infer_expr_type(&pairs[0].1);
                    match (key_type, value_type) {
                        (Some(k), Some(v)) => Some(Type::Dict(Box::new(k), Box::new(v))),
                        _ => Some(Type::Dict(Box::new(Type::Infer), Box::new(Type::Infer))),
                    }
                }
            }
            Expr::Await { future, .. } => {
                // Await returns the type of the future
                self.infer_expr_type(future)
            }
            Expr::AssignmentExpr { value, .. } => {
                // Walrus operator returns the type of the value
                self.infer_expr_type(value)
            }
            Expr::Super(_) => {
                // super() returns the base object type
                // The actual method resolution happens at compile time
                Some(Type::Object)
            }
            Expr::Call { func, args, keywords, span: _ } => {
                let mut call_args: Vec<&Expr> = args.iter().collect();
                for (_, value) in keywords {
                    call_args.push(value);
                }
                if let Expr::Ident(name, _) = func.as_ref() {
                    // Handle Result constructors with context from function return type
                    match name.as_str() {
                        "Ok" => {
                            // Ok(value) -> Result[value_type, Infer] or use return type context
                            let value_type = call_args
                                .first()
                                .and_then(|a| self.infer_expr_type(a));

                            // Check if we're in a function returning Result[T, E]
                            if let Some(ref ret_type) = self.current_return_type {
                                if let Type::Result(_expected_ok, expected_err) = ret_type {
                                    // Use the expected error type from return type
                                    let vt = value_type.clone().unwrap_or(Type::Infer);
                                    return Some(Type::Result(
                                        Box::new(vt),
                                        Box::new((**expected_err).clone()),
                                    ));
                                }
                            }

                            // No context, use Infer for error type
                            return if let Some(vt) = value_type {
                                Some(Type::Result(Box::new(vt), Box::new(Type::Infer)))
                            } else {
                                Some(Type::Result(Box::new(Type::Infer), Box::new(Type::Infer)))
                            };
                        }
                        "Err" => {
                            // Err(error) -> Result[Infer, error_type] or use return type context
                            let error_type = call_args
                                .first()
                                .and_then(|a| self.infer_expr_type(a));

                            // Check if we're in a function returning Result[T, E]
                            if let Some(ref ret_type) = self.current_return_type {
                                if let Type::Result(expected_ok, _expected_err) = ret_type {
                                    // Use the expected ok type from return type
                                    let et = error_type.clone().unwrap_or(Type::Infer);
                                    return Some(Type::Result(
                                        Box::new((**expected_ok).clone()),
                                        Box::new(et),
                                    ));
                                }
                            }

                            // No context, use Infer for ok type
                            return if let Some(et) = error_type {
                                Some(Type::Result(Box::new(Type::Infer), Box::new(et)))
                            } else {
                                Some(Type::Result(Box::new(Type::Infer), Box::new(Type::Infer)))
                            };
                        }
                        _ => {}
                    }

                    // Handle concurrency builtins (Phase 3)
                    match name.as_str() {
                        "chan" => Some(Type::Infer), // Chan element type inferred from usage
                        "recv" => Some(Type::Infer), // Returns channel element type
                        "WaitGroup" => Some(Type::WaitGroup),
                        "send" | "add" | "done" | "wait" => Some(Type::None),
                        // Type conversion functions
                        "str" => Some(Type::Str),
                        "int" => Some(Type::I64),
                        "float" => Some(Type::F64),
                        "bool" => Some(Type::Bool),
                        _ => {
                            // For function calls, use get_function_overloads to find by name prefix
                            let overloads = self.symbol_table.get_function_overloads(name);
                            if let Some(symbol) = overloads.first() {
                                if let SymbolKind::Function { return_type, .. } = &symbol.kind {
                                    return_type.clone()
                                } else {
                                    None
                                }
                            } else if let Some(symbol) = self.symbol_table.lookup(name) {
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
                                        // BigInt functions - removed, use int type instead
                                        // BigInt constructor removed
                                        crate::semantic::symbol_table::BuiltinSignature::StrBigint => {
                                            Some(Type::Str)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::IntBigint => {
                                            Some(Type::I64)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::AbsBigint => {
                                            Some(Type::BigInt)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::PowBigint => {
                                            Some(Type::BigInt)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::SqrtBigint => {
                                            Some(Type::BigInt)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::MinBigint => {
                                            Some(Type::BigInt)
                                        }
                                        crate::semantic::symbol_table::BuiltinSignature::MaxBigint => {
                                            Some(Type::BigInt)
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
                BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Mod
                | BinOp::FloorDiv
                | BinOp::Pow => {
                    // Get types of both operands for promotion rules
                    let left_type = self.infer_expr_type(left);
                    let right_type = self.infer_expr_type(right);

                    // Automatic promotion rules for mixed BigInt/i64 operations
                    // If either operand is BigInt, result is BigInt
                    // If either operand is F64, result is F64
                    // Otherwise, result is I64
                    match (&left_type, &right_type) {
                        (Some(Type::BigInt), _) | (_, Some(Type::BigInt)) => Some(Type::BigInt),
                        (Some(Type::F64), _) | (_, Some(Type::F64)) => Some(Type::F64),
                        (Some(lt), _) => Some(lt.clone()),
                        (_, Some(rt)) => Some(rt.clone()),
                        _ => None,
                    }
                }
                BinOp::Eq
                | BinOp::NotEq
                | BinOp::Lt
                | BinOp::LtEq
                | BinOp::Gt
                | BinOp::GtEq
                | BinOp::And
                | BinOp::Or
                | BinOp::Is
                | BinOp::IsNot
                | BinOp::In
                | BinOp::NotIn => Some(Type::Bool),
                BinOp::NullCoalesce => {
                    self.infer_expr_type(left).or_else(|| self.infer_expr_type(right))
                }
                _ => Some(Type::I64),
            },
            Expr::UnaryOp { op, operand, .. } => match op {
                UnaryOp::Neg | UnaryOp::Pos => self.infer_expr_type(operand),
                UnaryOp::Not => Some(Type::Bool),
                UnaryOp::Unwrap => {
                    // ? operator: requires Result[T, E], returns T
                    if let Some(operand_type) = self.infer_expr_type(operand) {
                        match operand_type {
                            Type::Result(ok_type, _) => Some(*ok_type),
                            _ => None, // Will be caught by validation
                        }
                    } else {
                        None
                    }
                }
                UnaryOp::UnwrapOrDefault => {
                    // unwrap_or_default(): requires Result[T, E], returns T
                    if let Some(operand_type) = self.infer_expr_type(operand) {
                        match operand_type {
                            Type::Result(ok_type, _) => Some(*ok_type),
                            _ => None,
                        }
                    } else {
                        None
                    }
                }
                _ => Some(Type::I64),
            },
            Expr::Index { obj, .. } => {
                if let Some(obj_type) = self.infer_expr_type(obj) {
                    match obj_type {
                        Type::List(elem_type) => Some(*elem_type),
                        Type::Dict(_, value_type) => Some(*value_type),
                        Type::Str => Some(Type::Str), // String indexing returns a string (char)
                        Type::Var(s) if s.starts_with("dict") => Some(Type::Infer),
                        _ => Some(Type::I64),
                    }
                } else {
                    None
                }
            }
            Expr::Attribute { obj, attr, .. } => {
                // Try to infer type from class field definition
                if let Expr::Ident(obj_name, _) = obj.as_ref() {
                    if obj_name == "self" {
                        // Look up the current class context from the symbol table
                        // We need to find the class that contains this method
                        for (_, symbol) in self.symbol_table.get_all_symbols() {
                            if let SymbolKind::Class { fields, .. } = &symbol.kind {
                                // Check if this field exists in the class
                                for (field_name, field_type) in fields {
                                    if field_name == attr {
                                        if field_type != &Type::Infer {
                                            return Some(field_type.clone());
                                        }
                                    }
                                }
                                // Also check if we can infer from field assignments
                                // For now, return Infer if field type is not known
                            }
                        }
                    }
                }
                Some(Type::Infer)
            }
            Expr::Conditional { then_expr, .. } => self.infer_expr_type(then_expr),
            Expr::Lambda { .. } => Some(Type::Var("lambda".to_string())),
            Expr::ListComprehension { .. } => Some(Type::List(Box::new(Type::Infer))),
            Expr::Slice { .. } => Some(Type::List(Box::new(Type::Infer))),
        }
    }
}
