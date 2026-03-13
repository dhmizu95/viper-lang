use crate::ast::{BinOp, Expr};
use crate::codegen::expressions::core::generate_expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate logical AND/OR with short-circuiting
pub fn generate_logical_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // Save the block where we evaluate lhs - this is where we'll branch from
    let lhs_block = state.builder.get_insert_block().unwrap();

    let lhs_val = generate_expr(state, left)?.into_int_value();

    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();

    let is_and = *op == BinOp::And;

    // For AND: if lhs is true, evaluate rhs; if false, short-circuit to end
    // For OR: if lhs is false, evaluate rhs; if true, short-circuit to end
    let evaluate_rhs_block = state.context.append_basic_block(func, "logic_evaluate_rhs");
    let end_block = state.context.append_basic_block(func, "logic_end");

    // Branch based on lhs value
    state
        .builder
        .build_conditional_branch(
            lhs_val,
            if is_and { evaluate_rhs_block } else { end_block },
            if is_and { end_block } else { evaluate_rhs_block },
        )
        .expect("branch");

    // Evaluate rhs block
    state.builder.position_at_end(evaluate_rhs_block);
    let rhs_val = generate_expr(state, right)?.into_int_value();
    state.builder.build_unconditional_branch(end_block).expect("branch");
    let rhs_block_end = state.builder.get_insert_block().unwrap();

    // Build phi in end block
    state.builder.position_at_end(end_block);
    let phi = state.builder.build_phi(state.context.bool_type(), "logic_result").expect("phi");

    // For both AND and OR:
    // - From lhs_block: if short-circuit happens (lhs decides result), use lhs_val
    // - From evaluate_rhs_block: use rhs_val
    phi.add_incoming(&[(&lhs_val, lhs_block), (&rhs_val, rhs_block_end)]);

    Ok(phi.as_basic_value())
}

/// Generate null coalescing operator (??)
/// Returns left operand if not null/None, otherwise returns right operand
pub fn generate_null_coalesce_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    right: &Expr,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    
    // Evaluate left operand
    let left_val = generate_expr(state, left)?;
    
    // Create blocks for right evaluation and merge
    let use_right_block = state.context.append_basic_block(func, "coalesce_use_right");
    let use_left_block = state.context.append_basic_block(func, "coalesce_use_left");
    let merge_block = state.context.append_basic_block(func, "coalesce_merge");
    
    // Check if left is null/None (zero for pointers, or special None value)
    let is_null = match left_val {
        inkwell::values::BasicValueEnum::PointerValue(ptr) => {
            let null_ptr = state.context.ptr_type(inkwell::AddressSpace::default()).const_null();
            state.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                state.builder.build_ptr_to_int(ptr, state.context.i64_type(), "ptr_to_int").expect("ptr to int"),
                state.builder.build_ptr_to_int(null_ptr, state.context.i64_type(), "null_to_int").expect("ptr to int"),
                "is_null",
            ).expect("cmp")
        }
        inkwell::values::BasicValueEnum::IntValue(i) => {
            // For integers, check if zero (None is represented as 0)
            state.builder.build_int_compare(
                inkwell::IntPredicate::EQ,
                i,
                state.context.i64_type().const_zero(),
                "is_null",
            ).expect("cmp")
        }
        _ => {
            // For other types, assume non-null
            state.context.bool_type().const_zero()
        }
    };
    
    // Branch based on null check
    state.builder.build_conditional_branch(
        is_null,
        use_right_block,  // if null, use right
        use_left_block,   // if not null, use left
    ).expect("coalesce branch");
    
    // Use left block (not null)
    state.builder.position_at_end(use_left_block);
    state.builder.build_unconditional_branch(merge_block).expect("branch to merge");
    let left_end_block = state.builder.get_insert_block().unwrap();
    
    // Use right block (is null)
    state.builder.position_at_end(use_right_block);
    let right_val = generate_expr(state, right)?;
    state.builder.build_unconditional_branch(merge_block).expect("branch to merge");
    let right_end_block = state.builder.get_insert_block().unwrap();
    
    // Merge block with phi node
    state.builder.position_at_end(merge_block);
    let phi = state.builder.build_phi(left_val.get_type(), "coalesce_result").expect("phi");
    phi.add_incoming(&[(&left_val, left_end_block), (&right_val, right_end_block)]);
    
    Ok(phi.as_basic_value())
}
