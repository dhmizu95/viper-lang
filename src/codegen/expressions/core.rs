//! Core expression code generation and type inference

use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarStorage, VarType};
use inkwell::values::BasicValueEnum;
use crate::codegen::expressions::builtins::*;
use crate::codegen::expressions::calls::*;
use crate::codegen::expressions::collections::*;
use crate::codegen::expressions::concurrency::*;
use crate::codegen::expressions::operators::*;

pub fn generate_tuple<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Use runtime function to create heap-allocated tuple
    let tuple_create_func = state
        .module
        .get_function("vp_tuple_create")
        .ok_or_else(|| "vp_tuple_create not declared".to_string())?;

    // Create tuple with the right size
    let size_val = state.ir_builder.i64_const(elements.len() as i64);
    let tuple_val = state
        .ir_builder
        .build_call(state.builder, tuple_create_func, &[size_val.into()], "new_tuple")
        .ok_or_else(|| "Failed to create tuple".to_string())?;

    // If tuple has elements, set them
    if !elements.is_empty() {
        let tuple_set_func = state
            .module
            .get_function("vp_tuple_set")
            .ok_or_else(|| "vp_tuple_set not declared".to_string())?;

        for (idx, elem) in elements.iter().enumerate() {
            let elem_val = generate_expr(state, elem)?;
            let index_val = state.ir_builder.i64_const(idx as i64);
            
            // Ensure element is i64 for storage in tuple
            let elem_i64 = if elem_val.is_int_value() {
                let int_val = elem_val.into_int_value();
                let int_type = int_val.get_type();
                // Extend smaller int types to i64
                if int_type.get_bit_width() != 64 {
                    state
                        .builder
                        .build_int_z_extend(int_val, state.context.i64_type(), "extend_to_i64")
                        .map_err(|e| format!("Failed to extend int to i64: {:?}", e))?
                } else {
                    int_val
                }
            } else if elem_val.is_float_value() {
                // Bitcast f64 to i64 for storage (tagged value)
                state
                    .builder
                    .build_float_to_signed_int(elem_val.into_float_value(), state.context.i64_type(), "f64_to_i64")
                    .map_err(|e| format!("Failed to convert float to i64: {:?}", e))?
            } else if elem_val.is_pointer_value() {
                // Pointer types (str, list, etc.) are already i64-compatible
                state
                    .builder
                    .build_ptr_to_int(elem_val.into_pointer_value(), state.context.i64_type(), "ptr_to_i64")
                    .map_err(|e| format!("Failed to convert ptr to i64: {:?}", e))?
            } else {
                // Check if it's a bool (i1 type)
                let ty = elem_val.get_type();
                if ty.is_int_type() && ty.into_int_type().get_bit_width() == 1 {
                    // Bool (i1) needs to be zero-extended to i64
                    state
                        .builder
                        .build_int_z_extend(elem_val.into_int_value(), state.context.i64_type(), "bool_to_i64")
                        .map_err(|e| format!("Failed to extend bool to i64: {:?}", e))?
                } else {
                    return Err(format!("Unsupported tuple element type: {:?}", ty));
                }
            };

            let _ = state
                .ir_builder
                .build_call(
                    state.builder,
                    tuple_set_func,
                    &[tuple_val.into(), index_val.into(), elem_i64.into()],
                    &format!("tuple_set_{}", idx),
                );
        }
    }

    Ok(tuple_val)
}

