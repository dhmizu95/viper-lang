//! Main dispatch functions for statement code generation.

use crate::ast::{Expr, Stmt, Type};
use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue};
use std::collections::{HashMap, HashSet};

use super::*;
use crate::codegen::builder::IRBuilder;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{LoopContext, VarInfo};
use crate::semantic::closure_analysis::ClosureAnalyzer;
use crate::semantic::escape_analysis::EscapeAnalyzer;

// Import helper functions from sibling modules
use crate::codegen::statements::assignment::{
    generate_assign, generate_aug_assign, generate_slice_assign, generate_tuple_unpack,
};
use crate::codegen::statements::concurrency::{
    generate_chan, generate_recv, generate_send, generate_sync, generate_task, generate_waitgroup,
    generate_wg_add, generate_wg_done, generate_wg_wait,
};
use crate::codegen::statements::declaration::{
    generate_const, generate_declare, generate_global, generate_nonlocal,
};
use crate::codegen::statements::patterns::generate_match_pattern;

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
    var_types: &mut HashMap<String, Type>,
    stmt: &Stmt,
) -> crate::codegen::Result<()> {
    let mut closure_cells = HashMap::new();
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
        var_types,
        &mut closure_cells,
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
    var_types: &mut HashMap<String, Type>,
    stmt: &Stmt,
    escape_analyzer: &mut EscapeAnalyzer,
    current_function: &str,
    current_class: Option<&str>,
) -> crate::codegen::Result<()> {
    let mut closure_cells = HashMap::new();
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
        var_types,
        escape_analyzer,
        current_function,
        &mut closure_cells,
    );
    state.current_class = current_class.map(|s| s.to_string());

    generate_stmt_internal(&mut state, stmt)
}

/// Generate code for a statement with escape analysis and closure analysis
pub fn generate_stmt_with_closure<'ctx>(
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
    var_types: &mut HashMap<String, Type>,
    stmt: &Stmt,
    escape_analyzer: &mut EscapeAnalyzer,
    current_function: &str,
    closure_analyzer: &ClosureAnalyzer,
    current_class: Option<&str>,
) -> crate::codegen::Result<()> {
    // Create a dummy closure_cells for single-statement generation
    // This is used for module-level statements where closure cells aren't needed
    let mut closure_cells = HashMap::new();

    let mut state = CodeGenState::with_closure_analysis(
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
        var_types,
        escape_analyzer,
        current_function,
        closure_analyzer,
        &mut closure_cells,
    );
    state.current_class = current_class.map(|s| s.to_string());

    generate_stmt_internal(&mut state, stmt)
}

