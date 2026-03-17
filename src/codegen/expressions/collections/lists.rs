use inkwell::values::BasicValueEnum;

use crate::ast::{Expr, Type};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarType};

use crate::codegen::expressions::{generate_expr, infer_expr_type};

/// Generate list creation
pub fn generate_list<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Determine element type by checking all elements or using type inference
    let is_float_list = elements.iter().any(|e| {
        if matches!(e, Expr::Float(..)) {
            return true;
        }
        if let Expr::Ident(name, _) = e {
            if let Some(ty) = state.var_types.get(name) {
                return matches!(ty, crate::ast::Type::F64);
            }
        }
        crate::codegen::expressions::infer_expr_type(e) == crate::ast::Type::F64
    });
    let is_bool_list = !is_float_list && elements.iter().any(|e| {
        if matches!(e, Expr::Bool(..)) {
            return true;
        }
        if let Expr::Ident(name, _) = e {
            if let Some(ty) = state.var_types.get(name) {
                return matches!(ty, crate::ast::Type::Bool);
            }
        }
        crate::codegen::expressions::infer_expr_type(e) == crate::ast::Type::Bool
    });

    // For empty lists or mixed types, check all elements
    let (list_func_name, append_func_name) = if is_float_list {
        ("vp_list_create_f64", "vp_list_append_f64")
    } else if is_bool_list {
        ("vp_bitvec_create", "vp_bitvec_append") // Use bit vector for bool lists
    } else {
        ("vp_list_create", "vp_list_append")
    };

    let list_func = state
        .module
        .get_function(list_func_name)
        .ok_or_else(|| format!("{} not declared", list_func_name))?;

    let list_val = state.ir_builder.build_call(state.builder, list_func, &[], "new_list").unwrap();

    let append_func = state
        .module
        .get_function(append_func_name)
        .ok_or_else(|| format!("{} not declared", append_func_name))?;

    for (idx, elem) in elements.iter().enumerate() {
        let mut elem_val = generate_expr(state, elem)?;

        // If float list but elem is int, convert to float
        if is_float_list && elem_val.is_int_value() {
            let int_val = elem_val.into_int_value();
            let float_val = state
                .builder
                .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
                .expect("int to float conversion");
            elem_val = float_val.into();
        } else if is_bool_list
            && elem_val.is_int_value()
            && elem_val.get_type().into_int_type().get_bit_width() > 1
        {
            // Convert i64 to bool for bool list (only if not already i1)
            let int_val = elem_val.into_int_value();
            let bool_val = state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    state.context.i64_type().const_zero(),
                    "i64_to_bool",
                )
                .expect("i64 to bool conversion");
            elem_val = bool_val.into();
        }

        let _ = state.ir_builder.build_call(
            state.builder,
            append_func,
            &[list_val.into(), elem_val.into()],
            &format!("list_append_{}", idx),
        );
    }

    Ok(list_val)
}

