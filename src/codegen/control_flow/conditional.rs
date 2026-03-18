use crate::ast::{Expr, Stmt};
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Convert a value to a boolean (i1) for conditional branching.
/// For pointers (lists): non-empty = true, empty = false
/// For integers: non-zero = true, zero = false
/// For bools (i1): use directly
fn convert_to_bool<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
    name: &str,
) -> crate::codegen::Result<inkwell::values::IntValue<'ctx>> {
    match value {
        BasicValueEnum::PointerValue(ptr) => {
            // For lists, check if length > 0 (Python: empty collections are falsy)
            // ViperList struct: length is at offset 0 (i64)
            let i64_ptr = state
                .builder
                .build_pointer_cast(ptr, state.context.ptr_type(inkwell::AddressSpace::default()), &format!("{}_as_i64_ptr", name))
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to cast pointer: {:?}", e)))?;
            let length = state
                .builder
                .build_load(state.context.i64_type(), i64_ptr, &format!("{}_length", name))
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to load length: {:?}", e)))?
                .into_int_value();
            state
                .builder
                .build_int_compare(
                    inkwell::IntPredicate::NE,
                    length,
                    state.context.i64_type().const_zero(),
                    &format!("{}_bool", name),
                )
                .map_err(|e| crate::codegen::codegen_err(format!("Failed to compare length: {:?}", e)))
        }
        BasicValueEnum::IntValue(int_val) => {
            // For integers, check if non-zero
            if int_val.get_type().get_bit_width() == 1 {
                Ok(int_val)
            } else {
                state
                    .builder
                    .build_int_compare(
                        inkwell::IntPredicate::NE,
                        int_val,
                        state.context.i64_type().const_zero(),
                        &format!("{}_bool", name),
                    )
                    .map_err(|e| crate::codegen::codegen_err(format!("Failed to compare integer: {:?}", e)))
            }
        }
        _ => {
            // For other types, default to true (shouldn't happen in normal code)
            Ok(state.context.bool_type().const_all_ones())
        }
    }
}

/// Generate an if statement with elif chain support
/// Returns true if all paths terminate (return/break/continue)
fn generate_if_chain<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elif_blocks: &[(Expr, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>,
    merge_block: inkwell::basic_block::BasicBlock<'ctx>,
) -> crate::codegen::Result<bool> {
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
            if current_block != merge_block && current_block.get_terminator().is_none() {
                state.ir_builder.build_branch(state.builder, merge_block);
            }
            return Ok(false);
        }
    }

    // Process the first elif block
    let (elif_cond, elif_body) = &elif_blocks[0];
    let elif_cond_val = crate::codegen::expressions::generate_expr(state, elif_cond)?;

    // Convert condition to i1 (boolean) for branch
    let elif_cond_i1 = convert_to_bool(state, elif_cond_val, "elif_cond")?;

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
) -> crate::codegen::Result<()> {
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let cond_val = crate::codegen::expressions::generate_expr(state, condition)?;

    let cond_i1 = convert_to_bool(state, cond_val, "cond")?;

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
        state
            .builder
            .build_unreachable()
            .map_err(|e| format!("Failed to build unreachable: {:?}", e))?;
    } else {
        state.builder.position_at_end(merge_block);
    }
    Ok(())
}
