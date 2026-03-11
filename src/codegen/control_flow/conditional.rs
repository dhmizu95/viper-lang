use crate::ast::{Expr, Stmt};
use crate::codegen::state::CodeGenState;

/// Generate an if statement with elif chain support
/// Returns true if all paths terminate (return/break/continue)
fn generate_if_chain<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elif_blocks: &[(Expr, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>,
    merge_block: inkwell::basic_block::BasicBlock<'ctx>,
) -> Result<bool, String> {
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();

    if elif_blocks.is_empty() {
        // No more elif blocks, handle else body
        if let Some(else_stmts) = else_body {
            for stmt in else_stmts {
                crate::codegen::statements::core::dispatch::generate_stmt_internal(state, stmt)?;
            }
            let terminates = state.builder.get_insert_block().unwrap().get_terminator().is_some();
            if !terminates {
                state.ir_builder.build_branch(state.builder, merge_block);
            }
            return Ok(terminates);
        } else {
            // No else body - this means all elif conditions were false
            // We need to continue execution after the if/elif chain
            // The current block should branch to merge_block
            // But first check if we're already at the merge_block (shouldn't happen, but be safe)
            let current_block = state.builder.get_insert_block().unwrap();
            if current_block != merge_block {
                state.ir_builder.build_branch(state.builder, merge_block);
            }
            return Ok(false);
        }
    }

    // Process the first elif block
    let (elif_cond, elif_body) = &elif_blocks[0];
    let elif_cond_val =
        crate::codegen::expressions::generate_expr(state, elif_cond)?.into_int_value();

    // Convert condition to i1 (boolean) for branch
    let elif_cond_i1 = if elif_cond_val.get_type().get_bit_width() == 1 {
        elif_cond_val
    } else {
        state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                elif_cond_val,
                state.context.i64_type().const_zero(),
                "elif_cond_bool",
            )
            .map_err(|e| format!("Failed to build elif condition: {:?}", e))?
    };

    let elif_then = state.context.append_basic_block(func, "elif_then");
    // Recursively handle remaining elif blocks
    let remaining_elif = &elif_blocks[1..];
    let elif_else = if !remaining_elif.is_empty() || else_body.is_some() {
        state.context.append_basic_block(func, "elif_else")
    } else {
        merge_block
    };

    state.ir_builder.build_cond_branch(state.builder, elif_cond_i1, elif_then, elif_else);

    // Generate then block for this elif
    state.builder.position_at_end(elif_then);
    for stmt in elif_body {
        crate::codegen::statements::core::dispatch::generate_stmt_internal(state, stmt)?;
    }
    let then_terminates = state.builder.get_insert_block().unwrap().get_terminator().is_some();
    if !then_terminates {
        state.ir_builder.build_branch(state.builder, merge_block);
    }

    // Position at else block and recursively process remaining elif blocks
    state.builder.position_at_end(elif_else);
    let else_terminates = generate_if_chain(state, remaining_elif, else_body, merge_block)?;

    // All paths terminate only if both then and else terminate
    Ok(then_terminates && else_terminates)
}

/// Generate an if statement
pub fn generate_if<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    body: &[Stmt],
    elif_blocks: &[(Expr, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>,
) -> Result<(), String> {
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let cond_val = crate::codegen::expressions::generate_expr(state, condition)?.into_int_value();

    let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
        cond_val
    } else {
        state
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                cond_val,
                state.context.i64_type().const_zero(),
                "cond_bool",
            )
            .expect("icmp")
    };

    let then_block = state.context.append_basic_block(func, "then");
    let else_block = state.context.append_basic_block(func, "else");
    let merge_block = state.context.append_basic_block(func, "if_cont");

    state.ir_builder.build_cond_branch(state.builder, cond_i1, then_block, else_block);

    // Then block
    state.builder.position_at_end(then_block);
    for stmt in body {
        crate::codegen::statements::core::dispatch::generate_stmt_internal(state, stmt)?;
    }
    let then_terminates = state.builder.get_insert_block().unwrap().get_terminator().is_some();
    if !then_terminates {
        state.ir_builder.build_branch(state.builder, merge_block);
    }

    // Else block (handle elif chains)
    state.builder.position_at_end(else_block);
    let else_terminates = generate_if_chain(state, elif_blocks, else_body, merge_block)?;

    // Only position at merge_block if at least one path doesn't terminate
    // If both then and else terminate, merge_block is unreachable - add unreachable terminator
    if then_terminates && else_terminates {
        // Both paths terminate - merge_block is unreachable, but LLVM requires all blocks to have terminators
        state.builder.position_at_end(merge_block);
        state.builder.build_unreachable().map_err(|e| format!("Failed to build unreachable: {:?}", e))?;
    } else {
        state.builder.position_at_end(merge_block);
    }
    Ok(())
}
