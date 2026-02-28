use crate::ast::{Expr, Stmt};
use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue};
use std::collections::{HashMap, HashSet};

use super::*;
use crate::codegen::builder::IRBuilder;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{LoopContext, VarInfo};
use crate::semantic::escape_analysis::EscapeAnalyzer;

/// Generate code for a statement
pub fn generate_stmt<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &IRBuilder<'ctx>,
    variables: &mut HashMap<String, VarInfo<'ctx>>,
    functions: &HashMap<String, FunctionValue<'ctx>>,
    global_constants: &mut HashMap<String, GlobalValue<'ctx>>,
    loop_stack: &mut Vec<LoopContext<'ctx>>,
    list_vars: &mut HashSet<String>,
    dict_vars: &mut HashSet<String>,
    bool_list_vars: &mut HashSet<String>,
    bigint_vars: &mut HashSet<String>,
    stmt: &Stmt,
) -> Result<(), String> {
    let mut state = CodeGenState::new(
        context,
        module,
        builder,
        ir_builder,
        variables,
        functions,
        global_constants,
        loop_stack,
        list_vars,
        dict_vars,
        bool_list_vars,
        bigint_vars,
    );

    generate_stmt_internal(&mut state, stmt)
}

/// Generate code for a statement with escape analysis
pub fn generate_stmt_with_escape<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &IRBuilder<'ctx>,
    variables: &mut HashMap<String, VarInfo<'ctx>>,
    functions: &HashMap<String, FunctionValue<'ctx>>,
    global_constants: &mut HashMap<String, GlobalValue<'ctx>>,
    loop_stack: &mut Vec<LoopContext<'ctx>>,
    list_vars: &mut HashSet<String>,
    dict_vars: &mut HashSet<String>,
    bool_list_vars: &mut HashSet<String>,
    bigint_vars: &mut HashSet<String>,
    stmt: &Stmt,
    escape_analyzer: &mut EscapeAnalyzer,
    current_function: &str,
) -> Result<(), String> {
    let mut state = CodeGenState::with_escape_analysis(
        context,
        module,
        builder,
        ir_builder,
        variables,
        functions,
        global_constants,
        loop_stack,
        list_vars,
        dict_vars,
        bool_list_vars,
        bigint_vars,
        escape_analyzer,
        current_function,
    );

    generate_stmt_internal(&mut state, stmt)
}

/// Internal statement generation
pub(crate) fn generate_stmt_internal<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    stmt: &Stmt,
) -> Result<(), String> {
    match stmt {
        Stmt::Expr(expr) => {
            crate::codegen::expressions::generate_expr(state, expr)?;
        }
        Stmt::Declare { name, value, mutable, .. } => {
            generate_declare(state, name, *mutable, value)?;
        }
        Stmt::Global { names, .. } => {
            generate_global(state, names)?;
        }
        Stmt::Nonlocal { names, .. } => {
            generate_nonlocal(state, names)?;
        }
        Stmt::Const { name, value, .. } => {
            generate_const(state, name, value)?;
        }
        Stmt::Assign { target, value, .. } => {
            // Check for tuple unpacking
            if let Expr::Tuple { elements, .. } = target.as_ref() {
                generate_tuple_unpack(state, elements, value)?;
            } else {
                generate_assign(state, target, value)?;
            }
        }
        Stmt::AugAssign { target, op, value, .. } => {
            generate_aug_assign(state, target, op, value)?;
        }
        Stmt::Return { value, .. } => {
            return crate::codegen::control_flow::generate_return(state, value);
        }
        Stmt::If { condition, body, elif_blocks, else_body, .. } => {
            return crate::codegen::control_flow::generate_if(
                state,
                condition,
                body,
                elif_blocks,
                else_body,
            );
        }
        Stmt::While { condition, body, else_body, .. } => {
            return crate::codegen::control_flow::generate_while(state, condition, body, else_body);
        }
        Stmt::For { target, iter, body, else_body, is_async, .. } => {
            if *is_async {
                return crate::codegen::control_flow::generate_async_for(state, target, iter, body);
            }
            return crate::codegen::control_flow::generate_for(
                state, target, iter, body, else_body, false,
            );
        }
        Stmt::Function { .. } => {
            // Already handled in first pass
        }
        Stmt::Break(_) => {
            return crate::codegen::control_flow::generate_break(
                state.builder,
                state.ir_builder,
                state.loop_stack,
            );
        }
        Stmt::Continue(_) => {
            return crate::codegen::control_flow::generate_continue(
                state.builder,
                state.ir_builder,
                state.loop_stack,
            );
        }
        Stmt::Pass(_) => {
            // No-op
        }
        // Concurrency statements (Phase 3)
        Stmt::Sync { body, .. } => {
            return generate_sync(state, body);
        }
        Stmt::Task { call, span } => {
            return generate_task(state, call, span);
        }
        Stmt::Chan { size, .. } => {
            return generate_chan(state, size);
        }
        Stmt::Send { chan, value, .. } => {
            return generate_send(state, chan, value);
        }
        Stmt::Recv { chan, .. } => {
            return generate_recv(state, chan);
        }
        Stmt::WaitGroup { .. } => {
            return generate_waitgroup(state);
        }
        Stmt::WgAdd { wg, n, .. } => {
            return generate_wg_add(state, wg, n);
        }
        Stmt::WgDone { wg, .. } => {
            return generate_wg_done(state, wg);
        }
        Stmt::WgWait { wg, .. } => {
            return generate_wg_wait(state, wg);
        }
        Stmt::Match { subject, cases, span: _ } => {
            let subject_val = crate::codegen::expressions::generate_expr(state, subject)?;

            // Generate each case as a simple if statement
            for case in cases {
                let matches = generate_match_pattern(state, &case.pattern, subject_val)?;

                // Create blocks for then and else
                let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
                let then_bb = state.context.append_basic_block(func, "match_then");
                let else_bb = state.context.append_basic_block(func, "match_else");

                // Generate the conditional branch
                state.builder.build_conditional_branch(matches, then_bb, else_bb).unwrap();

                // Generate then block (case body)
                state.builder.position_at_end(then_bb);
                for stmt in &case.body {
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
                        state.dict_vars,
                        state.bool_list_vars,
                        state.bigint_vars,
                        stmt,
                    )?;
                }

                // If no terminator, add one to else
                if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    state.builder.build_unconditional_branch(else_bb).unwrap();
                }

                // Position at else for next case
                state.builder.position_at_end(else_bb);
            }
        }
        _ => {}
    }
    Ok(())
}