/// Generate list comprehension: [expr for target in iter] or [expr for t1, t2 in iter if cond]
pub fn generate_list_comprehension<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    element: &Expr,
    target: &Expr,
    iter: &Expr,
    ifs: &[Expr],
    _span: crate::utils::Span,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Determine element type
    let elem_type = crate::codegen::expressions::infer_expr_type(element);
    let is_float_list = matches!(elem_type, Type::F64);
    let is_bool_list = matches!(elem_type, Type::Bool);

    let (list_func_name, append_func_name) = if is_float_list {
        ("vp_list_create_f64", "vp_list_append_f64")
    } else if is_bool_list {
        ("vp_bitvec_create", "vp_bitvec_append")
    } else {
        ("vp_list_create", "vp_list_append")
    };

    // Create result list
    let list_func = state.module.get_function(list_func_name)
        .ok_or_else(|| format!("{} not declared", list_func_name))?;
    let result_list = state.ir_builder.build_call(
        state.builder, list_func, &[], "comp_result"
    ).expect("list_create");

    // Append function
    let append_func = state.module.get_function(append_func_name)
        .ok_or_else(|| format!("{} not declared", append_func_name))?;

    // Analyze iterator
    let (is_range, start_val, end_val, iter_val, iter_is_float_list) = if let Expr::Call { func, args, .. } = iter {
        if let Expr::Ident(name, _) = func.as_ref() {
            if name == "range" {
                let (start, end) = match args.len() {
                    0 => return crate::codegen::codegen_error("range() requires at least 1 argument".to_string()),
                    1 => (
                        state.context.i64_type().const_int(0, false),
                        generate_expr(state, &args[0])?.into_int_value(),
                    ),
                    _ => (
                        generate_expr(state, &args[0])?.into_int_value(),
                        generate_expr(state, &args[1])?.into_int_value(),
                    ),
                };
                (true, start, end, None, false)
            } else {
                let v = generate_expr(state, iter)?;
                let len_func = state.module.get_function("vp_list_len")
                    .ok_or("vp_list_len not declared")?;
                let len = state.ir_builder.build_call(
                    state.builder, len_func, &[v.into()], "iter_len"
                ).unwrap().into_int_value();
                (false, state.context.i64_type().const_int(0, false), len, Some(v), false)
            }
        } else {
            let v = generate_expr(state, iter)?;
            let len_func = state.module.get_function("vp_list_len")
                .ok_or("vp_list_len not declared")?;
            let len = state.ir_builder.build_call(
                state.builder, len_func, &[v.into()], "iter_len"
            ).unwrap().into_int_value();
            (false, state.context.i64_type().const_int(0, false), len, Some(v), false)
        }
    } else {
        let v = generate_expr(state, iter)?;
        let len_func = state.module.get_function("vp_list_len")
            .ok_or("vp_list_len not declared")?;
        let len = state.ir_builder.build_call(
            state.builder, len_func, &[v.into()], "iter_len"
        ).unwrap().into_int_value();
        (false, state.context.i64_type().const_int(0, false), len, Some(v), false)
    };

    // Extract target variable names
    let target_names: Vec<String> = match target {
        Expr::Ident(name, _) => vec![name.clone()],
        Expr::Tuple { elements, .. } => {
            elements.iter().filter_map(|e| {
                if let Expr::Ident(name, _) = e {
                    Some(name.clone())
                } else {
                    None
                }
            }).collect()
        }
        _ => vec!["var".to_string()],
    };

    // Create loop blocks
    let func = state.builder.get_insert_block()
        .ok_or("No insertion block")?
        .get_parent()
        .ok_or("No parent function")?;

    let init_block = state.context.append_basic_block(func, "comp_init");
    let cond_block = state.context.append_basic_block(func, "comp_cond");
    let body_block = state.context.append_basic_block(func, "comp_body");
    let step_block = state.context.append_basic_block(func, "comp_step");
    let after_block = state.context.append_basic_block(func, "comp_after");

    // Branch to init
    state.ir_builder.build_branch(state.builder, init_block);

    // Init: counter = start
    state.builder.position_at_end(init_block);
    let counter = state.builder.build_alloca(state.context.i64_type(), "comp_counter").expect("alloca");
    state.builder.build_store(counter, start_val).expect("store");
    state.ir_builder.build_branch(state.builder, cond_block);

    // Condition: counter < end
    state.builder.position_at_end(cond_block);
    let counter_val = state.builder.build_load(state.context.i64_type(), counter, "counter_val")
        .expect("load")
        .into_int_value();
    let cond = state.ir_builder.build_icmp_lt(state.builder, counter_val, end_val, "comp_cond");
    state.ir_builder.build_cond_branch(state.builder, cond, body_block, after_block);

    // Body: bind variables, check filters, generate element, append
    state.builder.position_at_end(body_block);

    // Bind loop variables
    for (idx, var_name) in target_names.iter().enumerate() {
        let var_val = if idx == 0 {
            // First variable is the index/counter
            counter_val.into()
        } else if is_range {
            counter_val.into()
        } else if let Some(list_val) = iter_val {
            // Fetch from list
            let get_func_name = if iter_is_float_list { "vp_list_get_f64" } else { "vp_list_get" };
            let get_func = state.module.get_function(get_func_name)
                .ok_or_else(|| format!("{} not declared", get_func_name))?;
            state.ir_builder.build_call(
                state.builder, get_func,
                &[list_val.into(), counter_val.into()],
                "elem"
            ).unwrap()
        } else {
            counter_val.into()
        };

        let var_type = if idx == 0 { VarType::Int } else if iter_is_float_list { VarType::Float } else { VarType::Int };
        let storage_type: inkwell::types::BasicTypeEnum = if matches!(var_type, VarType::Float) {
            state.context.f64_type().into()
        } else {
            state.context.i64_type().into()
        };

        let var_ptr = state.builder.build_alloca(storage_type, var_name).expect("alloca");
        state.builder.build_store(var_ptr, var_val).expect("store");
        state.variables.insert(var_name.clone(), VarInfo::new_stack(var_ptr, var_type));
    }

    // Check filter conditions
    if !ifs.is_empty() {
        let filter_block = state.context.append_basic_block(func, "comp_filter");
        let pass_block = state.context.append_basic_block(func, "comp_pass");
        let fail_block = state.context.append_basic_block(func, "comp_fail");

        state.ir_builder.build_branch(state.builder, filter_block);

        state.builder.position_at_end(filter_block);
        let mut combined_cond: Option<inkwell::values::IntValue> = None;
        for if_expr in ifs {
            let cond_val = generate_expr(state, if_expr)?;
            let cond_i1 = if cond_val.is_int_value() {
                state.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    cond_val.into_int_value(),
                    state.context.i64_type().const_zero(),
                    "if_cond"
                ).expect("compare")
            } else {
                cond_val.into_int_value()
            };

            combined_cond = Some(if let Some(prev) = combined_cond {
                state.builder.build_and(prev, cond_i1, "combined_if").expect("and")
            } else {
                cond_i1
            });
        }

        if let Some(cond) = combined_cond {
            state.ir_builder.build_cond_branch(state.builder, cond, pass_block, fail_block);
        } else {
            state.ir_builder.build_branch(state.builder, pass_block);
        }

        // Fail block: skip append
        state.builder.position_at_end(fail_block);
        state.ir_builder.build_branch(state.builder, step_block);

        // Pass block: continue to element generation
        state.builder.position_at_end(pass_block);
    }

    // Generate element expression
    let elem_val = generate_expr(state, element)?;

    // Type conversion for append
    let elem_val = if is_float_list && elem_val.is_int_value() {
        state.builder.build_signed_int_to_float(
            elem_val.into_int_value(),
            state.context.f64_type(),
            "int_to_float"
        ).expect("int_to_float").into()
    } else if is_bool_list && elem_val.is_int_value() {
        state.builder.build_int_compare(
            inkwell::IntPredicate::NE,
            elem_val.into_int_value(),
            state.context.i64_type().const_zero(),
            "to_bool"
        ).expect("to_bool").into()
    } else {
        elem_val
    };

    // Append to result list
    let _ = state.ir_builder.build_call(
        state.builder, append_func,
        &[result_list.into(), elem_val.into()],
        "append"
    );

    // Remove loop variables from symbol table
    for var_name in &target_names {
        state.variables.remove(var_name);
    }

    // Branch to step
    state.ir_builder.build_branch(state.builder, step_block);

    // Step: counter++
    state.builder.position_at_end(step_block);
    let counter_val = state.builder.build_load(state.context.i64_type(), counter, "counter_step")
        .expect("load")
        .into_int_value();
    let next = state.ir_builder.build_add(
        state.builder, counter_val,
        state.context.i64_type().const_int(1, false),
        "next"
    );
    state.builder.build_store(counter, next).expect("store");
    state.ir_builder.build_branch(state.builder, cond_block);

    // After loop
    state.builder.position_at_end(after_block);

    Ok(result_list.into())
}
