use crate::ast::{Expr, Stmt, Type};
use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue, BasicValueEnum, BasicMetadataValueEnum};
use std::collections::{HashMap, HashSet};

use super::*;
use crate::codegen::builder::IRBuilder;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{LoopContext, VarInfo, VarType};
use crate::semantic::escape_analysis::EscapeAnalyzer;
use crate::semantic::closure_analysis::ClosureAnalyzer;

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
        var_types,
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
        var_types,
        escape_analyzer,
        current_function,
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
) -> Result<(), String> {
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
    );
    state.current_class = current_class.map(|s| s.to_string());

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
            return crate::codegen::control_flow::generate_for(state, target, iter, body, else_body, false);
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
                    end_bb  // Last case's else goes to end
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
                let keys_to_remove: Vec<String> = state.variables.keys()
                    .skip(saved_var_count)
                    .cloned()
                    .collect();
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
            // For now, just evaluate the condition and message (no actual assertion)
            let _cond_val = crate::codegen::expressions::generate_expr(state, condition)?;
            if let Some(msg) = message {
                let _ = crate::codegen::expressions::generate_expr(state, msg);
            }
            // TODO: Implement actual assertion with runtime panic
        }
        Stmt::Delete { targets, span: _ } => {
            // For now, just evaluate the targets (no actual deletion)
            for target in targets {
                let _ = crate::codegen::expressions::generate_expr(state, target)?;
            }
            // TODO: Implement actual deletion (decrement ref counts, etc.)
        }
        Stmt::Raise { exception, cause, span: _ } => {
            generate_raise(state, exception.as_deref(), cause.as_deref())?;
        }
        Stmt::Try { body, handlers, else_body, finally_body, span: _ } => {
            generate_try_except(state, body, handlers, else_body.as_deref(), finally_body.as_deref())?;
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

/// Generate code for raise statement
fn generate_raise<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    exception: Option<&Expr>,
    cause: Option<&Expr>,
) -> Result<(), String> {
    // Check if we have a cause (raise X from Y)
    if cause.is_some() {
        return generate_raise_with_cause(state, exception, cause);
    }

    // Get the raise exception function
    let raise_func = state.module.get_function("viper_raise_exception")
        .ok_or("viper_raise_exception function not found")?;

    // Determine exception type and message
    let (type_ptr, msg_ptr) = if let Some(exc) = exception {
        match exc {
            Expr::Call { func, args, .. } if matches!(func.as_ref(), Expr::Ident(..)) => {
                let name = if let Expr::Ident(name, _) = func.as_ref() {
                    name.clone()
                } else {
                    "Exception".to_string()
                };
                // Exception with constructor call: ValueError("message")
                let exc_type = state.context.const_string(name.as_bytes(), true);
                let type_global = state.module.add_global(
                    exc_type.get_type(),
                    None,
                    &format!("exc_type_{}", name)
                );
                type_global.set_initializer(&exc_type);
                let type_ptr = state.builder.build_pointer_cast(
                    type_global.as_pointer_value(),
                    state.context.ptr_type(inkwell::AddressSpace::default()),
                    "exc_type_ptr"
                ).map_err(|e| format!("Failed to cast exception type: {:?}", e))?;

                // Get message from first argument if present
                let msg_ptr = if let Some(first_arg) = args.first() {
                    let msg_val = crate::codegen::expressions::generate_expr(state, first_arg)?;
                    // Convert to string pointer if it's a string
                    if msg_val.is_pointer_value() {
                        msg_val.into_pointer_value()
                    } else {
                        // Use empty string for non-string messages
                        let empty = state.context.const_string(b"", true);
                        let empty_global = state.module.add_global(
                            empty.get_type(),
                            None,
                            "empty_str"
                        );
                        empty_global.set_initializer(&empty);
                        state.builder.build_pointer_cast(
                            empty_global.as_pointer_value(),
                            state.context.ptr_type(inkwell::AddressSpace::default()),
                            "empty_msg"
                        ).map_err(|e| format!("Failed to cast empty string: {:?}", e))?
                    }
                } else {
                    // No message argument
                    let empty = state.context.const_string(b"", true);
                    let empty_global = state.module.add_global(
                        empty.get_type(),
                        None,
                        "empty_str"
                    );
                    empty_global.set_initializer(&empty);
                    state.builder.build_pointer_cast(
                        empty_global.as_pointer_value(),
                        state.context.ptr_type(inkwell::AddressSpace::default()),
                        "empty_msg"
                    ).map_err(|e| format!("Failed to cast empty string: {:?}", e))?
                };

                (type_ptr, msg_ptr)
            }
            Expr::Ident(name, _) => {
                // Exception without call: ValueError
                let exc_type = state.context.const_string(name.as_bytes(), true);
                let type_global = state.module.add_global(
                    exc_type.get_type(),
                    None,
                    &format!("exc_type_{}", name)
                );
                type_global.set_initializer(&exc_type);
                let type_ptr = state.builder.build_pointer_cast(
                    type_global.as_pointer_value(),
                    state.context.ptr_type(inkwell::AddressSpace::default()),
                    "exc_type_ptr"
                ).map_err(|e| format!("Failed to cast exception type: {:?}", e))?;

                // Empty message
                let empty = state.context.const_string(b"", true);
                let empty_global = state.module.add_global(
                    empty.get_type(),
                    None,
                    "empty_str"
                );
                empty_global.set_initializer(&empty);
                let msg_ptr = state.builder.build_pointer_cast(
                    empty_global.as_pointer_value(),
                    state.context.ptr_type(inkwell::AddressSpace::default()),
                    "empty_msg"
                ).map_err(|e| format!("Failed to cast empty string: {:?}", e))?;

                (type_ptr, msg_ptr)
            }
            _ => {
                // Unknown expression type, use generic Exception
                let exc_type = state.context.const_string(b"Exception", true);
                let type_global = state.module.add_global(
                    exc_type.get_type(),
                    None,
                    "exc_type_generic"
                );
                type_global.set_initializer(&exc_type);
                let type_ptr = state.builder.build_pointer_cast(
                    type_global.as_pointer_value(),
                    state.context.ptr_type(inkwell::AddressSpace::default()),
                    "exc_type_ptr"
                ).map_err(|e| format!("Failed to cast exception type: {:?}", e))?;

                let empty = state.context.const_string(b"", true);
                let empty_global = state.module.add_global(
                    empty.get_type(),
                    None,
                    "empty_str"
                );
                empty_global.set_initializer(&empty);
                let msg_ptr = state.builder.build_pointer_cast(
                    empty_global.as_pointer_value(),
                    state.context.ptr_type(inkwell::AddressSpace::default()),
                    "empty_msg"
                ).map_err(|e| format!("Failed to cast empty string: {:?}", e))?;

                (type_ptr, msg_ptr)
            }
        }
    } else {
        // Re-raise current exception
        let reraise_func = state.module.get_function("viper_reraise_exception")
            .ok_or("viper_reraise_exception function not found")?;
        state.builder.build_call(reraise_func, &[], "reraise")
            .map_err(|e| format!("Failed to build reraise call: {:?}", e))?;
        return Ok(());
    };

    // Build the raise call
    state.builder.build_call(raise_func, &[type_ptr.into(), msg_ptr.into()], "raise")
        .map_err(|e| format!("Failed to build raise call: {:?}", e))?;

    // Note: raise never returns, but we need to satisfy LLVM's control flow
    // The runtime function exits the process
    Ok(())
}

