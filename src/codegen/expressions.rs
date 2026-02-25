//! Expression code generation for Viper

use crate::ast::{BinOp, Expr, UnaryOp};

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarStorage, VarType};

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
        },
        Expr::FString(elements, _) => {
            if elements.is_empty() {
                let str_val = state.ir_builder.string_const(state.module, "");
                let create_func = state.module.get_function("vp_str_create").unwrap();
                let result = state.ir_builder.build_call(state.builder, create_func, &[str_val.into()], "str_create").unwrap();
                return Ok(result);
            }
            let mut current = generate_str_call(state, &elements[0..1])?;
            for elem in elements.iter().skip(1) {
                let next_val = generate_str_call(state, std::slice::from_ref(elem))?;
                current = generate_str_concat(state, current, next_val)?;
            }
            Ok(current)
        },
        Expr::Ident(name, _span) => {
            // First check if it's a global constant
            if let Some(global) = state.global_constants.get(name) {
                // Load the global constant value directly
                let global_ptr = global.as_pointer_value();
                // Build load without explicit type - let LLVM infer it
                let loaded = state
                    .builder
                    .build_load(global_ptr.get_type(), global_ptr, name)
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
                                Ok(state
                                    .builder
                                    .build_load(f64_type, *alloca, name)
                                    .expect("load"))
                            }
                            VarType::Pointer => {
                                let ptr_type =
                                    state.context.ptr_type(inkwell::AddressSpace::default());
                                Ok(state
                                    .builder
                                    .build_load(ptr_type, *alloca, name)
                                    .expect("load"))
                            }
                            VarType::Int => {
                                let i64_type = state.context.i64_type();
                                Ok(state
                                    .builder
                                    .build_load(i64_type, *alloca, name)
                                    .expect("load"))
                            }
                        }
                    }
                }
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        Expr::List { elements, span: _ } => generate_list(state, elements),
        Expr::Array {
            elements,
            size,
            span: _,
        } => generate_array(state, elements, *size),
        Expr::Tuple { elements, span: _ } => {
            if elements.is_empty() {
                Ok(state.ir_builder.i64_const(0).into())
            } else {
                generate_expr(state, &elements[0])
            }
        }
        Expr::Dict { pairs, span: _ } => generate_dict(state, pairs),
        Expr::Index {
            obj,
            index,
            span: _,
        } => generate_index(state, obj, index),
        Expr::BinOp {
            left, op, right, ..
        } => generate_binop(state, left, op, right),
        Expr::UnaryOp { op, operand, .. } => generate_unary(state, op, operand),
        Expr::Conditional {
            condition,
            then_expr,
            else_expr,
            span: _,
        } => generate_conditional(state, condition, then_expr, else_expr),
        Expr::Call { func, args, span } => generate_call(state, func, args, *span),
        Expr::Attribute {
            obj,
            attr: _,
            span: _,
        } => generate_expr(state, obj),
        Expr::Await { future, span: _ } => generate_await(state, future),
        Expr::Lambda { params, body, span } => generate_lambda(state, params, body, *span),
    }
}

/// Generate lambda expression
fn generate_lambda<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    params: &[String],
    body: &Expr,
    span: crate::utils::Span,
) -> Result<BasicValueEnum<'ctx>, String> {
    // We assume i64 for all lambda params and return type for now
    let i64_type = state.context.i64_type();
    let mut param_types = Vec::new();
    for _ in params {
        param_types.push(i64_type.into());
    }
    
    let fn_type = i64_type.fn_type(&param_types, false);
    let lambda_name = format!("__lambda_{}_{}", span.line, span.column);
    let func = state.module.add_function(&lambda_name, fn_type, None);
    
    // Save insertion block
    let current_block = state.builder.get_insert_block().unwrap();
    
    let entry_block = state.context.append_basic_block(func, "entry");
    state.builder.position_at_end(entry_block);
    
    // Setup params (as i64)
    // We don't do full closure capture, only parameters
    // We need to temporarily push these params to state.variables
    let mut old_vars = Vec::new();
    for (i, param_name) in params.iter().enumerate() {
        let param_value = func.get_nth_param(i as u32).unwrap();
        let alloca = state.builder.build_alloca(i64_type, param_name).expect("alloca");
        state.builder.build_store(alloca, param_value).expect("store");
        
        let old_var = state.variables.insert(param_name.clone(), crate::codegen::variables::VarInfo::new_stack(alloca, crate::codegen::variables::VarType::Int));
        old_vars.push((param_name.clone(), old_var));
    }
    
    // Generate body
    let body_val = generate_expr(state, body)?;
    let body_int = if body_val.is_int_value() {
        body_val.into_int_value()
    } else {
        return Err("Lambda must return int value currently".to_string());
    };
    state.builder.build_return(Some(&body_int)).expect("return");
    
    // Restore builder
    state.builder.position_at_end(current_block);
    
    // Restore variables
    for (name, old_var) in old_vars {
        if let Some(var) = old_var {
            state.variables.insert(name, var);
        } else {
            state.variables.remove(&name);
        }
    }
    
    // Note: To return a lambda as a value, we can cast the function pointer
    // to a void pointer (ptr_type) representing a closure/function reference
    Ok(func.as_global_value().as_pointer_value().into())
}

