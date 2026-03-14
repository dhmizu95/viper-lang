//! List creation and list comprehension for Viper

use inkwell::values::BasicValueEnum;

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::VarInfo;

use crate::codegen::expressions::generate_expr;

/// Generate list creation
pub fn generate_list<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Determine element type
    let is_float_list = elements.first().map(|e| matches!(e, Expr::Float(..))).unwrap_or(false);
    let is_bool_list = elements.first().map(|e| matches!(e, Expr::Bool(..))).unwrap_or(false);
    
    eprintln!("DEBUG generate_list: is_float_list={}, is_bool_list={}, first_elem={:?}", is_float_list, is_bool_list, elements.first());

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

/// Generate list comprehension: [expr for var in iter]
/// Currently supports: [expr for var in range(n)] and [expr for var in range(start, end)]
pub fn generate_list_comprehension<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    element: &Expr,
    var: &str,
    iter: &Expr,
    _span: crate::utils::Span,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Determine element type by analyzing the element expression
    let elem_type = crate::codegen::expressions::infer_expr_type(element);
    let is_float_list = matches!(elem_type, crate::ast::Type::F64);
    let is_bool_list = matches!(elem_type, crate::ast::Type::Bool);

    let (list_func_name, append_func_name) = if is_float_list {
        ("vp_list_create_f64", "vp_list_append_f64")
    } else if is_bool_list {
        ("vp_bitvec_create", "vp_bitvec_append") // Use bit vector for bool lists
    } else {
        ("vp_list_create", "vp_list_append")
    };

    // Create result list
    let list_func = state
        .module
        .get_function(list_func_name)
        .ok_or_else(|| format!("{} not declared", list_func_name))?;

    let result_list =
        state.ir_builder.build_call(state.builder, list_func, &[], "comp_result").unwrap();

    // Append function
    let append_func = state
        .module
        .get_function(append_func_name)
        .ok_or_else(|| format!("{} not declared", append_func_name))?;

    // Handle range() specially (like generate_for does)
    let (start_val, end_val) = if let Expr::Call { func, args, .. } = iter {
        if let Expr::Ident(name, _) = func.as_ref() {
            if name == "range" {
                match args.len() {
                    0 => {
                        return crate::codegen::codegen_error(
                            "range expected at least 1 argument, got 0".to_string(),
                        )
                    }
                    1 => (
                        state.ir_builder.i64_const(0),
                        generate_expr(state, &args[0])?.into_int_value(),
                    ),
                    _ => (
                        generate_expr(state, &args[0])?.into_int_value(),
                        generate_expr(state, &args[1])?.into_int_value(),
                    ),
                }
            } else {
                return crate::codegen::codegen_error(
                    "List comprehension only supports range() iterator".to_string(),
                );
            }
        } else {
            return crate::codegen::codegen_error(
                "List comprehension only supports range() iterator".to_string(),
            );
        }
    } else {
        return crate::codegen::codegen_error(
            "List comprehension only supports range() iterator".to_string(),
        );
    };

    // Create loop blocks
    let func = state
        .builder
        .get_insert_block()
        .ok_or("No insertion block")?
        .get_parent()
        .ok_or("No parent function")?;

    let init_block = state.context.append_basic_block(func, "list_comp_init");
    let cond_block = state.context.append_basic_block(func, "list_comp_cond");
    let body_block = state.context.append_basic_block(func, "list_comp_body");
    let step_block = state.context.append_basic_block(func, "list_comp_step");
    let after_loop_block = state.context.append_basic_block(func, "list_comp_after");

    // Branch to init block
    state.builder.build_unconditional_branch(init_block).expect("branch to init");

    // Init block: create counter variable
    state.builder.position_at_end(init_block);
    let counter =
        state.builder.build_alloca(state.context.i64_type(), "comp_counter").expect("alloca");
    state.builder.build_store(counter, start_val).expect("store counter");

    // Branch to condition
    state.builder.build_unconditional_branch(cond_block).expect("branch to cond");

    // Condition block
    state.builder.position_at_end(cond_block);

    // Load counter
    let counter_val = state
        .builder
        .build_load(state.context.i64_type(), counter, "counter_val")
        .expect("load counter")
        .into_int_value();

    // Check if counter < end
    let cond = state.ir_builder.build_icmp_lt(state.builder, counter_val, end_val, "comp_cond");

    // Branch based on condition: if true -> body, if false -> after
    state.ir_builder.build_cond_branch(state.builder, cond, body_block, after_loop_block);

    // Body block
    state.builder.position_at_end(body_block);

    // Create a separate variable for the loop variable (copy counter value)
    let var_ptr = state.builder.build_alloca(state.context.i64_type(), var).expect("alloca");
    let counter_val = state
        .builder
        .build_load(state.context.i64_type(), counter, "counter_for_var")
        .expect("load counter")
        .into_int_value();
    state.builder.build_store(var_ptr, counter_val).expect("store var");

    // Set up the loop variable in the symbol table
    let old_var = state.variables.insert(
        var.to_string(),
        VarInfo::new_stack(var_ptr, crate::codegen::variables::VarType::Int),
    );

    // Generate the element expression
    let elem_val = generate_expr(state, element)?;

    // Handle type conversions
    let elem_val = if is_float_list && elem_val.is_int_value() {
        let int_val = elem_val.into_int_value();
        state
            .builder
            .build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion")
            .into()
    } else if is_bool_list
        && elem_val.is_int_value()
        && elem_val.get_type().into_int_type().get_bit_width() > 1
    {
        // Convert i64 to bool for bool list (only if not already i1)
        let int_val = elem_val.into_int_value();
        state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                int_val,
                state.context.i64_type().const_zero(),
                "i64_to_bool",
            )
            .expect("i64 to bool conversion")
            .into()
    } else {
        elem_val
    };

    // Append to result list
    let _ = state.ir_builder.build_call(
        state.builder,
        append_func,
        &[result_list.into(), elem_val.into()],
        "list_append",
    );

    // Restore the variable after body
    if let Some(old) = old_var {
        state.variables.insert(var.to_string(), old);
    } else {
        state.variables.remove(var);
    }

    // Branch to step block
    state.builder.build_unconditional_branch(step_block).expect("branch to step");

    // Step block: increment counter
    state.builder.position_at_end(step_block);
    let counter_val = state
        .builder
        .build_load(state.context.i64_type(), counter, "counter_step")
        .expect("load counter")
        .into_int_value();
    let next_val = state.ir_builder.build_add(
        state.builder,
        counter_val,
        state.context.i64_type().const_int(1, false),
        "next_counter",
    );
    state.builder.build_store(counter, next_val).expect("store counter");

    // Branch back to condition
    state.builder.build_unconditional_branch(cond_block).expect("branch back to cond");

    // After loop
    state.builder.position_at_end(after_loop_block);

    Ok(result_list)
}
