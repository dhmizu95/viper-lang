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
        match stmt {
            Stmt::Return { value: Some(expr), .. } => {
                return Some(infer_type_from_expr(expr, param_types));
            }
            Stmt::If { body, elif_blocks, else_body, .. } => {
                if let Some(rt) = infer_return_type_from_body(body, param_types) {
                    return Some(rt);
                }
                for (_, elif_body) in elif_blocks {
                    if let Some(rt) = infer_return_type_from_body(elif_body, param_types) {
                        return Some(rt);
                    }
                }
                if let Some(else_stmts) = else_body {
                    if let Some(rt) = infer_return_type_from_body(else_stmts, param_types) {
                        return Some(rt);
                    }
                }
            }
            Stmt::While { body, else_body, .. } | Stmt::For { body, else_body, .. } => {
                if let Some(rt) = infer_return_type_from_body(body, param_types) {
                    return Some(rt);
                }
                if let Some(else_stmts) = else_body {
                    if let Some(rt) = infer_return_type_from_body(else_stmts, param_types) {
                        return Some(rt);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn infer_type_from_expr(expr: &Expr, param_types: &[(String, Type)]) -> Type {
    use crate::ast::BinOp;

    match expr {
        Expr::Int(_, _) => Type::Int,
        Expr::Float(_, _) => Type::F64,
        Expr::Bool(_, _) => Type::Bool,
        Expr::Str(_, _) => Type::Str,
        Expr::BigInt(_, _) => Type::Int,
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
                } else if lt == Type::BigInt
                    || rt == Type::BigInt
                    || lt == Type::Int
                    || rt == Type::Int
                {
                    Type::Int
                } else if lt == Type::F64 || rt == Type::F64 {
                    Type::F64
                } else {
                    Type::Int
                }
            }
            _ => {
                let lt = infer_type_from_expr(left, param_types);
                let rt = infer_type_from_expr(right, param_types);
                // BigInt operations return BigInt
                if lt == Type::BigInt || rt == Type::BigInt || lt == Type::Int || rt == Type::Int {
                    Type::Int
                } else if lt == Type::F64 || rt == Type::F64 {
                    Type::F64
                } else {
                    Type::Int
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

            // Arithmetic/comparison usage indicates the language-level int type.
            if param_is_used_as_scalar(&param.name, body) {
                return Type::Int;
            }

            // Check if parameter is used with BigInt (assigned or compared with BigInt literals)
            if param_is_used_as_bigint(&param.name, body) {
                return Type::BigInt;
            }

            // Check if parameter is only returned (identity function pattern like copy(x))
            // For identity functions, use Type::Infer since they are inlined at call sites
            if param_is_only_returned(&param.name, body) {
                return infer_param_type_for_identity(&param.name, body);
            }

            // Check if parameter is passed to collection-processing functions (sum, min, max, etc.)
            // This indicates the parameter is a list/iterable type
            if param_is_passed_to_collection_function(&param.name, body) {
                return Type::List(Box::new(Type::Infer));
            }

            // Default to Int (tagged integer) for unannotated parameters
            // This is the most common case in Viper code
            Type::Int
        })
        .collect()
}

/// Check if a parameter is only used in a return statement (identity function pattern)
fn param_is_only_returned(param_name: &str, body: &[Stmt]) -> bool {
    // Check if the body is just a single return statement returning the parameter
    if body.len() == 1 {
        if let Stmt::Return { value: Some(expr), .. } = &body[0] {
            if let Expr::Ident(name, _) = expr {
                return name == param_name;
            }
        }
    }
    false
}

/// For identity functions, use Infer type since they are inlined at call sites
/// This avoids type conversion issues - the argument type flows through unchanged
fn infer_param_type_for_identity(_param_name: &str, _body: &[Stmt]) -> Type {
    Type::Infer
}

/// Collection-processing functions that accept list/iterable arguments
/// Parameters passed to these functions should be inferred as list types
fn is_collection_function(name: &str) -> bool {
    matches!(name,
        "sum" | "min" | "max" | "len" | "avg" | "mean" | "median" | "mode"
        | "sorted" | "reversed" | "enumerate" | "zip" | "map" | "filter"
        | "all" | "any" | "count" | "index"
        | "reduce" | "fold" | "accumulate"
        | "vp_list_sum" | "vp_list_min" | "vp_list_max" | "vp_list_len"
    )
}

/// Check if a parameter is passed as argument to a collection-processing function
/// This indicates the parameter is a list/iterable type
fn param_is_passed_to_collection_function(param_name: &str, body: &[Stmt]) -> bool {
    for stmt in body {
        if stmt_contains_collection_function_call_with_param(param_name, stmt) {
            return true;
        }
    }
    false
}

/// Check if a statement contains a collection function call where the parameter is passed
fn stmt_contains_collection_function_call_with_param(param_name: &str, stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(expr) => expr_contains_collection_function_call_with_param(param_name, expr),
        Stmt::Assign { value, .. } => {
            expr_contains_collection_function_call_with_param(param_name, value)
        }
        Stmt::Declare { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_collection_function_call_with_param(param_name, v))
        }
        Stmt::If { condition, body, else_body, .. } => {
            expr_contains_collection_function_call_with_param(param_name, condition)
                || body.iter().any(|s| stmt_contains_collection_function_call_with_param(param_name, s))
                || else_body.as_ref().map_or(false, |eb| {
                    eb.iter().any(|s| stmt_contains_collection_function_call_with_param(param_name, s))
                })
        }
        Stmt::While { condition, body, .. } => {
            expr_contains_collection_function_call_with_param(param_name, condition)
                || body.iter().any(|s| stmt_contains_collection_function_call_with_param(param_name, s))
        }
        Stmt::Return { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_collection_function_call_with_param(param_name, v))
        }
        Stmt::Function { body, .. } => {
            body.iter().any(|s| stmt_contains_collection_function_call_with_param(param_name, s))
        }
        Stmt::For { body, .. } => {
            body.iter().any(|s| stmt_contains_collection_function_call_with_param(param_name, s))
        }
        Stmt::AugAssign { value, .. } => {
            expr_contains_collection_function_call_with_param(param_name, value)
        }
        _ => false,
    }
}