/// Generate list creation
fn generate_list<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    // Determine if this is a float list by checking the first element
    let is_float_list = elements
        .first()
        .map(|e| matches!(e, Expr::Float(..)))
        .unwrap_or(false);

    let (list_func_name, append_func_name) = if is_float_list {
        ("vp_list_create_f64", "vp_list_append_f64")
    } else {
        ("vp_list_create", "vp_list_append")
    };

    let list_func = state
        .module
        .get_function(list_func_name)
        .ok_or_else(|| format!("{} not declared", list_func_name))?;

    let list_val = state
        .ir_builder
        .build_call(state.builder, list_func, &[], "new_list")
        .unwrap();

    let append_func = state
        .module
        .get_function(append_func_name)
        .ok_or_else(|| format!("{} not declared", append_func_name))?;

    for (idx, elem) in elements.iter().enumerate() {
        let elem_val = generate_expr(state, elem)?;

        // If float list but elem is int, convert to float
        let elem_val = if is_float_list && elem_val.is_int_value() {
            let int_val = elem_val.into_int_value();
            state
                .builder
                .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
                .expect("int to float conversion")
                .into()
        // Convert bool to i64 for list operations
        } else {
            match elem {
                Expr::Bool(true, _) => state.ir_builder.i64_const(1).into(),
                Expr::Bool(false, _) => state.ir_builder.i64_const(0).into(),
                _ => elem_val.into(),
            }
        };

        let _ = state.ir_builder.build_call(
            state.builder,
            append_func,
            &[list_val.into(), elem_val],
            &format!("list_append_{}", idx),
        );
    }

    Ok(list_val)
}

/// Generate dict creation
fn generate_dict<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    pairs: &[(Expr, Expr)],
) -> Result<BasicValueEnum<'ctx>, String> {
    let dict_create_func = state
        .module
        .get_function("vp_dict_create")
        .ok_or_else(|| "vp_dict_create not declared".to_string())?;

    let dict_val = state
        .ir_builder
        .build_call(state.builder, dict_create_func, &[], "new_dict")
        .unwrap();

    let dict_set_func = state
        .module
        .get_function("vp_dict_set_i64")
        .ok_or_else(|| "vp_dict_set_i64 not declared".to_string())?;

    for (i, (key_expr, value_expr)) in pairs.iter().enumerate() {
        let key_val = generate_expr(state, key_expr)?;
        let value_val = generate_expr(state, value_expr)?;

        let _ = state.ir_builder.build_call(
            state.builder,
            dict_set_func,
            &[dict_val.into(), key_val.into(), value_val.into()],
            &format!("dict_set_{}", i),
        );
    }

    Ok(dict_val)
}

/// Generate array creation (fixed-size, stack-allocated)
fn generate_array<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
    size: Option<usize>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let array_size = size.unwrap_or_else(|| elements.len());

    if array_size == 0 {
        // Empty array - return null pointer
        let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
        return Ok(ptr_type.const_null().into());
    }

    // Get element type from first element or default to i64
    let elem_type: inkwell::types::BasicTypeEnum = if let Some(first_elem) = elements.first() {
        match first_elem {
            Expr::Int(_, _) => state.context.i64_type().into(),
            Expr::Float(_, _) => state.context.f64_type().into(),
            Expr::Bool(_, _) => state.context.bool_type().into(),
            _ => state.context.i64_type().into(),
        }
    } else {
        state.context.i64_type().into()
    };

    // Allocate array on stack as a single alloca of element type with size
    let array_alloca = state
        .builder
        .build_array_alloca(
            elem_type,
            state.context.i32_type().const_int(array_size as u64, false),
            "array",
        )
        .map_err(|e| format!("Failed to allocate array: {:?}", e))?;

    // Check if this is array repeat syntax: [value; size]
    let is_repeat = elements.len() == 1 && size.is_some() && size.unwrap() > 1;

    // Initialize elements
    for i in 0..array_size {
        let elem_val = if is_repeat {
            // For repeat syntax, use the first element value for all positions
            generate_expr(state, &elements[0])?
        } else if i < elements.len() {
            // For regular arrays, use the corresponding element
            generate_expr(state, &elements[i])?
        } else {
            // Fill remaining elements with zero
            let zero_val: BasicValueEnum = if elem_type.is_int_type() {
                elem_type.into_int_type().const_zero().into()
            } else if elem_type.is_float_type() {
                elem_type.into_float_type().const_zero().into()
            } else {
                elem_type.into_int_type().const_zero().into()
            };
            zero_val
        };

        // Create GEP to element position
        let elem_ptr = unsafe {
            state.builder.build_in_bounds_gep(
                elem_type,
                array_alloca,
                &[state.context.i32_type().const_int(i as u64, false)],
                &format!("elem_{}", i),
            )
        }
        .map_err(|e| format!("Failed to build GEP: {:?}", e))?;

        state
            .builder
            .build_store(elem_ptr, elem_val)
            .map_err(|e| format!("Failed to store element: {:?}", e))?;
    }

    Ok(array_alloca.into())
}

