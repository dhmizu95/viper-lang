//! Function declaration for Viper code generation

use crate::ast::{Expr, Param, Stmt, Type};
use crate::utils::mangle_function_name;
use inkwell::context::Context;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
use std::collections::HashMap;

use crate::codegen::types::TypeMapper;

fn infer_return_type_from_body(body: &[Stmt], param_types: &[(String, Type)]) -> Option<Type> {
    for stmt in body {
        if let Stmt::Return { value, .. } = stmt {
            if let Some(expr) = value {
                return Some(infer_type_from_expr(expr, param_types));
            }
        }
        if let Stmt::If { body, else_body, .. } = stmt {
            if let Some(rt) = infer_return_type_from_body(body, param_types) {
                return Some(rt);
            }
            if let Some(else_stmts) = else_body {
                if let Some(rt) = infer_return_type_from_body(else_stmts, param_types) {
                    return Some(rt);
                }
            }
        }
    }
    None
}

fn infer_type_from_expr(expr: &Expr, param_types: &[(String, Type)]) -> Type {
    use crate::ast::BinOp;

    match expr {
        Expr::Int(_, _) => Type::I64,
        Expr::Float(_, _) => Type::F64,
        Expr::Bool(_, _) => Type::Bool,
        Expr::Str(_, _) => Type::Str,
        Expr::None(_) => Type::None,
        Expr::Ident(name, _) => param_types
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, t)| t.clone())
            .unwrap_or(Type::Infer),
        Expr::List { elements, .. } => {
            if let Some(first) = elements.first() {
                let elem_type = infer_type_from_expr(first, param_types);
                Type::List(Box::new(elem_type))
            } else {
                Type::List(Box::new(Type::Infer))
            }
        }
        Expr::ListComprehension { .. } => Type::List(Box::new(Type::Infer)),
        Expr::BinOp { op, left, right, .. } => match op {
            BinOp::Eq
            | BinOp::NotEq
            | BinOp::Lt
            | BinOp::Gt
            | BinOp::LtEq
            | BinOp::GtEq
            | BinOp::Is
            | BinOp::IsNot
            | BinOp::In
            | BinOp::NotIn
            | BinOp::And
            | BinOp::Or => Type::Bool,
            _ => {
                let lt = infer_type_from_expr(left, param_types);
                let rt = infer_type_from_expr(right, param_types);
                if lt == Type::F64 || rt == Type::F64 {
                    Type::F64
                } else {
                    Type::I64
                }
            }
        },
        Expr::UnaryOp { operand, .. } => infer_type_from_expr(operand, param_types),
        Expr::Tuple { elements, .. } => {
            Type::Tuple(elements.iter().map(|e| infer_type_from_expr(e, param_types)).collect())
        }
        _ => Type::Infer,
    }
}

/// Infer parameter types from function body by analyzing how parameters are used
pub fn infer_param_types_from_body(params: &[Param], body: &[Stmt]) -> Vec<Type> {
    params
        .iter()
        .map(|param| {
            // If parameter has explicit type annotation, use it
            if let Some(ref ty) = param.type_ann {
                return ty.clone();
            }

            // Otherwise, try to infer from usage in body
            // For now, check if parameter is used with index operations (indicating a list)
            if param_is_used_as_list(&param.name, body) {
                return Type::List(Box::new(Type::Infer));
            }

            // Default to I64 for unannotated parameters
            Type::I64
        })
        .collect()
}

/// Check if a parameter is used as a list (indexed with [])
fn param_is_used_as_list(param_name: &str, body: &[Stmt]) -> bool {
    for stmt in body {
        if stmt_contains_list_index(param_name, stmt) {
            return true;
        }
    }
    false
}

