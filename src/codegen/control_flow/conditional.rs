use crate::ast::{Expr, Stmt};
use crate::codegen::state::CodeGenState;

/// Generate an if statement
pub fn generate_if<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    body: &[Stmt],
    elif_blocks: &[(Expr, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>,
) -> Result<(), String> {
    let func = state
        .builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
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

    state
        .ir_builder
        .build_cond_branch(state.builder, cond_i1, then_block, else_block);

    // Then block
    state.builder.position_at_end(then_block);
    for stmt in body {
        crate::codegen::statements::generate_stmt(
            state.context,
            state.module,
            state.builder,
            state.ir_builder,
            state.variables,
            state.functions,
            state.global_constants,
            state.loop_stack,
            state.list_vars,
            stmt,
        )?;
    }
    if state
        .builder
        .get_insert_block()
        .unwrap()
        .get_terminator()
        .is_none()
    {
        state.ir_builder.build_branch(state.builder, merge_block);
    }

    // Else block (handle elif chains)
    state.builder.position_at_end(else_block);

    if !elif_blocks.is_empty() {
        let (elif_cond, elif_body) = &elif_blocks[0];
        let elif_cond_val =
            crate::codegen::expressions::generate_expr(state, elif_cond)?.into_int_value();
        let elif_then = state.context.append_basic_block(func, "elif_then");
        let elif_else = if elif_blocks.len() > 1 {
            state.context.append_basic_block(func, "elif_else")
        } else if else_body.is_some() {
            state.context.append_basic_block(func, "else")
        } else {
            merge_block
        };

        state
            .ir_builder
            .build_cond_branch(state.builder, elif_cond_val, elif_then, elif_else);

        state.builder.position_at_end(elif_then);
        for stmt in elif_body {
            crate::codegen::statements::generate_stmt(
                state.context,
                state.module,
                state.builder,
                state.ir_builder,
                state.variables,
                state.functions,
                state.global_constants,
                state.loop_stack,
                state.list_vars,
                stmt,
            )?;
        }
        if state
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            state.ir_builder.build_branch(state.builder, merge_block);
        }

        if elif_blocks.len() > 1 || else_body.is_some() {
            state.builder.position_at_end(elif_else);
            if let Some(else_stmts) = else_body {
                for stmt in else_stmts {
                    crate::codegen::statements::generate_stmt(
                        state.context,
                        state.module,
                        state.builder,
                        state.ir_builder,
                        state.variables,
                        state.functions,
                        state.global_constants,
                        state.loop_stack,
                        state.list_vars,
                        stmt,
                    )?;
                }
                if state
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    state.ir_builder.build_branch(state.builder, merge_block);
                }
            } else {
                if state
                    .builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    state.ir_builder.build_branch(state.builder, merge_block);
                }
            }
        }
    } else if let Some(else_stmts) = else_body {
        for stmt in else_stmts {
            crate::codegen::statements::generate_stmt(
                state.context,
                state.module,
                state.builder,
                state.ir_builder,
                state.variables,
                state.functions,
                state.global_constants,
                state.loop_stack,
                state.list_vars,
                stmt,
            )?;
        }
        if state
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            state.ir_builder.build_branch(state.builder, merge_block);
        }
    } else {
        if state
            .builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            state.ir_builder.build_branch(state.builder, merge_block);
        }
    }

    state.builder.position_at_end(merge_block);
    Ok(())
}