/// Generate index access
fn generate_index<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    index: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let obj_val = generate_expr(state, obj)?;
    let index_val = generate_expr(state, index)?;

    // Check if indexing with a string key (dict access)
    if index_val.is_pointer_value() && obj_val.is_pointer_value() {
        let dict_get = state
            .module
            .get_function("vp_dict_get_i64")
            .ok_or_else(|| "vp_dict_get_i64 not declared".to_string())?;

        let result = state
            .ir_builder
            .build_call(
                state.builder,
                dict_get,
                &[obj_val.into(), index_val.into()],
                "dict_get",
            )
            .ok_or_else(|| "build call failed".to_string())?;

        return Ok(result);
    }

    let index_val = index_val.into_int_value();

    // Try array indexing first (gep on pointer)
    if obj_val.is_pointer_value() {
        let obj_ptr = obj_val.into_pointer_value();
        let elem_type = state.context.i64_type(); // Default to i64

        let elem_ptr = unsafe {
            state
                .builder
                .build_in_bounds_gep(elem_type, obj_ptr, &[index_val], "array_elem")
        }
        .map_err(|e| format!("Failed to build array index GEP: {:?}", e))?;

        let loaded = state
            .builder
            .build_load(elem_type, elem_ptr, "array_load")
            .map_err(|e| format!("Failed to load array element: {:?}", e))?;

        return Ok(loaded);
    }

    // Fall back to list indexing
    let list_get = state
        .module
        .get_function("vp_list_get")
        .ok_or_else(|| "vp_list_get not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(
            state.builder,
            list_get,
            &[obj_val.into(), index_val.into()],
            "list_get",
        )
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}

/// Generate binary operation
fn generate_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    if matches!(op, BinOp::And | BinOp::Or) {
        return generate_logical_op(state, left, op, right);
    }

    if matches!(op, BinOp::In | BinOp::NotIn) {
        return generate_membership_op(state, left, op, right);
    }

    // Handle string concatenation with + operator
    if *op == BinOp::Add {
        let lhs_val = generate_expr(state, left)?;
        let rhs_val = generate_expr(state, right)?;

        // Check if both operands are strings (pointer types)
        if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
            return generate_str_concat(state, lhs_val, rhs_val);
        }
    }

    // Handle list * int for list/array literals: [elem] * n
    if *op == BinOp::Mul {
        // Check for List or Array literal
        let elements = match left {
            Expr::List { elements, .. } => Some(elements),
            Expr::Array { elements, .. } => Some(elements),
            _ => None,
        };

        if let Some(elems) = elements {
            if let Some(elem) = elems.first() {
                let count_val = generate_expr(state, right)?;
                let count_int = count_val.into_int_value();

                let elem_val = generate_expr(state, elem)?;

                let elem_i64 = match elem {
                    Expr::Bool(true, _) => state.ir_builder.i64_const(1),
                    Expr::Bool(false, _) => state.ir_builder.i64_const(0),
                    _ => {
                        if elem_val.is_int_value() {
                            elem_val.into_int_value()
                        } else {
                            return Err(
                                "List repeat requires integer or boolean elements".to_string()
                            );
                        }
                    }
                };

                let list_repeat_func = state
                    .module
                    .get_function("vp_list_repeat")
                    .ok_or_else(|| "vp_list_repeat not declared".to_string())?;

                let result = state
                    .ir_builder
                    .build_call(
                        state.builder,
                        list_repeat_func,
                        &[elem_i64.into(), count_int.into()],
                        "list_repeat",
                    )
                    .expect("list_repeat call");

                return Ok(result.into());
            }
        }
    }

    // General binary operation handling
    let lhs_val = generate_expr(state, left)?;
    let rhs_val = generate_expr(state, right)?;

    // Reject pointer values in binary operations (except for Add with strings)
    if lhs_val.is_pointer_value() || rhs_val.is_pointer_value() {
        return Err("Binary operators cannot be applied to pointer values (lists)".to_string());
    }

    // Auto-convert int to float when one operand is float
    if lhs_val.is_float_value() && !rhs_val.is_float_value() {
        // Convert rhs (int) to float
        let rhs_int = rhs_val.into_int_value();
        let rhs_float = state
            .builder
            .build_signed_int_to_float(rhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return generate_float_binop(state.builder, lhs_val, rhs_float.into(), op);
    } else if !lhs_val.is_float_value() && rhs_val.is_float_value() {
        // Convert lhs (int) to float
        let lhs_int = lhs_val.into_int_value();
        let lhs_float = state
            .builder
            .build_signed_int_to_float(lhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return generate_float_binop(state.builder, lhs_float.into(), rhs_val, op);
    } else if lhs_val.is_float_value() {
        return generate_float_binop(state.builder, lhs_val, rhs_val, op);
    } else {
        return generate_int_binop(state, lhs_val, rhs_val, op);
    }
}

/// Generate string concatenation
fn generate_str_concat<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let str_concat = state
        .module
        .get_function("vp_str_concat")
        .ok_or_else(|| "vp_str_concat not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(
            state.builder,
            str_concat,
            &[lhs.into(), rhs.into()],
            "str_concat",
        )
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}