pub fn infer_expr_type(expr: &Expr) -> Type {
    match expr {
        Expr::Int(_, _) => Type::I64,
        Expr::Float(_, _) => Type::F64,
        Expr::Bool(_, _) => Type::Bool,
        Expr::Str(_, _) => Type::Str,
        Expr::Bytes(_, _) => Type::Bytes,
        Expr::BigInt(_, _) => Type::BigInt,
        Expr::None(_) => Type::None,
        Expr::Ident(_, _) => Type::Infer, // Will be resolved during codegen
        Expr::Call { func, args, .. } => {
            if let Expr::Ident(name, _) = func.as_ref() {
                // Built-in BigInt functions (case insensitive for constructor)
                if name == "bigint" || name == "BigInt" || name == "abs_bigint" || name == "pow_bigint" || name == "sqrt_bigint" || name == "min_bigint" || name == "max_bigint" {
                    return Type::BigInt;
                }
                // str() returns string, not BigInt
                if name == "str" {
                    return Type::Str;
                }
                // print(), len() etc. return None or i64
                if name == "print" || name == "len" || name == "range" {
                    return if name == "len" { Type::I64 } else { Type::None };
                }

                let arg_types: Vec<Type> = args.iter().map(infer_expr_type).collect();
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
            if lt == Type::BigInt || rt == Type::BigInt {
                Type::BigInt
            } else if lt == Type::F64 || rt == Type::F64 {
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
        Expr::AssignmentExpr { value, .. } => infer_expr_type(value),
        Expr::Super(_) => Type::Object,  // super() returns base object type
    }
}

/// Generate code for an expression
pub fn generate_expr<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    expr: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    match expr {
        Expr::Int(n, _) => Ok(state.ir_builder.i64_const(*n).into()),
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
        Expr::BigInt(s, _) => {
            // Call vp_bigint_from_str to create a BigInt from string
            let str_val = state.ir_builder.string_const(state.module, s);
            let create_func = state
                .module
                .get_function("vp_bigint_from_str")
                .ok_or_else(|| "vp_bigint_from_str not declared".to_string())?;
            let result = state
                .ir_builder
                .build_call(state.builder, create_func, &[str_val.into()], "bigint_create")
                .expect("bigint_from_str call");
            Ok(result.into())
        }
        Expr::Bytes(b, _) => {
            let bytes_val = state.ir_builder.bytes_const(state.module, b);
            let create_func = state
                .module
                .get_function("vp_bytes_create")
                .ok_or_else(|| "vp_bytes_create not declared. Add to runtime library.".to_string())?;
            let result = state
                .ir_builder
                .build_call(state.builder, create_func, &[bytes_val.into(), state.ir_builder.i64_const(b.len() as i64).into()], "bytes_create")
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
                    None => {
                        // No initializer (e.g., __name__), check var_types for pointer types
                        if state.var_types.get(name).map(|t| matches!(t, crate::ast::Type::Str | crate::ast::Type::List(_) | crate::ast::Type::Dict(_, _) | crate::ast::Type::Bytes)).unwrap_or(false) {
                            state.context.ptr_type(inkwell::AddressSpace::default()).into()
                        } else {
                            state.context.i64_type().into()
                        }
                    }
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
                        // Check if this is a tuple type (heap-allocated pointer)
                        let is_tuple = state.var_types.get(name)
                            .map(|t| matches!(t, crate::ast::Type::Tuple(_)))
                            .unwrap_or(false);
                        
                        if is_tuple {
                            // Tuples are now heap-allocated pointers
                            let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                            Ok(state.builder.build_load(ptr_type, *alloca, name).expect("load tuple"))
                        } else {
                            match var_info.var_type {
                                VarType::Float => {
                                    let f64_type = state.context.f64_type();
                                    Ok(state.builder.build_load(f64_type, *alloca, name).expect("load"))
                                }
                                VarType::Pointer | VarType::Bytes => {
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
                                VarType::Struct => {
                                    // For struct types (Result), use the default Result struct type
                                    let result_struct_type = state.context.struct_type(&[
                                        state.context.i8_type().into(),
                                        state.context.i64_type().into(),
                                    ], false);
                                    Ok(state.builder.build_load(result_struct_type, *alloca, name).expect("load struct"))
                                }
                            }
                        }
                    }
                    VarStorage::ClosureCell(_) => {
                        // Closure cell: load through the cell's value pointer
                        if let Some(value_ptr) = &var_info.closure_value_ptr {
                            // Check if this is a tuple type (heap-allocated pointer)
                            let is_tuple = state.var_types.get(name)
                                .map(|t| matches!(t, crate::ast::Type::Tuple(_)))
                                .unwrap_or(false);
                            
                            if is_tuple {
                                // Tuples are now heap-allocated pointers
                                let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                                Ok(state.builder.build_load(ptr_type, *value_ptr, name).expect("load tuple from cell"))
                            } else {
                                match var_info.var_type {
                                    VarType::Float => {
                                        let f64_type = state.context.f64_type();
                                        Ok(state.builder.build_load(f64_type, *value_ptr, name).expect("load from cell"))
                                    }
                                    VarType::Pointer | VarType::Bytes => {
                                        let ptr_type =
                                            state.context.ptr_type(inkwell::AddressSpace::default());
                                        Ok(state.builder.build_load(ptr_type, *value_ptr, name).expect("load from cell"))
                                    }
                                    VarType::Bool => {
                                        let bool_type = state.context.bool_type();
                                        Ok(state
                                            .builder
                                            .build_load(bool_type, *value_ptr, name)
                                            .expect("load from cell"))
                                    }
                                    VarType::Int => {
                                        let i64_type = state.context.i64_type();
                                        Ok(state.builder.build_load(i64_type, *value_ptr, name).expect("load from cell"))
                                    }
                                    VarType::Struct => {
                                        // Load struct value (e.g., Result)
                                        let result_struct_type = state.context.struct_type(&[
                                            state.context.i8_type().into(),
                                            state.context.i64_type().into(),
                                        ], false);
                                        Ok(state.builder.build_load(result_struct_type, *value_ptr, name).expect("load from cell"))
                                    }
                                }
                            }
                        } else {
                            Err(format!("Closure cell for '{}' missing value pointer", name))
                        }
                    }
                }
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        Expr::List { elements, span: _ } => generate_list(state, elements),
        Expr::Array { elements, size, span: _ } => generate_array(state, elements, *size),
        Expr::Tuple { elements, span: _ } => generate_tuple(state, elements),
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
        Expr::Attribute { obj, attr, span: _ } => {
            // First try user-defined class attribute access
            if let Ok(result) = crate::codegen::oop::generate_attribute_access(state, obj, attr) {
                return Ok(result);
            }
            // Fall back to just evaluating the object
            generate_expr(state, obj)
        }
        Expr::Await { future, span: _ } => generate_await(state, future),
        Expr::Lambda { params, body, span } => generate_lambda(state, params, body, *span),
        Expr::ListComprehension { element, var, iter, span } => {
            generate_list_comprehension(state, element, var, iter, *span)
        }
        Expr::AssignmentExpr { target, value, span } => {
            generate_assignment_expr(state, target, value, *span)
        }
        Expr::Super(_span) => {
            // super() - returns a special super object for method resolution
            // For now, we'll handle this specially in method call generation
            // Return a null pointer as a placeholder - the actual resolution happens
            // when super().method() is called
            Ok(state.context.ptr_type(inkwell::AddressSpace::default()).const_null().into())
        }
    }
}