/// Generate code for raise statement with cause (raise X from Y)
fn generate_raise_with_cause<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    exception: Option<&Expr>,
    cause: Option<&Expr>,
) -> Result<(), String> {
    // Get the raise with cause function
    let raise_cause_func = state.module.get_function("viper_raise_with_cause")
        .ok_or("viper_raise_with_cause function not found")?;

    // Extract exception type and message
    let (exc_type_str, exc_msg_str) = extract_exception_info(state, exception)?;

    // Extract cause type and message
    let (cause_type_str, cause_msg_str) = extract_exception_info(state, cause)?;

    // Create string constants
    let exc_type = state.context.const_string(exc_type_str.as_bytes(), true);
    let exc_msg = state.context.const_string(exc_msg_str.as_bytes(), true);
    let cause_type = state.context.const_string(cause_type_str.as_bytes(), true);
    let cause_msg = state.context.const_string(cause_msg_str.as_bytes(), true);

    // Create globals for strings
    let exc_type_global = state.module.add_global(exc_type.get_type(), None, "exc_type");
    exc_type_global.set_initializer(&exc_type);
    let exc_msg_global = state.module.add_global(exc_msg.get_type(), None, "exc_msg");
    exc_msg_global.set_initializer(&exc_msg);
    let cause_type_global = state.module.add_global(cause_type.get_type(), None, "cause_type");
    cause_type_global.set_initializer(&cause_type);
    let cause_msg_global = state.module.add_global(cause_msg.get_type(), None, "cause_msg");
    cause_msg_global.set_initializer(&cause_msg);

    // Cast to pointers
    let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
    let exc_type_ptr = state.builder.build_pointer_cast(
        exc_type_global.as_pointer_value(),
        ptr_type,
        "exc_type_ptr"
    ).map_err(|e| format!("Failed to cast exception type: {:?}", e))?;
    let exc_msg_ptr = state.builder.build_pointer_cast(
        exc_msg_global.as_pointer_value(),
        ptr_type,
        "exc_msg_ptr"
    ).map_err(|e| format!("Failed to cast exception message: {:?}", e))?;
    let cause_type_ptr = state.builder.build_pointer_cast(
        cause_type_global.as_pointer_value(),
        ptr_type,
        "cause_type_ptr"
    ).map_err(|e| format!("Failed to cast cause type: {:?}", e))?;
    let cause_msg_ptr = state.builder.build_pointer_cast(
        cause_msg_global.as_pointer_value(),
        ptr_type,
        "cause_msg_ptr"
    ).map_err(|e| format!("Failed to cast cause message: {:?}", e))?;

    // Build the raise with cause call
    let i64_type = state.context.i64_type();
    state.builder.build_call(
        raise_cause_func,
        &[
            exc_type_ptr.into(),
            exc_msg_ptr.into(),
            i64_type.const_int(0, false).into(), // code
            cause_type_ptr.into(),
            cause_msg_ptr.into(),
        ],
        "raise_with_cause"
    ).map_err(|e| format!("Failed to build raise with cause call: {:?}", e))?;

    // Note: raise never returns
    Ok(())
}