/// Generate logical AND/OR with short-circuiting
fn generate_logical_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs_val = generate_expr(state, left)?.into_int_value();

    let func = state
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
    let then_block = state.context.append_basic_block(func, "logic_then");
    let end_block = state.context.append_basic_block(func, "logic_end");

    let is_and = *op == BinOp::And;

    state
        .builder
        .build_conditional_branch(
            lhs_val,
            if is_and { then_block } else { end_block },
            if is_and { end_block } else { then_block },
        )
        .expect("branch");

    state.builder.position_at_end(then_block);
    let rhs_val = generate_expr(state, right)?.into_int_value();
    state
        .builder
        .build_unconditional_branch(end_block)
        .expect("branch");
    let then_block_end = state.builder.get_insert_block().unwrap();

    state.builder.position_at_end(end_block);
    let phi = state
        .builder
        .build_phi(state.context.bool_type(), "logic_result")
        .expect("phi");

    let cond_block = state
        .builder
        .get_insert_block()
        .unwrap()
        .get_previous_basic_block()
        .unwrap();
    if is_and {
        phi.add_incoming(&[(&lhs_val, cond_block), (&rhs_val, then_block_end)]);
    } else {
        phi.add_incoming(&[(&lhs_val, cond_block), (&rhs_val, then_block_end)]);
    }

    Ok(phi.as_basic_value())
}

/// Generate membership IN/NOT IN operators
fn generate_membership_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let value_val = generate_expr(state, left)?;
    let list_val = generate_expr(state, right)?;

    let list_contains = state
        .module
        .get_function("vp_list_contains")
        .ok_or_else(|| "vp_list_contains not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        list_contains,
        &[list_val.into(), value_val.into()],
        if matches!(op, BinOp::In) {
            "list_contains"
        } else {
            "not_in_contains"
        },
    );
    let contains_val: BasicValueEnum = result.unwrap_or(state.ir_builder.i64_const(0).into());

    if matches!(op, BinOp::NotIn) {
        Ok(state
            .builder
            .build_not(contains_val.into_int_value(), "not_in_result")
            .expect("not")
            .into())
    } else {
        Ok(contains_val)
    }
}

/// Generate float binary operation
fn generate_float_binop<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_float_value();
    let rhs = rhs.into_float_value();

    match op {
        BinOp::Add => Ok(builder
            .build_float_add(lhs, rhs, "fadd")
            .expect("fadd")
            .into()),
        BinOp::Sub => Ok(builder
            .build_float_sub(lhs, rhs, "fsub")
            .expect("fsub")
            .into()),
        BinOp::Mul => Ok(builder
            .build_float_mul(lhs, rhs, "fmul")
            .expect("fmul")
            .into()),
        BinOp::Div => Ok(builder
            .build_float_div(lhs, rhs, "fdiv")
            .expect("fdiv")
            .into()),
        BinOp::Eq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "feq")
            .expect("feq")
            .into()),
        BinOp::NotEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::ONE, lhs, rhs, "fne")
            .expect("fne")
            .into()),
        BinOp::Lt => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OLT, lhs, rhs, "flt")
            .expect("flt")
            .into()),
        BinOp::Gt => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OGT, lhs, rhs, "fgt")
            .expect("fgt")
            .into()),
        BinOp::LtEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OLE, lhs, rhs, "fle")
            .expect("fle")
            .into()),
        BinOp::GtEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OGE, lhs, rhs, "fge")
            .expect("fge")
            .into()),
        BinOp::Is => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_is")
            .expect("f_is")
            .into()),
        BinOp::IsNot => {
            let eq = builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_isnot")
                .expect("f_isnot");
            Ok(builder.build_not(eq, "f_isnot_result").expect("not").into())
        }
        BinOp::FloorDiv => {
            let div = builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv");
            Ok(div.into())
        }
        BinOp::Pow => Err("pow for floats not implemented".to_string()),
        BinOp::In | BinOp::NotIn => {
            Err("Membership operators not supported for float types".to_string())
        }
        _ => Err(format!("Unsupported float operator: {:?}", op)),
    }
}

/// Generate integer binary operation
fn generate_int_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_int_value();
    let rhs = rhs.into_int_value();

    match op {
        BinOp::Add => Ok(state
            .ir_builder
            .build_add(state.builder, lhs, rhs, "add")
            .into()),
        BinOp::Sub => Ok(state
            .ir_builder
            .build_sub(state.builder, lhs, rhs, "sub")
            .into()),
        BinOp::Mul => Ok(state
            .ir_builder
            .build_mul(state.builder, lhs, rhs, "mul")
            .into()),
        BinOp::Div => Ok(state
            .ir_builder
            .build_div(state.builder, lhs, rhs, "div")
            .into()),
        BinOp::Eq => Ok(state
            .ir_builder
            .build_icmp_eq(state.builder, lhs, rhs, "eq")
            .into()),
        BinOp::NotEq => {
            let eq = state
                .ir_builder
                .build_icmp_eq(state.builder, lhs, rhs, "eq");
            Ok(state.builder.build_not(eq, "neq").expect("not").into())
        }
        BinOp::Lt => Ok(state
            .ir_builder
            .build_icmp_lt(state.builder, lhs, rhs, "lt")
            .into()),
        BinOp::Gt => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gt")
            .expect("gt")
            .into()),
        BinOp::LtEq => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "lte")
            .expect("lte")
            .into()),
        BinOp::GtEq => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gte")
            .expect("gte")
            .into()),
        BinOp::Is => Ok(state
            .ir_builder
            .build_icmp_eq(state.builder, lhs, rhs, "is_cmp")
            .into()),
        BinOp::IsNot => {
            let eq = state
                .ir_builder
                .build_icmp_eq(state.builder, lhs, rhs, "isnot_cmp");
            Ok(state
                .builder
                .build_not(eq, "isnot_result")
                .expect("not")
                .into())
        }
        BinOp::Mod => Ok(state
            .builder
            .build_int_signed_rem(lhs, rhs, "mod")
            .expect("mod")
            .into()),
        BinOp::FloorDiv => Ok(state
            .ir_builder
            .build_div(state.builder, lhs, rhs, "floordiv")
            .into()),
        BinOp::BitAnd => Ok(state
            .builder
            .build_and(lhs, rhs, "bitand")
            .expect("bitand")
            .into()),
        BinOp::BitOr => Ok(state
            .builder
            .build_or(lhs, rhs, "bitor")
            .expect("bitor")
            .into()),
        BinOp::BitXor => Ok(state
            .builder
            .build_xor(lhs, rhs, "bitxor")
            .expect("bitxor")
            .into()),
        BinOp::LShift => Ok(state
            .builder
            .build_left_shift(lhs, rhs, "lshift")
            .expect("lshift")
            .into()),
        BinOp::RShift => Ok(state
            .builder
            .build_right_shift(lhs, rhs, false, "rshift")
            .expect("rshift")
            .into()),
        BinOp::Pow => Err("pow for ints not implemented".to_string()),
        _ => Err(format!("Unsupported int operator: {:?}", op)),
    }
}

