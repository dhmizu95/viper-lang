use crate::ast::{Expr, UnaryOp};
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarStorage, VarType};
use inkwell::values::BasicValueEnum;

/// Generate increment/decrement operation
pub fn generate_incdec<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Increment/decrement only works on variables
    let (name, alloca, var_type) = match operand {
        Expr::Ident(name, _) => {
            if let Some(var_info) = state.variables.get(name) {
                match &var_info.storage {
                    VarStorage::Stack(alloca) => (name, *alloca, var_info.var_type),
                    VarStorage::Register(value) => {
                        // For register-allocated variables, we need to create an alloca
                        // and store the value there, then use that for inc/dec operations
                        let alloca = state
                            .builder
                            .build_alloca(value.get_type(), &format!("{}_incdec", name))
                            .expect("alloca");
                        state.builder.build_store(alloca, *value).expect("store");
                        (name, alloca, var_info.var_type)
                    }
                    VarStorage::ClosureCell(_) => {
                        // For closure cell variables, use the value pointer
                        if let Some(value_ptr) = &var_info.closure_value_ptr {
                            (name, *value_ptr, var_info.var_type)
                        } else {
                            return Err(format!("Closure cell for '{}' missing value pointer", name));
                        }
                    }
                }
            } else {
                return Err(format!("Undefined variable: {}", name));
            }
        }
        _ => {
            return Err("Increment/decrement only supported on variables".to_string());
        }
    };

    // Only support integer types for now
    if var_type != VarType::Int {
        return Err(format!(
            "Increment/decrement only supported on integer variables, found {:?}",
            var_type
        ));
    }

    let i64_type = state.context.i64_type();
    let one = i64_type.const_int(1, false);

    // Load current value
    let current = state.builder.build_load(i64_type, alloca, name).expect("load").into_int_value();

    // Calculate new value
    let new_val = match op {
        UnaryOp::PreIncrement | UnaryOp::PostIncrement => {
            state.builder.build_int_add(current, one, "inc").expect("inc")
        }
        UnaryOp::PreDecrement | UnaryOp::PostDecrement => {
            state.builder.build_int_sub(current, one, "dec").expect("dec")
        }
        _ => return Err("Expected Increment or Decrement".to_string()),
    };

    // Store new value
    state.builder.build_store(alloca, new_val).expect("store");

    // Return old value for postfix, new value for prefix
    match op {
        UnaryOp::PostIncrement | UnaryOp::PostDecrement => Ok(current.into()),
        _ => Ok(new_val.into()),
    }
}

/// Generate ternary conditional expression
pub fn generate_conditional<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    then_expr: &Expr,
    else_expr: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
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

    state.ir_builder.build_cond_branch(state.builder, cond_i1, then_block, else_block);

    state.builder.position_at_end(then_block);
    let then_val = generate_expr(state, then_expr)?;
    let then_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(else_block);
    let else_val = generate_expr(state, else_expr)?;
    let else_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(merge_block);
    let phi = state.builder.build_phi(then_val.get_type(), "ternary_result").expect("phi");
    phi.add_incoming(&[(&then_val, then_block_end), (&else_val, else_block_end)]);

    Ok(phi.as_basic_value())
}