/// Helper to extract exception type and message from an expression
fn extract_exception_info<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    expr: Option<&Expr>,
) -> Result<(String, String), String> {
    match expr {
        Some(exc) => {
            match exc {
                Expr::Call { func, args, .. } if matches!(func.as_ref(), Expr::Ident(..)) => {
                    let name = if let Expr::Ident(name, _) = func.as_ref() {
                        name.clone()
                    } else {
                        "Exception".to_string()
                    };
                    // Get message from first argument if present
                    let msg = if let Some(first_arg) = args.first() {
                        if let Expr::Str(s, _) = first_arg {
                            s.clone()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    Ok((name, msg))
                }
                Expr::Ident(name, _) => {
                    Ok((name.clone(), String::new()))
                }
                _ => Ok(("Exception".to_string(), String::new())),
            }
        }
        None => Ok(("Exception".to_string(), String::new())),
    }
}

/// Generate code for try-except statement
/// NOTE: This is a simplified implementation that just generates the try body
/// Full exception handling with LLVM exception handling is a work in progress
fn generate_try_except<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    body: &[Stmt],
    _handlers: &[crate::ast::ExceptHandler],
    else_body: Option<&[Stmt]>,
    finally_body: Option<&[Stmt]>,
) -> Result<(), String> {
    // For now, generate the try body only
    // Full exception handling requires more complex LLVM unwinding support
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
            state.dict_vars,
            state.bool_list_vars,
            state.bigint_vars,
            state.var_types,
            stmt,
        )?;
    }
    
    // Generate else body if present
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
                state.dict_vars,
                state.bool_list_vars,
                state.bigint_vars,
                state.var_types,
                stmt,
            )?;
        }
    }
    
    // Generate finally body if present
    if let Some(finally_stmts) = finally_body {
        for stmt in finally_stmts {
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
    }

    Ok(())
}