/// Generate unary operation
fn generate_unary<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let val = generate_expr(state, operand)?;

    if val.is_float_value() {
        let float_val = val.into_float_value();
        match op {
            UnaryOp::Neg => Ok(state
                .builder
                .build_float_neg(float_val, "fneg")
                .expect("fneg")
                .into()),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Not | UnaryOp::Invert => Err(format!(
                "Unary operator {:?} not supported for float types",
                op
            )),
        }
    } else {
        let int_val = val.into_int_value();
        match op {
            UnaryOp::Neg => Ok(state
                .builder
                .build_int_neg(int_val, "neg")
                .expect("neg")
                .into()),
            UnaryOp::Not => Ok(state.builder.build_not(int_val, "not").expect("not").into()),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Invert => Ok(state
                .builder
                .build_xor(int_val, state.context.i64_type().const_all_ones(), "invert")
                .expect("invert")
                .into()),
        }
    }
}

/// Generate ternary conditional expression
fn generate_conditional<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    then_expr: &Expr,
    else_expr: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
    let cond_val = generate_expr(state, condition)?.into_int_value();

    let then_block = state.context.append_basic_block(func, "ternary_then");
    let else_block = state.context.append_basic_block(func, "ternary_else");
    let merge_block = state.context.append_basic_block(func, "ternary_end");

    let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
        cond_val
    } else {
        state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                cond_val,
                state.context.i64_type().const_zero(),
                "ternary_cond",
            )
            .expect("ternary_cond")
    };

    state
        .ir_builder
        .build_cond_branch(state.builder, cond_i1, then_block, else_block);

    state.builder.position_at_end(then_block);
    let then_val = generate_expr(state, then_expr)?;
    let then_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(else_block);
    let else_val = generate_expr(state, else_expr)?;
    let else_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(merge_block);
    let phi = state
        .builder
        .build_phi(then_val.get_type(), "ternary_result")
        .expect("phi");
    phi.add_incoming(&[(&then_val, then_block_end), (&else_val, else_block_end)]);

    Ok(phi.as_basic_value())
}

/// Generate function/method call
fn generate_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    func: &Expr,
    args: &[Expr],
    _span: crate::utils::Span,
) -> Result<BasicValueEnum<'ctx>, String> {
    if let Expr::Attribute { obj, attr, .. } = func {
        return generate_method_call(state, obj, attr, args);
    }

    if let Expr::Ident(name, _) = func {
        if name == "print" {
            return generate_print_call(state, args);
        }

        if name == "len" {
            return generate_len_call(state, args);
        }

        if name == "str" {
            return generate_str_call(state, args);
        }

        // Math builtins
        if name == "sqrt" || name == "abs" || name == "ln" || name == "floor" {
            return generate_math_builtin(state, name, args);
        }

        // Concurrency builtins (Phase 3)
        if name == "chan" {
            return generate_chan_create(state, args);
        }
        if name == "send" {
            return generate_chan_send(state, args);
        }
        if name == "recv" {
            return generate_chan_recv(state, args);
        }
        if name == "WaitGroup" {
            return generate_waitgroup_create(state, args);
        }
        if name == "add" {
            return generate_waitgroup_add(state, args);
        }
        if name == "done" {
            return generate_waitgroup_done(state, args);
        }
        if name == "wait" {
            return generate_waitgroup_wait(state, args);
        }

        // Struct module builtins
        if name == "struct_pack" || name == "pack" {
            return generate_struct_pack(state, args);
        }
        if name == "struct_unpack" || name == "unpack" {
            return generate_struct_unpack(state, args);
        }

        if let Some(&func_val) = state.functions.get(name) {
            let arg_values: Vec<_> = args
                .iter()
                .map(|a| generate_expr(state, a).map(|v| v.into()))
                .collect::<Result<_, _>>()?;

            let result = state
                .ir_builder
                .build_call(state.builder, func_val, &arg_values, "call");
            return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
        }
    }

    // Not a direct named function call or it's a variable reference
    let var_val = generate_expr(state, func).map_err(|e| format!("Call target failed: {}", e))?;
    if var_val.is_pointer_value() {
        let arg_values: Vec<_> = args
            .iter()
            .map(|a| generate_expr(state, a).map(|v| v.into()))
            .collect::<Result<_, _>>()?;
            
        let i64_type = state.context.i64_type();
        let mut param_types = Vec::new();
        for _ in args {
            param_types.push(i64_type.into());
        }
        let fn_type = i64_type.fn_type(&param_types, false);
        let result = state.builder.build_indirect_call(fn_type, var_val.into_pointer_value(), &arg_values, "indirect_call").expect("indirect call");
        match result.try_as_basic_value() {
            inkwell::values::ValueKind::Basic(basic_val) => return Ok(basic_val),
            _ => return Ok(state.ir_builder.i64_const(0).into()),
        }
    }

    return Err(format!("Call target is not a function: {:?}", func));
}

