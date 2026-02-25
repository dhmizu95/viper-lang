//! Function declaration for Viper code generation

use crate::ast::{Expr, Param, Stmt, Type};
use crate::utils::mangle_function_name;
use inkwell::context::Context;
use inkwell::types::BasicType;
use inkwell::values::FunctionValue;
use std::collections::HashMap;

use crate::codegen::types::TypeMapper;

fn infer_return_type_from_body(body: &[Stmt]) -> Option<Type> {
    for stmt in body {
        if let Stmt::Return { value, .. } = stmt {
            if let Some(expr) = value {
                return Some(infer_type_from_expr(expr));
            }
        }
        if let Stmt::If {
            body, else_body, ..
        } = stmt
        {
            if let Some(rt) = infer_return_type_from_body(body) {
                return Some(rt);
            }
            if let Some(else_stmts) = else_body {
                if let Some(rt) = infer_return_type_from_body(else_stmts) {
                    return Some(rt);
                }
            }
        }
    }
    None
}

fn infer_type_from_expr(expr: &Expr) -> Type {
    match expr {
        Expr::Int(_, _) => Type::I64,
        Expr::Float(_, _) => Type::F64,
        Expr::Bool(_, _) => Type::Bool,
        Expr::Str(_, _) => Type::Str,
        Expr::None(_) => Type::None,
        Expr::Ident(_, _) => Type::Infer,
        Expr::BinOp { left, right, .. } => {
            let lt = infer_type_from_expr(left);
            let rt = infer_type_from_expr(right);
            if lt == Type::F64 || rt == Type::F64 {
                Type::F64
            } else {
                Type::I64
            }
        }
        Expr::UnaryOp { operand, .. } => infer_type_from_expr(operand),
        _ => Type::Infer,
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
    let param_types: Vec<Type> = params
        .iter()
        .map(|p| p.type_ann.clone().unwrap_or(Type::I64))
        .collect();

    let param_llvm_types: Vec<_> = param_types
        .iter()
        .map(|ty| type_mapper.llvm_type(ty).as_basic_type_enum().into())
        .collect();

    // If no return type annotation, try to infer from body
    let inferred_return_type = if return_type.is_none() {
        if let Some(body) = body {
            infer_return_type_from_body(body)
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