/// Generate code for a sync with statement
/// with expr as var:
///     body
fn generate_sync_with<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    items: &[crate::ast::WithItem],
    body: &[Stmt],
) -> Result<(), String> {
    use inkwell::values::BasicValueEnum;

    let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let with_num = WITH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // Create blocks for control flow
    let enter_block = state.context.append_basic_block(func_ctx, &format!("with_enter{}", with_num));
    let body_block = state.context.append_basic_block(func_ctx, &format!("with_body{}", with_num));
    let exit_block = state.context.append_basic_block(func_ctx, &format!("with_exit{}", with_num));
    let continue_block = state.context.append_basic_block(func_ctx, &format!("with_continue{}", with_num));

    // Branch to enter block
    state.ir_builder.build_branch(state.builder, enter_block);

    // Enter block: evaluate context expressions and call __enter__
    state.builder.position_at_end(enter_block);

    // Store context manager objects and enter results for each item
    let mut context_managers: Vec<(BasicValueEnum<'ctx>, Option<String>)> = Vec::new();

    for (_i, item) in items.iter().enumerate() {
        // Evaluate context expression
        let context_val = crate::codegen::expressions::generate_expr(state, &item.context_expr)?;

        // Call __enter__ method on the context manager
        let enter_result = call_context_enter(state, &context_val)?;

        // Store context manager and enter result
        context_managers.push((context_val, item.optional_vars.clone()));

        // Bind __enter__ result to variable if present
        if let Some(var_name) = &item.optional_vars {
            let var_type = enter_result.get_type();
            let var_alloca = state.builder.build_alloca(var_type, var_name).expect("alloca");
            state.builder.build_store(var_alloca, enter_result).expect("store");

            let ast_type = if enter_result.is_int_value() {
                if enter_result.into_int_value().get_type().get_bit_width() == 1 {
                    state.var_types.insert(var_name.clone(), Type::Bool);
                    VarType::Bool
                } else {
                    state.var_types.insert(var_name.clone(), Type::Int);
                    VarType::Int
                }
            } else if enter_result.is_float_value() {
                state.var_types.insert(var_name.clone(), Type::F64);
                VarType::Float
            } else {
                state.var_types.insert(var_name.clone(), Type::Infer);
                VarType::Pointer
            };

            state.variables.insert(
                var_name.clone(),
                VarInfo {
                    storage: crate::codegen::variables::VarStorage::Stack(var_alloca),
                    var_type: ast_type,
                    class_name: None,
                    closure_value_ptr: None,
                },
            );
        }
    }

    state.ir_builder.build_branch(state.builder, body_block);

    // Body block: execute the with body
    state.builder.position_at_end(body_block);

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
            state.dict_vars,
            state.bool_list_vars,
            state.bigint_vars,
            state.var_types,
            stmt,
        )?;
    }

    // Branch to exit block (normal execution path)
    state.ir_builder.build_branch(state.builder, exit_block);

    // Exit block: call __exit__ for each context manager (in reverse order)
    state.builder.position_at_end(exit_block);

    // Call __exit__ with no exception (exc_type=None, exc_val=None, exc_tb=None)
    for (_i, (context_val, _)) in context_managers.iter().rev().enumerate() {
        call_context_exit(state, context_val, false)?;
    }

    state.ir_builder.build_branch(state.builder, continue_block);

    // Continue block: merge point
    state.builder.position_at_end(continue_block);

    Ok(())
}

/// Call __enter__ method on a context manager object
fn call_context_enter<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    context_val: &BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    // Context manager must be a pointer (object)
    if !context_val.is_pointer_value() {
        // For non-object types (like literals), just return the value
        return Ok(*context_val);
    }

    let context_ptr = context_val.into_pointer_value();

    // Try to infer the class type from the context value
    // For now, we'll call __enter__ directly using the method lookup
    let enter_result = call_method_on_object(state, context_ptr, "__enter__", &[])?;

    Ok(enter_result)
}

/// Call __exit__ method on a context manager object
/// If has_exception is true, passes exception info; otherwise passes None values
fn call_context_exit<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    context_val: &BasicValueEnum<'ctx>,
    has_exception: bool,
) -> Result<(), String> {
    // Context manager must be a pointer (object)
    if !context_val.is_pointer_value() {
        // For non-object types, nothing to do
        return Ok(());
    }

    let context_ptr = context_val.into_pointer_value();

    // Build exception info arguments
    let i64_type = state.context.i64_type();
    let exc_type = if has_exception {
        i64_type.const_int(1, false).into()
    } else {
        i64_type.const_int(0, false).into()
    };
    let exc_val = i64_type.const_int(0, false).into();
    let exc_tb = i64_type.const_int(0, false).into();

    // Call __exit__(exc_type, exc_val, exc_tb)
    let args = [exc_type, exc_val, exc_tb];
    call_method_on_object(state, context_ptr, "__exit__", &args)?;

    Ok(())
}

