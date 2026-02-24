//! Control flow handling for Viper code generation

use crate::ast::{Expr, Stmt};

use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{LoopContext, VarInfo, VarType};

/// Generate an if statement
pub fn generate_if<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    body: &[Stmt],
    elif_blocks: &[(Expr, Vec<Stmt>)],
    else_body: &Option<Vec<Stmt>>,
) -> Result<(), String> {
    let func = state.builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
    let cond_val = crate::codegen::expressions::generate_expr(state, condition)?.into_int_value();

    let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
        cond_val
    } else {
        state.builder
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

    state.ir_builder
        .build_cond_branch(state.builder, cond_i1, then_block, else_block);

    // Then block
    state.builder.position_at_end(then_block);
    for stmt in body {
        crate::codegen::statements::generate_stmt(
            state.context, state.module, state.builder, state.ir_builder,
            state.variables, state.functions, state.global_constants, state.loop_stack, stmt
        )?;
    }
    if state.builder
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
        let elif_cond_val = crate::codegen::expressions::generate_expr(state, elif_cond)?.into_int_value();
        let elif_then = state.context.append_basic_block(func, "elif_then");
        let elif_else = if elif_blocks.len() > 1 {
            state.context.append_basic_block(func, "elif_else")
        } else if else_body.is_some() {
            state.context.append_basic_block(func, "else")
        } else {
            merge_block
        };

        state.ir_builder
            .build_cond_branch(state.builder, elif_cond_val, elif_then, elif_else);

        state.builder.position_at_end(elif_then);
        for stmt in elif_body {
            crate::codegen::statements::generate_stmt(
                state.context, state.module, state.builder, state.ir_builder,
                state.variables, state.functions, state.global_constants, state.loop_stack, stmt
            )?;
        }
        if state.builder
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
                        state.context, state.module, state.builder, state.ir_builder,
                        state.variables, state.functions, state.global_constants, state.loop_stack, stmt
                    )?;
                }
                if state.builder
                    .get_insert_block()
                    .unwrap()
                    .get_terminator()
                    .is_none()
                {
                    state.ir_builder.build_branch(state.builder, merge_block);
                }
            } else {
                if state.builder
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
                state.context, state.module, state.builder, state.ir_builder,
                state.variables, state.functions, state.global_constants, state.loop_stack, stmt
            )?;
        }
        if state.builder
            .get_insert_block()
            .unwrap()
            .get_terminator()
            .is_none()
        {
            state.ir_builder.build_branch(state.builder, merge_block);
        }
    } else {
        if state.builder
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

/// Generate a while loop
pub fn generate_while<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    body: &[Stmt],
) -> Result<(), String> {
    let func = state.builder
        .get_insert_block()
        .unwrap()
        .get_parent()
        .unwrap();
    let cond_block = state.context.append_basic_block(func, "while_cond");
    let body_block = state.context.append_basic_block(func, "while_body");
    let exit_block = state.context.append_basic_block(func, "while_exit");

    state.ir_builder.build_branch(state.builder, cond_block);

    state.builder.position_at_end(cond_block);
    let cond_expr = crate::codegen::expressions::generate_expr(state, condition)?;
    let cond_val = cond_expr.into_int_value();
    let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
        cond_val
    } else {
        state.builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                cond_val,
                state.context.i64_type().const_zero(),
                "cond_bool",
            )
            .expect("icmp")
    };
    state.ir_builder
        .build_cond_branch(state.builder, cond_i1, body_block, exit_block);

    state.builder.position_at_end(body_block);
    state.loop_stack.push(LoopContext::new(exit_block, cond_block));

    for stmt in body {
        crate::codegen::statements::generate_stmt(
            state.context, state.module, state.builder, state.ir_builder,
            state.variables, state.functions, state.global_constants, state.loop_stack, stmt
        )?;
    }

    state.loop_stack.pop();
    state.ir_builder.build_branch(state.builder, cond_block);
    state.builder.position_at_end(exit_block);
    Ok(())
}

