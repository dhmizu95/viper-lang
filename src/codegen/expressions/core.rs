//! Core expression code generation and type inference

use super::*;
use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarStorage, VarType};
use crate::utils::mangle_function_name;
use inkwell::values::BasicValueEnum;

pub fn infer_expr_type(expr: &Expr) -> Type {
    match expr {
        Expr::Int(_, _) => Type::I64,
        Expr::BigInt(_, _) => Type::BigInt,
        Expr::Float(_, _) => Type::F64,
        Expr::Bool(_, _) => Type::Bool,
        Expr::Str(_, _) => Type::Str,
        Expr::None(_) => Type::None,
        Expr::Ident(_, _) => Type::Infer, // Will be resolved during codegen
        Expr::Call { func, args, .. } => {
            if let Expr::Ident(name, _) = func.as_ref() {
                let arg_types: Vec<Type> = args.iter().map(infer_expr_type).collect();
                let _mangled = mangle_function_name(name, &arg_types);
                Type::Fn(arg_types, Box::new(Type::Infer))
            } else {
                Type::Infer
            }
        }
        Expr::List { elements, .. } => {
            if let Some(first) = elements.first() {
                Type::List(Box::new(infer_expr_type(first)))
            } else {
                Type::List(Box::new(Type::Infer))
            }
        }
        Expr::Array { elements, size, .. } => {
            if let Some(first) = elements.first() {
                Type::Array(Box::new(infer_expr_type(first)), size.unwrap_or(0))
            } else {
                Type::Array(Box::new(Type::Infer), size.unwrap_or(0))
            }
        }
        Expr::Tuple { elements, .. } => Type::Tuple(elements.iter().map(infer_expr_type).collect()),
        Expr::Dict { .. } => Type::Var("dict".to_string()),
        Expr::BinOp { op: _, left, right, .. } => {
            let lt = infer_expr_type(left);
            let rt = infer_expr_type(right);
            if lt == Type::F64 || rt == Type::F64 {
                Type::F64
            } else {
                Type::I64
            }
        }
        Expr::UnaryOp { op: _, operand, .. } => infer_expr_type(operand),
        Expr::Attribute { .. } => Type::Infer,
        Expr::Index { .. } => Type::Infer,
        Expr::Slice { .. } => Type::List(Box::new(Type::Infer)),
        Expr::FString { .. } => Type::Str,
        Expr::Await { .. } => Type::Infer,
        Expr::Lambda { .. } => Type::Fn(vec![], Box::new(Type::Infer)),
        Expr::Conditional { .. } => Type::Infer,
        Expr::ListComprehension { .. } => Type::List(Box::new(Type::Infer)),
    }
}