/// Helper function to call a method on an object pointer
fn call_method_on_object<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj_ptr: inkwell::values::PointerValue<'ctx>,
    method_name: &str,
    args: &[inkwell::values::BasicMetadataValueEnum<'ctx>],
) -> Result<BasicValueEnum<'ctx>, String> {
    use crate::codegen::oop::with_class_registry;
    use crate::ast::Type;

    // Try to find the method in the class registry
    let mut method_info: Option<(String, Type)> = None;

    with_class_registry(|reg| {
        if let Some((_class, method)) = reg.find_method(method_name) {
            method_info = Some((method.mangled_name.clone(), method.return_type.clone()));
        }
    });

    let (mangled_name, return_type) = method_info
        .ok_or_else(|| format!("Method '{}' not found on context manager", method_name))?;

    // Get the function
    let func_val = state.functions.get(&mangled_name).copied()
        .ok_or_else(|| format!("Function '{}' not found", mangled_name))?;

    // Build argument list: self + method args
    let mut arg_values: Vec<BasicMetadataValueEnum<'ctx>> = Vec::with_capacity(1 + args.len());
    arg_values.push(obj_ptr.into());
    arg_values.extend_from_slice(args);

    // Call the method
    let result = state.ir_builder.build_call(
        state.builder,
        func_val,
        &arg_values,
        &format!("context_{}_call", method_name.trim_matches('_')),
    );

    // Return the result directly if it exists, otherwise return a default
    if let Some(call_result) = result {
        Ok(call_result)
    } else {
        // Return appropriate default based on method return type
        match return_type {
            Type::Class(_) | Type::Instance(_) | Type::Str | Type::List(_) | Type::Dict(_, _) => {
                // Return null pointer for reference types
                Ok(state.context.ptr_type(inkwell::AddressSpace::default()).const_null().into())
            }
            Type::Bool | Type::I8 => Ok(state.context.i8_type().const_int(0, false).into()),
            Type::I16 => Ok(state.context.i16_type().const_int(0, false).into()),
            Type::I32 => Ok(state.context.i32_type().const_int(0, false).into()),
            Type::F32 => Ok(state.context.f32_type().const_float(0.0).into()),
            Type::F64 => Ok(state.context.f64_type().const_float(0.0).into()),
            _ => Ok(state.context.i64_type().const_int(0, false).into()),
        }
    }
}

/// Generate code for an async with statement
/// async with expr as var:
///     body
fn generate_async_with<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    items: &[crate::ast::WithItem],
    body: &[Stmt],
) -> Result<(), String> {
    let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let with_num = WITH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // Create blocks for each phase
    let enter_block = state.context.append_basic_block(func_ctx, &format!("async_with_enter{}", with_num));
    let body_block = state.context.append_basic_block(func_ctx, &format!("async_with_body{}", with_num));
    let exit_block = state.context.append_basic_block(func_ctx, &format!("async_with_exit{}", with_num));
    let continue_block = state.context.append_basic_block(func_ctx, &format!("async_with_continue{}", with_num));

    // Branch to enter block
    state.ir_builder.build_branch(state.builder, enter_block);

    // Enter block: call __aenter__ for each item
    state.builder.position_at_end(enter_block);

    // Store context managers for exit phase
    let mut context_managers: Vec<(inkwell::values::BasicValueEnum<'ctx>, Option<String>)> = Vec::new();

    // Process each with item
    for (_i, item) in items.iter().enumerate() {
        // Evaluate context expression
        let context_val = crate::codegen::expressions::generate_expr(state, &item.context_expr)?;

        // Call __aenter__ method on the context manager
        let aenter_result = call_async_context_enter(state, &context_val)?;

        // Store context manager
        context_managers.push((context_val, item.optional_vars.clone()));

        // Bind __aenter__ result to variable if present
        if let Some(var_name) = &item.optional_vars {
            let var_type = aenter_result.get_type();
            let var_alloca = state.builder.build_alloca(var_type, var_name).expect("alloca");
            state.builder.build_store(var_alloca, aenter_result).expect("store");

            state.variables.insert(
                var_name.clone(),
                VarInfo {
                    storage: crate::codegen::variables::VarStorage::Stack(var_alloca),
                    var_type: VarType::Pointer,
                    class_name: None,
                    closure_value_ptr: None,
                },
            );
        }
    }

    state.ir_builder.build_branch(state.builder, body_block);

    // Body block: execute the with body
    state.builder.position_at_end(body_block);

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
            state.dict_vars,
            state.bool_list_vars,
            state.bigint_vars,
            state.var_types,
            stmt,
        )?;
    }

    state.ir_builder.build_branch(state.builder, exit_block);

    // Exit block: call __aexit__ for each item (in reverse order)
    state.builder.position_at_end(exit_block);

    // Call __aexit__ with no exception for each context manager
    for (_i, (context_val, _)) in context_managers.iter().rev().enumerate() {
        call_async_context_exit(state, context_val, false)?;
    }

    state.ir_builder.build_branch(state.builder, continue_block);

    // Continue block: merge point
    state.builder.position_at_end(continue_block);

    Ok(())
}