/// Generate print call
fn generate_print_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Ok(state.ir_builder.i64_const(0).into());
    }

    let val = generate_expr(state, &args[0])?;

    if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 64 {
        let print_func = state
            .module
            .get_function("vp_print_i64")
            .ok_or_else(|| "vp_print_i64 not declared".to_string())?;
        state
            .builder
            .build_call(print_func, &[val.into()], "print_i64")
            .expect("vp_print_i64");

        let newline_func = state
            .module
            .get_function("vp_print_newline")
            .ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state
            .builder
            .build_call(newline_func, &[], "print_newline")
            .expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else if val.is_float_value() {
        let print_func = state
            .module
            .get_function("vp_print_f64")
            .ok_or_else(|| "vp_print_f64 not declared".to_string())?;
        state
            .builder
            .build_call(print_func, &[val.into()], "print_f64")
            .expect("vp_print_f64");

        let newline_func = state
            .module
            .get_function("vp_print_newline")
            .ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state
            .builder
            .build_call(newline_func, &[], "print_newline")
            .expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
        let print_func = state
            .module
            .get_function("vp_print_bool")
            .ok_or_else(|| "vp_print_bool not declared".to_string())?;
        state
            .builder
            .build_call(print_func, &[val.into()], "print_bool")
            .expect("vp_print_bool");

        let newline_func = state
            .module
            .get_function("vp_print_newline")
            .ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state
            .builder
            .build_call(newline_func, &[], "print_newline")
            .expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else if val.is_pointer_value() {
        let print_func = state
            .module
            .get_function("vp_print_str")
            .ok_or_else(|| "vp_print_str not declared".to_string())?;
        state
            .builder
            .build_call(print_func, &[val.into()], "print_str")
            .expect("vp_print_str");

        let newline_func = state
            .module
            .get_function("vp_print_newline")
            .ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state
            .builder
            .build_call(newline_func, &[], "print_newline")
            .expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else {
        return Err(format!(
            "print() does not support type {:?}",
            val.get_type()
        ));
    }
}

/// Generate len() call
fn generate_len_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "len() takes exactly 1 argument, got {}",
            args.len()
        ));
    }

    let obj_val = generate_expr(state, &args[0])?;

    // Check if it's a string (pointer type)
    if obj_val.is_pointer_value() {
        // Call vp_str_len for strings
        let str_len = state
            .module
            .get_function("vp_str_len")
            .ok_or_else(|| "vp_str_len not declared".to_string())?;
        let result =
            state
                .ir_builder
                .build_call(state.builder, str_len, &[obj_val.into()], "str_len");
        return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
    }

    // Otherwise treat as list
    let list_len = state
        .module
        .get_function("vp_list_len")
        .ok_or_else(|| "vp_list_len not declared".to_string())?;
    let result =
        state
            .ir_builder
            .build_call(state.builder, list_len, &[obj_val.into()], "list_len");
    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate str() call - convert value to string
fn generate_str_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "str() takes exactly 1 argument, got {}",
            args.len()
        ));
    }

    let arg_val = generate_expr(state, &args[0])?;

    let func_name = if arg_val.is_float_value() {
        "vp_str_from_f64"
    } else if arg_val.is_pointer_value() {
        return Ok(arg_val);
    } else {
        "vp_str_from_i64"
    };

    let str_func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result = state
        .ir_builder
        .build_call(state.builder, str_func, &[arg_val.into()], "str_conv")
        .expect("str conversion call");

    Ok(result.into())
}

/// Generate math builtin function calls (sqrt, abs, ln, floor)
fn generate_math_builtin<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "{}() takes exactly 1 argument, got {}",
            name,
            args.len()
        ));
    }

    let arg_val = generate_expr(state, &args[0])?;

    // Convert to float if necessary
    let arg_float = if arg_val.is_float_value() {
        arg_val.into_float_value()
    } else {
        let int_val = arg_val.into_int_value();
        state
            .builder
            .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion")
    };

    let func_name = match name {
        "sqrt" => "vp_math_sqrt",
        "abs" => "vp_math_abs",
        "ln" => "vp_math_ln",
        "floor" => "vp_math_floor",
        _ => return Err(format!("Unknown math builtin: {}", name)),
    };

    let math_func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result =
        state
            .ir_builder
            .build_call(state.builder, math_func, &[arg_float.into()], "math_result");
    Ok(result.unwrap_or(state.ir_builder.f64_const(0.0).into()))
}

