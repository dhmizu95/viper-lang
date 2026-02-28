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
        Expr::BigInt(_, _) => Type::BigInt,
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
            BinOp::Add => {
                // String concatenation: str + str = str
                let lt = infer_type_from_expr(left, param_types);
                let rt = infer_type_from_expr(right, param_types);
                if lt == Type::Str && rt == Type::Str {
                    Type::Str
                } else if lt == Type::BigInt || rt == Type::BigInt {
                    Type::BigInt
                } else if lt == Type::F64 || rt == Type::F64 {
                    Type::F64
                } else {
                    Type::I64
                }
            }
            _ => {
                let lt = infer_type_from_expr(left, param_types);
                let rt = infer_type_from_expr(right, param_types);
                // BigInt operations return BigInt
                if lt == Type::BigInt || rt == Type::BigInt {
                    Type::BigInt
                } else if lt == Type::F64 || rt == Type::F64 {
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
            // Check if parameter is used with index operations (indicating a list)
            if param_is_used_as_list(&param.name, body) {
                return Type::List(Box::new(Type::Infer));
            }

            // Check if parameter is used as an iterable in a for loop
            if param_is_used_as_iterable(&param.name, body) {
                return Type::List(Box::new(Type::Infer));
            }

            // Check if parameter is used in arithmetic/comparison operations (indicating scalar)
            if param_is_used_as_scalar(&param.name, body) {
                return Type::I64;
            }

            // Check if parameter is used with BigInt (assigned or compared with BigInt literals)
            if param_is_used_as_bigint(&param.name, body) {
                return Type::BigInt;
            }

            // Also check if parameter is passed to another function (indicating reference type)
            // This is a fallback for parameters that are only passed through
            if param_is_passed_to_function(&param.name, body) {
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

/// Check if a parameter is used as an iterable in a for loop
fn param_is_used_as_iterable(param_name: &str, body: &[Stmt]) -> bool {
    for stmt in body {
        if stmt_contains_iterable_usage(param_name, stmt) {
            return true;
        }
    }
    false
}

/// Check if a statement contains a for loop where the parameter is the iterable
fn stmt_contains_iterable_usage(param_name: &str, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::For { iter, .. } => {
            if let Expr::Ident(name, _) = iter.as_ref() {
                return name == param_name;
            }
            expr_contains_ident(param_name, iter)
        }
        Stmt::If { body, else_body, .. } => {
            body.iter().any(|s| stmt_contains_iterable_usage(param_name, s))
                || else_body.as_ref().map_or(false, |eb| {
                    eb.iter().any(|s| stmt_contains_iterable_usage(param_name, s))
                })
        }
        Stmt::While { body, .. } => {
            body.iter().any(|s| stmt_contains_iterable_usage(param_name, s))
        }
        Stmt::Function { body, .. } => {
            body.iter().any(|s| stmt_contains_iterable_usage(param_name, s))
        }
        _ => false,
    }
}

/// Check if an expression contains an identifier
fn expr_contains_ident(param_name: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Ident(name, _) => name == param_name,
        Expr::BinOp { left, right, .. } => {
            expr_contains_ident(param_name, left) || expr_contains_ident(param_name, right)
        }
        Expr::UnaryOp { operand, .. } => expr_contains_ident(param_name, operand),
        Expr::Call { func, args, .. } => {
            expr_contains_ident(param_name, func)
                || args.iter().any(|arg| expr_contains_ident(param_name, arg))
        }
        Expr::Attribute { obj, .. } => expr_contains_ident(param_name, obj),
        Expr::Index { obj, index, .. } => {
            expr_contains_ident(param_name, obj) || expr_contains_ident(param_name, index)
        }
        _ => false,
    }
}

/// Check if a parameter is used as a scalar (in arithmetic/comparison operations)
fn param_is_used_as_scalar(param_name: &str, body: &[Stmt]) -> bool {
    for stmt in body {
        if stmt_contains_scalar_usage(param_name, stmt) {
            return true;
        }
    }
    false
}

/// Check if a parameter is used with BigInt (assigned BigInt or used in BigInt operations)
fn param_is_used_as_bigint(param_name: &str, body: &[Stmt]) -> bool {
    for stmt in body {
        if stmt_contains_bigint_usage(param_name, stmt) {
            return true;
        }
    }
    false
}

/// Check if a parameter is passed as argument to a function call
/// This indicates the parameter might be a reference type (list, dict, etc.)
fn param_is_passed_to_function(param_name: &str, body: &[Stmt]) -> bool {
    for stmt in body {
        if stmt_contains_function_call_with_param(param_name, stmt) {
            return true;
        }
    }
    false
}

/// Check if a parameter is used as a scalar (in arithmetic/comparison operations)
fn stmt_contains_scalar_usage(param_name: &str, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(expr) => expr_contains_scalar_usage(param_name, expr),
        Stmt::If { condition, .. } => expr_contains_scalar_usage(param_name, condition),
        Stmt::While { condition, .. } => expr_contains_scalar_usage(param_name, condition),
        Stmt::Return { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_scalar_usage(param_name, v))
        }
        Stmt::Assign { value, .. } => expr_contains_scalar_usage(param_name, value),
        Stmt::Declare { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_scalar_usage(param_name, v))
        }
        Stmt::AugAssign { target, value, .. } => {
            expr_contains_scalar_usage(param_name, target)
                || expr_contains_scalar_usage(param_name, value)
        }
        _ => false,
    }
}