/// Generate a for loop (simplified: only handles range() calls)
pub fn generate_for<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    iter: &Expr,
    body: &[Stmt],
) -> Result<(), String> {
    if let Expr::Call { func, args, .. } = iter {
        if let Expr::Ident(name, _) = func.as_ref() {
            if name == "range" {
                let end_val = if args.len() == 1 {
                    crate::codegen::expressions::generate_expr(state, &args[0])?.into_int_value()
                } else {
                    state.ir_builder.i64_const(0)
                };

                let func_ctx = state.builder
                    .get_insert_block()
                    .unwrap()
                    .get_parent()
                    .unwrap();
                let init_block = state.context.append_basic_block(func_ctx, "for_init");
                let cond_block = state.context.append_basic_block(func_ctx, "for_cond");
                let body_block = state.context.append_basic_block(func_ctx, "for_body");
                let step_block = state.context.append_basic_block(func_ctx, "for_step");
                let exit_block = state.context.append_basic_block(func_ctx, "for_exit");

                state.ir_builder.build_branch(state.builder, init_block);
                state.builder.position_at_end(init_block);
                let counter = state.builder
                    .build_alloca(state.context.i64_type(), "for_counter")
                    .expect("alloca");
                state.builder
                    .build_store(counter, state.ir_builder.i64_const(0))
                    .expect("store");
                state.ir_builder.build_branch(state.builder, cond_block);

                state.builder.position_at_end(cond_block);
                let counter_val = state.builder
                    .build_load(state.context.i64_type(), counter, "counter_val")
                    .expect("load")
                    .into_int_value();
                let cond = state.ir_builder.build_icmp_lt(
                    state.builder,
                    counter_val,
                    end_val,
                    "for_cond",
                );
                state.ir_builder
                    .build_cond_branch(state.builder, cond, body_block, exit_block);

                state.builder.position_at_end(body_block);
                if let Expr::Ident(target_name, _) = target {
                    state.variables.insert(target_name.clone(), VarInfo::new_stack(counter, VarType::Int));
                }

                for stmt in body {
                    crate::codegen::statements::generate_stmt(
                        state.context, state.module, state.builder, state.ir_builder,
                        state.variables, state.functions, state.global_constants, state.loop_stack, stmt
                    )?;
                }

                state.ir_builder.build_branch(state.builder, step_block);

                state.builder.position_at_end(step_block);
                let counter_val = state.builder
                    .build_load(state.context.i64_type(), counter, "counter_val")
                    .expect("load")
                    .into_int_value();
                let next_val = state.ir_builder.build_add(
                    state.builder,
                    counter_val,
                    state.ir_builder.i64_const(1),
                    "next_counter",
                );
                state.builder.build_store(counter, next_val).expect("store");
                state.ir_builder.build_branch(state.builder, cond_block);

                state.builder.position_at_end(exit_block);
                return Ok(());
            }
        }
    }

    Ok(())
}

/// Generate a return statement
pub fn generate_return<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    value: &Option<Expr>,
) -> Result<(), String> {
    if let Some(val) = value {
        let v = crate::codegen::expressions::generate_expr(state, val)?;
        state.ir_builder.build_return(state.builder, Some(&v));
    } else {
        state.ir_builder.build_return(state.builder, None);
    }
    Ok(())
}

/// Generate a break statement
pub fn generate_break<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &crate::codegen::builder::IRBuilder<'ctx>,
    loop_stack: &[LoopContext<'ctx>],
) -> Result<(), String> {
    if let Some(loop_ctx) = loop_stack.last() {
        ir_builder.build_branch(builder, loop_ctx.break_block);
        Ok(())
    } else {
        Err("break statement outside of loop".to_string())
    }
}

/// Generate a continue statement
pub fn generate_continue<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &crate::codegen::builder::IRBuilder<'ctx>,
    loop_stack: &[LoopContext<'ctx>],
) -> Result<(), String> {
    if let Some(loop_ctx) = loop_stack.last() {
        ir_builder.build_branch(builder, loop_ctx.continue_block);
        Ok(())
    } else {
        Err("continue statement outside of loop".to_string())
    }
}