/// Generate struct.pack call
fn generate_struct_pack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 {
        return Err("struct.pack requires at least 2 arguments (format, value)".to_string());
    }

    // Generate format string (first arg)
    let format_expr = &args[0];
    let format_val = generate_expr(state, format_expr)?;
    let format_ptr = format_val.into_pointer_value();

    // Generate value (second arg)
    let value_expr = &args[1];
    let value_val = generate_expr(state, value_expr)?;
    let value_int = if value_val.is_int_value() {
        value_val.into_int_value()
    } else if value_val.is_float_value() {
        let float_val = value_val.into_float_value();
        state
            .builder
            .build_float_to_signed_int(float_val, state.context.i64_type(), "float_to_int")
            .expect("float to int")
    } else {
        return Err("Unsupported type for struct.pack".to_string());
    };

    let struct_pack_func = state
        .module
        .get_function("vp_struct_pack")
        .ok_or_else(|| "vp_struct_pack not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        struct_pack_func,
        &[format_ptr.into(), value_int.into()],
        "struct_pack_result",
    );

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate struct.unpack call
fn generate_struct_unpack<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() < 2 {
        return Err("struct.unpack requires at least 2 arguments (format, data)".to_string());
    }

    // Generate format string (first arg)
    let format_expr = &args[0];
    let format_val = generate_expr(state, format_expr)?;
    let format_ptr = format_val.into_pointer_value();

    // Generate data pointer (second arg)
    let data_expr = &args[1];
    let data_val = generate_expr(state, data_expr)?;
    let data_ptr = data_val.into_pointer_value();

    let struct_unpack_func = state
        .module
        .get_function("vp_struct_unpack")
        .ok_or_else(|| "vp_struct_unpack not declared".to_string())?;

    let len_val = state.context.i64_type().const_int(0, false);

    let result = state.ir_builder.build_call(
        state.builder,
        struct_unpack_func,
        &[format_ptr.into(), data_ptr.into(), len_val.into()],
        "struct_unpack_result",
    );

    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate method call
fn generate_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    method_name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let obj_val = generate_expr(state, obj)?;

    match method_name {
        "append" => {
            if args.len() != 1 {
                return Err(format!(
                    "append() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let val = generate_expr(state, &args[0])?.into_int_value();
            let list_append = state
                .module
                .get_function("vp_list_append")
                .ok_or_else(|| "vp_list_append not declared".to_string())?;
            state.ir_builder.build_call(
                state.builder,
                list_append,
                &[obj_val.into(), val.into()],
                "list_append",
            );
            Ok(state.ir_builder.i64_const(0).into())
        }
        "insert" => {
            if args.len() != 2 {
                return Err(format!(
                    "insert() takes exactly 2 arguments, got {}",
                    args.len()
                ));
            }
            let index = generate_expr(state, &args[0])?.into_int_value();
            let val = generate_expr(state, &args[1])?.into_int_value();
            let list_insert = state
                .module
                .get_function("vp_list_insert")
                .ok_or_else(|| "vp_list_insert not declared".to_string())?;
            state.ir_builder.build_call(
                state.builder,
                list_insert,
                &[obj_val.into(), index.into(), val.into()],
                "list_insert",
            );
            Ok(state.ir_builder.i64_const(0).into())
        }
        "remove" => {
            if args.len() != 1 {
                return Err(format!(
                    "remove() takes exactly 1 argument, got {}",
                    args.len()
                ));
            }
            let index = generate_expr(state, &args[0])?.into_int_value();
            let list_remove = state
                .module
                .get_function("vp_list_remove")
                .ok_or_else(|| "vp_list_remove not declared".to_string())?;
            let result = state.ir_builder.build_call(
                state.builder,
                list_remove,
                &[obj_val.into(), index.into()],
                "list_remove",
            );
            Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
        }
        "pop" => {
            if !args.is_empty() {
                return Err(format!("pop() takes no arguments, got {}", args.len()));
            }
            let list_pop = state
                .module
                .get_function("vp_list_pop")
                .ok_or_else(|| "vp_list_pop not declared".to_string())?;
            let result =
                state
                    .ir_builder
                    .build_call(state.builder, list_pop, &[obj_val.into()], "list_pop");
            Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
        }
        "clear" => {
            if !args.is_empty() {
                return Err(format!("clear() takes no arguments, got {}", args.len()));
            }
            let list_clear = state
                .module
                .get_function("vp_list_clear")
                .ok_or_else(|| "vp_list_clear not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, list_clear, &[obj_val.into()], "list_clear");
            Ok(state.ir_builder.i64_const(0).into())
        }
        "upper" => {
            if !args.is_empty() {
                return Err("upper() takes no arguments".to_string());
            }
            let func = state.module.get_function("vp_str_upper").unwrap();
            let result = state.ir_builder.build_call(state.builder, func, &[obj_val.into()], "str_upper");
            Ok(result.unwrap())
        }
        "lower" => {
            if !args.is_empty() {
                return Err("lower() takes no arguments".to_string());
            }
            let func = state.module.get_function("vp_str_lower").unwrap();
            let result = state.ir_builder.build_call(state.builder, func, &[obj_val.into()], "str_lower");
            Ok(result.unwrap())
        }
        "split" => {
            if args.len() != 1 {
                return Err("split() takes exactly 1 argument".to_string());
            }
            let delim_val = generate_expr(state, &args[0])?;
            let func = state.module.get_function("vp_str_split").unwrap();
            let result = state.ir_builder.build_call(state.builder, func, &[obj_val.into(), delim_val.into()], "str_split");
            Ok(result.unwrap())
        }
        "replace" => {
            if args.len() != 2 {
                return Err("replace() takes exactly 2 arguments".to_string());
            }
            let old_val = generate_expr(state, &args[0])?;
            let new_val = generate_expr(state, &args[1])?;
            let func = state.module.get_function("vp_str_replace").unwrap();
            let result = state.ir_builder.build_call(state.builder, func, &[obj_val.into(), old_val.into(), new_val.into()], "str_replace");
            Ok(result.unwrap())
        }
        "len" => Err("len() is a builtin function, not a method".to_string()),
        _ => Err(format!("Unknown method: {}", method_name)),
    }
}

/* ============================================ */
/* Concurrency Builtins (Phase 3)               */
/* ============================================ */

/// Generate chan(size) - create a channel
fn generate_chan_create<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "chan() takes 1 argument (capacity), got {}",
            args.len()
        ));
    }

    let size_val = generate_expr(state, &args[0])?;
    let chan_func = state
        .module
        .get_function("vp_chan_create")
        .ok_or_else(|| "vp_chan_create not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, chan_func, &[size_val.into()], "chan");
    // Return the pointer value directly
    Ok(result.expect("chan_create"))
}