/// Internal statement generation
pub(crate) fn generate_stmt_internal<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    stmt: &Stmt,
) -> crate::codegen::Result<()> {
    match stmt {
        Stmt::Expr(expr) => {
            crate::codegen::expressions::generate_expr(state, expr)?;
        }
        Stmt::Declare { name, value, mutable, type_ann, .. } => {
            generate_declare(state, name, *mutable, value, type_ann)?;
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
        Stmt::SliceAssign { obj, start, end, step, value, .. } => {
            generate_slice_assign(state, obj, start, end, step, value)?;
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
        Stmt::Import { module, alias, .. } => {
            generate_import(state, module, alias.as_deref())?;
        }
        Stmt::FromImport { module, names, .. } => {
            generate_from_import(state, module, names)?;
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

            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let end_bb = state.context.append_basic_block(func, "match_end");

            // Generate each case as a simple if statement
            for (i, case) in cases.iter().enumerate() {
                let matches = generate_match_pattern(state, &case.pattern, subject_val)?;

                // Create blocks for this case
                let then_bb = state.context.append_basic_block(func, &format!("match_case_{}", i));
                let next_else_bb = if i < cases.len() - 1 {
                    state.context.append_basic_block(func, &format!("match_else_{}", i))
                } else {
                    end_bb // Last case's else goes to end
                };

                // Generate the conditional branch
                state.builder.build_conditional_branch(matches, then_bb, next_else_bb).unwrap();

                // Generate then block (case body)
                state.builder.position_at_end(then_bb);

                // Save the current variable count to restore after case
                let saved_var_count = state.variables.len();

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
                        state.var_types,
                        stmt,
                    )?;
                }

                // Branch to end after case body
                if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    state.builder.build_unconditional_branch(end_bb).unwrap();
                }

                // Remove variables that were added in this case
                // (restore to the count before the case)
                let keys_to_remove: Vec<String> =
                    state.variables.keys().skip(saved_var_count).cloned().collect();
                for key in keys_to_remove {
                    state.variables.remove(&key);
                }

                // Position at else for next case
                state.builder.position_at_end(next_else_bb);
            }

            // Position at end
            state.builder.position_at_end(end_bb);
        }
        // New Python keyword statements - stub implementations for now
        Stmt::Assert { condition, message, span: _ } => {
            let cond_val = crate::codegen::expressions::generate_expr(state, condition)?.into_int_value();
            
            // Convert condition to i1 (boolean) for branch
            let is_true = if cond_val.get_type().get_bit_width() == 1 {
                cond_val
            } else {
                state.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    cond_val,
                    state.context.i64_type().const_int(0, false),
                    "assert_cond"
                ).unwrap()
            };
            
            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let fail_bb = state.context.append_basic_block(func, "assert_fail");
            let pass_bb = state.context.append_basic_block(func, "assert_pass");
            
            state.builder.build_conditional_branch(is_true, pass_bb, fail_bb).unwrap();
            
            // Failure path
            state.builder.position_at_end(fail_bb);
            let panic_func = state.module.get_function("viper_panic")
                .ok_or_else(|| "viper_panic not declared".to_string())?;
            
            let msg_ptr = if let Some(msg_expr) = message {
                // If message is a string literal, use it directly
                if let Expr::Str(s, _) = &**msg_expr {
                    state.builder.build_global_string_ptr(s, "assert_msg").unwrap().as_pointer_value()
                } else {
                    // Otherwise just use default
                    state.builder.build_global_string_ptr("Assertion failed", "assert_msg").unwrap().as_pointer_value()
                }
            } else {
                state.builder.build_global_string_ptr("Assertion failed", "assert_msg").unwrap().as_pointer_value()
            };
            
            state.builder.build_call(panic_func, &[msg_ptr.into()], "panic_call").unwrap();
            state.builder.build_unreachable().unwrap();
            
            // Success path
            state.builder.position_at_end(pass_bb);
        }
        Stmt::Delete { targets, span: _ } => {
            for target in targets {
                match target {
                    Expr::Ident(name, _) => {
                        let var = state.variables.get(name).cloned();
                        if let Some(var) = var {
                            if let Some(ptr) = var.get_alloca() {
                                let i64_type = state.context.i64_type();
                                let val = state.builder.build_load(i64_type, ptr, "del_val").unwrap();
                                
                                // Release the value (handles BigInts/Objects)
                                let release_func = state.module.get_function("tagged_int_release")
                                    .ok_or_else(|| "tagged_int_release not declared".to_string())?;
                                state.builder.build_call(release_func, &[val.into()], "release_call").unwrap();
                                
                                // Clear to None (0)
                                state.builder.build_store(ptr, state.context.i64_type().const_int(0, false)).unwrap();
                            }
                            state.variables.remove(name);
                        }
                    }
                    Expr::Index { obj, index, .. } => {
                        let obj_val = crate::codegen::expressions::generate_expr(state, obj)?;
                        let index_val = crate::codegen::expressions::generate_expr(state, index)?;
                        
                        let obj_type = crate::codegen::expressions::core::infer_type_with_state(state, obj);
                        
                        // Decide which removal function to use based on inferred type
                        if matches!(obj_type, Type::Dict(..)) {
                            let remove_func = state.module.get_function("vp_dict_remove")
                                .ok_or_else(|| "vp_dict_remove not declared".to_string())?;
                            
                            // vp_dict_remove(dict, key) - key must be un-tagged if it's a string
                            // Dictionaries currently only support strings as keys
                            state.builder.build_call(remove_func, &[obj_val.into(), index_val.into()], "del_dict_item").unwrap();
                        } else {
                            // Assume it's a list
                            let remove_func = state.module.get_function("vp_list_remove")
                                .ok_or_else(|| "vp_list_remove not declared".to_string())?;
                            
                            let result_call = state.builder.build_call(remove_func, &[obj_val.into(), index_val.into()], "del_list_item").unwrap();
                            let removed_val = match result_call.try_as_basic_value() {
                                inkwell::values::ValueKind::Basic(v) => v,
                                _ => panic!("Expected basic value from vp_list_remove"),
                            };
                            
                            // vp_list_remove returns the item, we must release it
                            let release_func = state.module.get_function("tagged_int_release")
                                .ok_or_else(|| "tagged_int_release not declared".to_string())?;
                            state.builder.build_call(release_func, &[removed_val.into()], "release_removed").unwrap();
                        }
                    }
                    _ => {}
                }
            }
        }
        Stmt::Raise { exception, cause, span: _ } => {
            generate_raise(state, exception.as_deref(), cause.as_deref())?;
        }
        Stmt::Try { body, handlers, else_body, finally_body, span: _ } => {
            generate_try_except(
                state,
                body,
                handlers,
                else_body.as_deref(),
                finally_body.as_deref(),
            )?;
        }
        Stmt::With { items, body, is_async, span: _ } => {
            if *is_async {
                generate_async_with(state, items, body)?;
            } else {
                generate_sync_with(state, items, body)?;
            }
        }
        Stmt::Yield { value, span: _ } => {
            // For now, just evaluate the value
            if let Some(val) = value {
                let _ = crate::codegen::expressions::generate_expr(state, val)?;
            }
            // TODO: Implement generator yield
        }
        _ => {}
    }
    Ok(())
}