/// Check if a statement contains list indexing of a parameter
fn stmt_contains_list_index(param_name: &str, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(expr) => expr_contains_list_index(param_name, expr),
        Stmt::Assign { target, value, .. } => {
            expr_contains_list_index(param_name, target)
                || expr_contains_list_index(param_name, value)
        }
        Stmt::If { condition, body, else_body, .. } => {
            expr_contains_list_index(param_name, condition)
                || body.iter().any(|s| stmt_contains_list_index(param_name, s))
                || else_body
                    .as_ref()
                    .map_or(false, |eb| eb.iter().any(|s| stmt_contains_list_index(param_name, s)))
        }
        Stmt::While { condition, body, .. } => {
            expr_contains_list_index(param_name, condition)
                || body.iter().any(|s| stmt_contains_list_index(param_name, s))
        }
        Stmt::Return { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_list_index(param_name, v))
        }
        Stmt::Function { body, .. } => body.iter().any(|s| stmt_contains_list_index(param_name, s)),
        Stmt::For { body, .. } => body.iter().any(|s| stmt_contains_list_index(param_name, s)),
        Stmt::Declare { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_list_index(param_name, v))
        }
        Stmt::AugAssign { value, .. } => expr_contains_list_index(param_name, value),
        _ => false,
    }
}

/// Check if an expression contains list indexing of a parameter
fn expr_contains_list_index(param_name: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Index { obj, .. } => {
            // Check if the object being indexed is the parameter
            if let Expr::Ident(name, _) = obj.as_ref() {
                if name == param_name {
                    return true;
                }
            }
            // Recursively check sub-expressions
            expr_contains_list_index(param_name, obj)
        }
        Expr::BinOp { left, right, .. } => {
            expr_contains_list_index(param_name, left)
                || expr_contains_list_index(param_name, right)
        }
        Expr::UnaryOp { operand, .. } => expr_contains_list_index(param_name, operand),
        Expr::Call { func, args, .. } => {
            expr_contains_list_index(param_name, func)
                || args.iter().any(|arg| expr_contains_list_index(param_name, arg))
        }
        Expr::Attribute { obj, .. } => expr_contains_list_index(param_name, obj),
        _ => false,
    }
}

/// Declare a function (forward declaration)
pub fn declare_function<'ctx>(
    context: &'ctx Context,
    module: &mut inkwell::module::Module<'ctx>,
    type_mapper: &TypeMapper<'ctx>,
    functions: &mut HashMap<String, FunctionValue<'ctx>>,
    name: &str,
    params: &[Param],
    return_type: &Option<Type>,
    body: Option<&[Stmt]>,
) -> Result<(), String> {
    // Infer parameter types from body if not annotated
    let param_types = if let Some(body) = body {
        infer_param_types_from_body(params, body)
    } else {
        params.iter().map(|p| p.type_ann.clone().unwrap_or(Type::I64)).collect()
    };

    let param_llvm_types: Vec<_> = param_types
        .iter()
        .map(|ty| type_mapper.llvm_type(ty).as_basic_type_enum().into())
        .collect();

    // If no return type annotation, try to infer from body
    let param_type_pairs: Vec<(String, Type)> =
        params.iter().zip(param_types.iter()).map(|(p, t)| (p.name.clone(), t.clone())).collect();
    let inferred_return_type = if return_type.is_none() {
        if let Some(body) = body {
            infer_return_type_from_body(body, &param_type_pairs)
        } else {
            None
        }
    } else {
        return_type.clone()
    };

    // Special case: main() always returns i64 for proper exit code
    let fn_type = if name == "main" && return_type.is_none() && inferred_return_type.is_none() {
        let i64_type = context.i64_type();
        i64_type.fn_type(&param_llvm_types, false)
    } else {
        match type_mapper.llvm_return_type(&inferred_return_type) {
            Some(return_ty) => return_ty.fn_type(&param_llvm_types, false),
            None => context.void_type().fn_type(&param_llvm_types, false),
        }
    };

    let mangled_name = mangle_function_name(name, &param_types);
    let func = module.add_function(&mangled_name, fn_type, None);
    functions.insert(mangled_name, func);

    Ok(())
}