/// Generate send(chan, value) - send to channel
fn generate_chan_send<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "send() takes 2 arguments (chan, value), got {}",
            args.len()
        ));
    }

    let chan_val = generate_expr(state, &args[0])?;
    let val_val = generate_expr(state, &args[1])?;
    let send_func = state
        .module
        .get_function("vp_chan_send")
        .ok_or_else(|| "vp_chan_send not declared".to_string())?;

    state.ir_builder.build_call(
        state.builder,
        send_func,
        &[chan_val.into(), val_val.into()],
        "send",
    );
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate recv(chan) - receive from channel
fn generate_chan_recv<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!(
            "recv() takes 1 argument (chan), got {}",
            args.len()
        ));
    }

    let chan_val = generate_expr(state, &args[0])?;
    let recv_func = state
        .module
        .get_function("vp_chan_recv")
        .ok_or_else(|| "vp_chan_recv not declared".to_string())?;

    let result =
        state
            .ir_builder
            .build_call(state.builder, recv_func, &[chan_val.into()], "recv_val");
    // Return the pointer value directly (received value is a pointer)
    Ok(result.expect("recv"))
}

/// Generate WaitGroup() - create a wait group
fn generate_waitgroup_create<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if !args.is_empty() {
        return Err(format!(
            "WaitGroup() takes no arguments, got {}",
            args.len()
        ));
    }

    let wg_func = state
        .module
        .get_function("vp_waitgroup_create")
        .ok_or_else(|| "vp_waitgroup_create not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, wg_func, &[], "wg");
    // Return the pointer value directly
    Ok(result.expect("wg_create"))
}

/// Generate add(wg, n) - add to wait group
fn generate_waitgroup_add<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 2 {
        return Err(format!(
            "add() takes 2 arguments (wg, n), got {}",
            args.len()
        ));
    }

    let wg_val = generate_expr(state, &args[0])?;
    let n_val = generate_expr(state, &args[1])?;
    let add_func = state
        .module
        .get_function("vp_waitgroup_add")
        .ok_or_else(|| "vp_waitgroup_add not declared".to_string())?;

    state.ir_builder.build_call(
        state.builder,
        add_func,
        &[wg_val.into(), n_val.into()],
        "wg_add",
    );
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate done(wg) - signal done on wait group
fn generate_waitgroup_done<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("done() takes 1 argument (wg), got {}", args.len()));
    }

    let wg_val = generate_expr(state, &args[0])?;
    let done_func = state
        .module
        .get_function("vp_waitgroup_done")
        .ok_or_else(|| "vp_waitgroup_done not declared".to_string())?;

    state
        .ir_builder
        .build_call(state.builder, done_func, &[wg_val.into()], "wg_done");
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate wait(wg) - wait on wait group
fn generate_waitgroup_wait<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("wait() takes 1 argument (wg), got {}", args.len()));
    }

    let wg_val = generate_expr(state, &args[0])?;
    let wait_func = state
        .module
        .get_function("vp_waitgroup_wait")
        .ok_or_else(|| "vp_waitgroup_wait not declared".to_string())?;

    state
        .ir_builder
        .build_call(state.builder, wait_func, &[wg_val.into()], "wg_wait");
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate await expression - suspend until future is ready
fn generate_await<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    future: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    // For now, generate a simple call to vp_future_await
    // A full implementation would transform the async function into a state machine
    let future_val = generate_expr(state, future)?;

    let await_func = state
        .module
        .get_function("vp_future_await")
        .ok_or_else(|| "vp_future_await not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        await_func,
        &[future_val.into()],
        "await_result",
    );

    Ok(result.unwrap())
}