/// Check if a parameter is used with BigInt (assigned BigInt or used in BigInt operations)
fn stmt_contains_bigint_usage(param_name: &str, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Assign { value, .. } => {
            expr_contains_bigint_usage(param_name, value)
        }
        Stmt::Declare { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_bigint_usage(param_name, v))
        }
        Stmt::If { condition, body, else_body, .. } => {
            expr_contains_bigint_usage(param_name, condition)
                || body.iter().any(|s| stmt_contains_bigint_usage(param_name, s))
                || else_body
                    .as_ref()
                    .map_or(false, |eb| eb.iter().any(|s| stmt_contains_bigint_usage(param_name, s)))
        }
        Stmt::While { condition, body, .. } => {
            expr_contains_bigint_usage(param_name, condition)
                || body.iter().any(|s| stmt_contains_bigint_usage(param_name, s))
        }
        Stmt::Return { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_bigint_usage(param_name, v))
        }
        _ => false,
    }
}

/// Check if an expression uses BigInt
fn expr_contains_bigint_usage(param_name: &str, expr: &Expr) -> bool {
    match expr {
        // BigInt literal
        Expr::BigInt(..) => true,
        // Check if param is compared or assigned with BigInt
        Expr::BinOp { left, right, .. } => {
            expr_contains_bigint_usage(param_name, left)
                || expr_contains_bigint_usage(param_name, right)
        }
        Expr::Call { func, args, .. } => {
            // Check if calling BigInt constructor or related functions
            if let Expr::Ident(name, _) = func.as_ref() {
                if name == "BigInt" || name == "int" {
                    return args.iter().any(|arg| {
                        matches!(arg, Expr::Ident(n, _) if n == param_name)
                    });
                }
            }
            args.iter().any(|arg| expr_contains_bigint_usage(param_name, arg))
        }
        _ => false,
    }
}

/// Check if an expression uses the parameter as a scalar (arithmetic/comparison)
fn expr_contains_scalar_usage(param_name: &str, expr: &Expr) -> bool {
    match expr {
        // Binary operations (arithmetic, comparison, logical) indicate scalar usage
        Expr::BinOp { left, right, .. } => {
            // Check if this param is directly involved in the operation
            let left_is_param = matches!(left.as_ref(), Expr::Ident(name, _) if name == param_name);
            let right_is_param =
                matches!(right.as_ref(), Expr::Ident(name, _) if name == param_name);
            if left_is_param || right_is_param {
                return true;
            }
            // Recursively check sub-expressions
            expr_contains_scalar_usage(param_name, left)
                || expr_contains_scalar_usage(param_name, right)
        }
        Expr::UnaryOp { operand, .. } => {
            if matches!(operand.as_ref(), Expr::Ident(name, _) if name == param_name) {
                return true;
            }
            expr_contains_scalar_usage(param_name, operand)
        }
        // Index operations on the param indicate it's a list, not scalar
        Expr::Index { obj, .. } => {
            if matches!(obj.as_ref(), Expr::Ident(name, _) if name == param_name) {
                return false; // This is list usage, not scalar
            }
            expr_contains_scalar_usage(param_name, obj)
        }
        // Function calls - check arguments but passing to function doesn't indicate scalar
        Expr::Call { args, .. } => {
            args.iter().any(|arg| expr_contains_scalar_usage(param_name, arg))
        }
        Expr::Attribute { obj, .. } => expr_contains_scalar_usage(param_name, obj),
        _ => false,
    }
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

/// Check if a statement contains a function call where the parameter is passed as an argument
fn stmt_contains_function_call_with_param(param_name: &str, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(expr) => expr_contains_function_call_with_param(param_name, expr),
        Stmt::Assign { target, value, .. } => {
            expr_contains_function_call_with_param(param_name, target)
                || expr_contains_function_call_with_param(param_name, value)
        }
        Stmt::If { condition, body, else_body, .. } => {
            expr_contains_function_call_with_param(param_name, condition)
                || body.iter().any(|s| stmt_contains_function_call_with_param(param_name, s))
                || else_body.as_ref().map_or(false, |eb| {
                    eb.iter().any(|s| stmt_contains_function_call_with_param(param_name, s))
                })
        }
        Stmt::While { condition, body, .. } => {
            expr_contains_function_call_with_param(param_name, condition)
                || body.iter().any(|s| stmt_contains_function_call_with_param(param_name, s))
        }
        Stmt::For { body, .. } => {
            body.iter().any(|s| stmt_contains_function_call_with_param(param_name, s))
        }
        Stmt::Return { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_function_call_with_param(param_name, v))
        }
        Stmt::Function { body, .. } => {
            body.iter().any(|s| stmt_contains_function_call_with_param(param_name, s))
        }
        Stmt::Declare { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_function_call_with_param(param_name, v))
        }
        Stmt::AugAssign { value, .. } => expr_contains_function_call_with_param(param_name, value),
        _ => false,
    }
}

/// Check if an expression contains a function call where the parameter is passed as an argument
fn expr_contains_function_call_with_param(param_name: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Call { func: _, args, .. } => {
            // Check if any argument is the parameter
            args.iter().any(|arg| {
                if let Expr::Ident(name, _) = arg {
                    name == param_name
                } else {
                    false
                }
            })
            // Also check nested calls in arguments
            || args.iter().any(|arg| expr_contains_function_call_with_param(param_name, arg))
        }
        Expr::BinOp { left, right, .. } => {
            expr_contains_function_call_with_param(param_name, left)
                || expr_contains_function_call_with_param(param_name, right)
        }
        Expr::UnaryOp { operand, .. } => {
            expr_contains_function_call_with_param(param_name, operand)
        }
        Expr::Attribute { obj, .. } => expr_contains_function_call_with_param(param_name, obj),
        Expr::Index { obj, index, .. } => {
            expr_contains_function_call_with_param(param_name, obj)
                || expr_contains_function_call_with_param(param_name, index)
        }
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