/// Call __aenter__ method on an async context manager object
fn call_async_context_enter<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    context_val: &inkwell::values::BasicValueEnum<'ctx>,
) -> Result<inkwell::values::BasicValueEnum<'ctx>, String> {
    // Context manager must be a pointer (object)
    if !context_val.is_pointer_value() {
        // For non-object types, just return the value
        return Ok(*context_val);
    }

    let context_ptr = context_val.into_pointer_value();

    // Call __aenter__ method (no arguments)
    let aenter_result = call_method_on_object(state, context_ptr, "__aenter__", &[])?;

    Ok(aenter_result)
}

/// Call __aexit__ method on an async context manager object
fn call_async_context_exit<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    context_val: &inkwell::values::BasicValueEnum<'ctx>,
    has_exception: bool,
) -> Result<(), String> {
    // Context manager must be a pointer (object)
    if !context_val.is_pointer_value() {
        // For non-object types, nothing to do
        return Ok(());
    }

    let context_ptr = context_val.into_pointer_value();

    // Build exception info arguments
    let i64_type = state.context.i64_type();
    let exc_type = if has_exception {
        i64_type.const_int(1, false).into()
    } else {
        i64_type.const_int(0, false).into()
    };
    let exc_val = i64_type.const_int(0, false).into();
    let exc_tb = i64_type.const_int(0, false).into();

    // Call __aexit__(exc_type, exc_val, exc_tb)
    let args = [exc_type, exc_val, exc_tb];
    call_method_on_object(state, context_ptr, "__aexit__", &args)?;

    Ok(())
}

/// Generate code for import statement: import module [as alias]
fn generate_import<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    module_name: &str,
    alias: Option<&str>,
) -> Result<(), String> {
    // For now, imports are handled at the semantic level
    // The module is loaded and its symbols are available
    // We just need to ensure the module is in the registry
    
    let import_name = alias.unwrap_or(module_name);
    
    // Create a marker global to indicate the module is imported
    // This prevents "undefined variable" errors when using module.func()
    let i8_ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
    let module_marker = state.module.add_global(i8_ptr_type, None, &format!("__import_{}", import_name));
    module_marker.set_initializer(&i8_ptr_type.const_null());
    
    Ok(())
}

/// Generate code for from import statement: from module import name1, name2 [as alias]
fn generate_from_import<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    module_name: &str,
    names: &[(String, Option<String>)],
) -> Result<(), String> {
    // For each imported name, create a reference to the module's symbol
    for (name, alias) in names {
        let import_name = alias.as_deref().unwrap_or(name);
        
        // Create a global placeholder for the imported symbol
        // The actual symbol will be resolved at runtime through the module system
        let i8_ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
        let symbol_marker = state.module.add_global(
            i8_ptr_type, 
            None, 
            &format!("__from_import_{}_{}", module_name, import_name)
        );
        symbol_marker.set_initializer(&i8_ptr_type.const_null());
    }
    
    Ok(())
}

// Counter for async with blocks
static WITH_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