/// Check if an expression contains a collection function call where the parameter is passed
fn expr_contains_collection_function_call_with_param(param_name: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Call { func, args, .. } => {
            // Check if this is a collection function and the parameter is an argument
            let is_collection_func = if let Expr::Ident(func_name, _) = func.as_ref() {
                is_collection_function(func_name)
            } else {
                false
            };

            if is_collection_func {
                // Check if any argument is the parameter
                let param_is_arg = args.iter().any(|arg| {
                    if let Expr::Ident(name, _) = arg {
                        name == param_name
                    } else {
                        false
                    }
                });
                if param_is_arg {
                    return true;
                }
            }
            // Also check nested calls in arguments
            args.iter().any(|arg| expr_contains_collection_function_call_with_param(param_name, arg))
        }
        Expr::BinOp { left, right, .. } => {
            expr_contains_collection_function_call_with_param(param_name, left)
                || expr_contains_collection_function_call_with_param(param_name, right)
        }
        Expr::UnaryOp { operand, .. } => {
            expr_contains_collection_function_call_with_param(param_name, operand)
        }
        Expr::Attribute { obj, .. } => expr_contains_collection_function_call_with_param(param_name, obj),
        Expr::Index { obj, index, .. } => {
            expr_contains_collection_function_call_with_param(param_name, obj)
                || expr_contains_collection_function_call_with_param(param_name, index)
        }
        _ => false,
    }
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
        Stmt::For { iter, body, else_body, .. } => {
            let used_as_iter =
                if let Expr::Ident(name, _) = iter.as_ref() { name == param_name } else { false };

            used_as_iter
                || body.iter().any(|s| stmt_contains_iterable_usage(param_name, s))
                || else_body.as_ref().map_or(false, |eb| {
                    eb.iter().any(|s| stmt_contains_iterable_usage(param_name, s))
                })
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
        Stmt::Assign { value, .. } => expr_contains_bigint_usage(param_name, value),
        Stmt::Declare { value, .. } => {
            value.as_ref().map_or(false, |v| expr_contains_bigint_usage(param_name, v))
        }
        Stmt::If { condition, body, else_body, .. } => {
            expr_contains_bigint_usage(param_name, condition)
                || body.iter().any(|s| stmt_contains_bigint_usage(param_name, s))
                || else_body.as_ref().map_or(false, |eb| {
                    eb.iter().any(|s| stmt_contains_bigint_usage(param_name, s))
                })
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
                if name == "bigint" || name == "int" {
                    return args
                        .iter()
                        .any(|arg| matches!(arg, Expr::Ident(n, _) if n == param_name));
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
        // Function calls - passing a parameter to a function doesn't indicate scalar usage
        // We only check the function expression itself (for method calls like obj.method())
        Expr::Call { func, .. } => expr_contains_scalar_usage(param_name, func),
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
) -> crate::codegen::Result<()> {
    // Normalize return type (convert GenericApp Result to Type::Result)
    let normalized_return_type = return_type.as_ref().map(|t| normalize_type(t));

    // Infer parameter types from body if not annotated
    let param_types = if let Some(body) = body {
        infer_param_types_from_body(params, body)
    } else {
        // Default to Int (tagged integer) for unannotated parameters
        params.iter().map(|p| p.type_ann.clone().unwrap_or(Type::Int)).collect()
    };

    let param_llvm_types: Vec<_> = param_types
        .iter()
        .map(|ty| type_mapper.llvm_type(ty).as_basic_type_enum().into())
        .collect();

    // If no return type annotation, try to infer from body
    let param_type_pairs: Vec<(String, Type)> =
        params.iter().zip(param_types.iter()).map(|(p, t)| (p.name.clone(), t.clone())).collect();
    let inferred_return_type = if normalized_return_type.is_none() {
        if let Some(body) = body {
            infer_return_type_from_body(body, &param_type_pairs)
        } else {
            None
        }
    } else {
        normalized_return_type.clone()
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

/// Declare a function with closure cell parameters (for nested functions with nonlocal)
pub fn declare_function_with_closure<'ctx>(
    context: &'ctx Context,
    module: &mut inkwell::module::Module<'ctx>,
    type_mapper: &TypeMapper<'ctx>,
    functions: &mut HashMap<String, FunctionValue<'ctx>>,
    name: &str,
    params: &[Param],
    return_type: &Option<Type>,
    body: Option<&[Stmt]>,
    nonlocal_vars: &[String],
) -> crate::codegen::Result<()> {
    // First declare with regular params
    let normalized_return_type = return_type.as_ref().map(|t| normalize_type(t));

    // Use type annotations directly if present, otherwise infer from body
    let param_types: Vec<Type> = if let Some(body) = body {
        infer_param_types_from_body(params, body)
    } else {
        params.iter().map(|p| p.type_ann.clone().unwrap_or(Type::Int)).collect()
    };

    // Build LLVM parameter types for regular params
    let mut param_llvm_types: Vec<_> = param_types
        .iter()
        .map(|ty| type_mapper.llvm_type(ty).as_basic_type_enum().into())
        .collect();

    // Add closure cell parameters (i8* for each nonlocal variable)
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    for _ in nonlocal_vars {
        param_llvm_types.push(i8_ptr_type.as_basic_type_enum().into());
    }

    // Compute mangled name
    let mangled_name = crate::utils::mangling::mangle_function_name_with_closure(
        name,
        &param_types,
        nonlocal_vars,
    );

    // If no return type annotation, try to infer from body
    let param_type_pairs: Vec<(String, Type)> =
        params.iter().zip(param_types.iter()).map(|(p, t)| (p.name.clone(), t.clone())).collect();
    let inferred_return_type = if normalized_return_type.is_none() {
        if let Some(body) = body {
            infer_return_type_from_body(body, &param_type_pairs)
        } else {
            None
        }
    } else {
        normalized_return_type.clone()
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

    let func = module.add_function(&mangled_name, fn_type, None);
    functions.insert(mangled_name, func);

    Ok(())
}

/// Declare a function with a simple name (no mangling) - used for class methods
pub fn declare_function_simple<'ctx>(
    context: &'ctx Context,
    module: &mut inkwell::module::Module<'ctx>,
    type_mapper: &TypeMapper<'ctx>,
    functions: &mut HashMap<String, FunctionValue<'ctx>>,
    name: &str,
    params: &[Param],
    return_type: &Option<Type>,
) -> crate::codegen::Result<()> {
    // Build LLVM parameter types directly
    let param_llvm_types: Vec<_> = params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            // First parameter named 'self' should be a pointer (instance reference)
            if i == 0 && p.name == "self" {
                context.ptr_type(inkwell::AddressSpace::default()).as_basic_type_enum().into()
            } else {
                // For parameters without type annotation, use pointer type as default
                // This allows unannotated parameters to accept any reference type (str, list, etc.)
                // and is compatible with Viper's dynamic typing for unannotated params
                let ty = p.type_ann.clone().unwrap_or(Type::Str);
                type_mapper.llvm_type(&ty).as_basic_type_enum().into()
            }
        })
        .collect();

    let fn_type = match type_mapper.llvm_return_type(return_type) {
        Some(return_ty) => return_ty.fn_type(&param_llvm_types, false),
        None => context.void_type().fn_type(&param_llvm_types, false),
    };

    let func = module.add_function(name, fn_type, None);
    functions.insert(name.to_string(), func);

    Ok(())
}

/// Normalize a type - convert GenericApp Result[T, E] to Type::Result(T, E)
fn normalize_type(ty: &Type) -> Type {
    match ty {
        Type::GenericApp { name, type_args } => {
            if name == "Result" && type_args.len() == 2 {
                Type::Result(
                    Box::new(normalize_type(&type_args[0])),
                    Box::new(normalize_type(&type_args[1])),
                )
            } else if name == "List" && type_args.len() == 1 {
                Type::List(Box::new(normalize_type(&type_args[0])))
            } else if name == "Dict" && type_args.len() == 2 {
                Type::Dict(
                    Box::new(normalize_type(&type_args[0])),
                    Box::new(normalize_type(&type_args[1])),
                )
            } else if name == "Optional" && type_args.len() == 1 {
                Type::Optional(Box::new(normalize_type(&type_args[0])))
            } else if name == "Future" && type_args.len() == 1 {
                Type::Future(Box::new(normalize_type(&type_args[0])))
            } else if name == "Chan" && type_args.len() == 1 {
                Type::Chan(Box::new(normalize_type(&type_args[0])))
            } else {
                ty.clone()
            }
        }
        // Recursively normalize nested types
        Type::List(inner) => Type::List(Box::new(normalize_type(inner))),
        Type::Dict(k, v) => Type::Dict(Box::new(normalize_type(k)), Box::new(normalize_type(v))),
        Type::Tuple(types) => Type::Tuple(types.iter().map(|t| normalize_type(t)).collect()),
        Type::Fn(params, ret) => Type::Fn(
            params.iter().map(|p| normalize_type(p)).collect(),
            Box::new(normalize_type(ret)),
        ),
        Type::Union(variants) => Type::Union(variants.iter().map(|t| normalize_type(t)).collect()),
        Type::Optional(inner) => Type::Optional(Box::new(normalize_type(inner))),
        Type::Future(inner) => Type::Future(Box::new(normalize_type(inner))),
        Type::Chan(inner) => Type::Chan(Box::new(normalize_type(inner))),
        _ => ty.clone(),
    }
}