/// Generate code for an expression
pub fn generate_expr<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    expr: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    match expr {
        Expr::Int(n, _) => Ok(state.ir_builder.i64_const(*n).into()),
        Expr::BigInt(s, _) => {
            // Create BigInt from string literal
            let str_val = state.ir_builder.string_const(state.module, s);
            let create_func = state
                .module
                .get_function("vp_bigint_from_str")
                .ok_or_else(|| "vp_bigint_from_str not declared".to_string())?;
            let result = state
                .ir_builder
                .build_call(state.builder, create_func, &[str_val.into()], "bigint_create")
                .unwrap();
            Ok(result)
        }
        Expr::Float(n, _) => Ok(state.ir_builder.f64_const(*n).into()),
        Expr::Bool(b, _) => Ok(state.ir_builder.bool_const(*b).into()),
        Expr::None(_) => Ok(state.ir_builder.i64_const(0).into()),
        Expr::Str(s, _) => {
            let str_val = state.ir_builder.string_const(state.module, s);
            let create_func = state
                .module
                .get_function("vp_str_create")
                .ok_or_else(|| "vp_str_create not declared".to_string())?;
            let result = state
                .ir_builder
                .build_call(state.builder, create_func, &[str_val.into()], "str_create")
                .unwrap();
            Ok(result)
        }
        Expr::FString(elements, _) => {
            if elements.is_empty() {
                let str_val = state.ir_builder.string_const(state.module, "");
                let create_func = state.module.get_function("vp_str_create").unwrap();
                let result = state
                    .ir_builder
                    .build_call(state.builder, create_func, &[str_val.into()], "str_create")
                    .unwrap();
                return Ok(result);
            }
            let mut current = generate_str_call(state, &elements[0..1])?;
            for elem in elements.iter().skip(1) {
                let next_val = generate_str_call(state, std::slice::from_ref(elem))?;
                current = generate_str_concat(state, current, next_val)?;
            }
            Ok(current)
        }
        Expr::Ident(name, _span) => {
            // First check if it's a global constant
            if let Some(global) = state.global_constants.get(name) {
                // Load the global constant value directly
                let global_ptr = global.as_pointer_value();
                // For simple literal types, determine the load type from the value
                let load_type = match global.get_initializer() {
                    Some(init) => init.get_type(),
                    None => state.context.i64_type().into(),
                };
                let loaded = state
                    .builder
                    .build_load(load_type, global_ptr, name)
                    .expect("load global constant");
                return Ok(loaded);
            }

            // Otherwise check local variables
            if let Some(var_info) = state.variables.get(name) {
                // Handle both stack and register allocated variables
                match &var_info.storage {
                    VarStorage::Register(value) => {
                        // Register-allocated variable: return value directly
                        Ok(*value)
                    }
                    VarStorage::Stack(alloca) => {
                        // Stack-allocated variable: load from alloca
                        match var_info.var_type {
                            VarType::Float => {
                                let f64_type = state.context.f64_type();
                                Ok(state.builder.build_load(f64_type, *alloca, name).expect("load"))
                            }
                            VarType::Pointer | VarType::BigInt => {
                                let ptr_type =
                                    state.context.ptr_type(inkwell::AddressSpace::default());
                                Ok(state.builder.build_load(ptr_type, *alloca, name).expect("load"))
                            }
                            VarType::Bool => {
                                let bool_type = state.context.bool_type();
                                Ok(state
                                    .builder
                                    .build_load(bool_type, *alloca, name)
                                    .expect("load"))
                            }
                            VarType::Int => {
                                let i64_type = state.context.i64_type();
                                Ok(state.builder.build_load(i64_type, *alloca, name).expect("load"))
                            }
                        }
                    }
                }
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        Expr::List { elements, span: _ } => generate_list(state, elements),
        Expr::Array { elements, size, span: _ } => generate_array(state, elements, *size),
        Expr::Tuple { elements, span: _ } => {
            if elements.is_empty() {
                Ok(state.ir_builder.i64_const(0).into())
            } else {
                generate_expr(state, &elements[0])
            }
        }
        Expr::Dict { pairs, span: _ } => generate_dict(state, pairs),
        Expr::Index { obj, index, span: _ } => generate_index(state, obj, index),
        Expr::Slice { obj, start, end, step, span: _ } => {
            generate_slice(state, obj, start, end, step)
        }
        Expr::BinOp { left, op, right, .. } => generate_binop(state, left, op, right),
        Expr::UnaryOp { op, operand, .. } => generate_unary(state, op, operand),
        Expr::Conditional { condition, then_expr, else_expr, span: _ } => {
            generate_conditional(state, condition, then_expr, else_expr)
        }
        Expr::Call { func, args, span } => generate_call(state, func, args, *span),
        Expr::Attribute { obj, attr: _, span: _ } => generate_expr(state, obj),
        Expr::Await { future, span: _ } => generate_await(state, future),
        Expr::Lambda { params, body, span } => generate_lambda(state, params, body, *span),
        Expr::ListComprehension { element, var, iter, span } => {
            generate_list_comprehension(state, element, var, iter, *span)
        }
    }
}
