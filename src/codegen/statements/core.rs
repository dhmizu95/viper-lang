use crate::ast::{Expr, Stmt};
use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue};
use std::collections::{HashMap, HashSet};

use super::*;
use crate::codegen::builder::IRBuilder;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{LoopContext, VarInfo, VarType};
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
    let mut var_types = HashMap::new();
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
        &mut var_types,
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
    let mut var_types = HashMap::new();
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
        &mut var_types,
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
    _cause: Option<&Expr>,
) -> Result<(), String> {
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
                    state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
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
                            state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
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
                        state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
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
                    state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
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
                    state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
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
                    state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
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
                    state.context.i8_type().ptr_type(inkwell::AddressSpace::default()),
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
    // For each with item:
    // 1. Evaluate context expression
    // 2. Call __enter__ and bind to variable if present
    // 3. Execute body
    // 4. Call __exit__ (will be added in cleanup block)
    
    for item in items {
        // Evaluate context expression
        let context_val = crate::codegen::expressions::generate_expr(state, &item.context_expr)?;
        
        // Bind to variable if present
        if let Some(var_name) = &item.optional_vars {
            // Store in alloca for the variable
            let var_type = context_val.get_type();
            let var_alloca = state.builder.build_alloca(var_type, var_name).expect("alloca");
            state.builder.build_store(var_alloca, context_val).expect("store");

            state.variables.insert(
                var_name.clone(),
                VarInfo {
                    storage: crate::codegen::variables::VarStorage::Stack(var_alloca),
                    var_type: VarType::Pointer, // Context managers are heap-allocated objects
                    class_name: None,
                },
            );
        }
    }
    
    // Generate body
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
            stmt,
        )?;
    }
    
    // TODO: Generate __exit__ calls in cleanup block
    Ok(())
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
    
    // Enter block: call vp_async_context_enter for each item
    state.builder.position_at_end(enter_block);
    
    // Get the vp_async_context_enter function
    let enter_func = state.module.get_function("vp_async_context_enter")
        .ok_or_else(|| "vp_async_context_enter not declared".to_string())?;
    
    // Process each with item
    for (i, item) in items.iter().enumerate() {
        // Evaluate context expression
        let context_val = crate::codegen::expressions::generate_expr(state, &item.context_expr)?;
        
        // Call vp_async_context_enter(context)
        let enter_result = state.ir_builder.build_call(
            state.builder,
            enter_func,
            &[context_val.into()],
            &format!("async_with_enter_result{}", i),
        ).ok_or_else(|| "vp_async_context_enter call failed".to_string())?;
        
        // Bind to variable if present
        if let Some(var_name) = &item.optional_vars {
            let var_type = enter_result.get_type();
            let var_alloca = state.builder.build_alloca(var_type, var_name).expect("alloca");
            state.builder.build_store(var_alloca, enter_result).expect("store");

            state.variables.insert(
                var_name.clone(),
                VarInfo {
                    storage: crate::codegen::variables::VarStorage::Stack(var_alloca),
                    var_type: VarType::Pointer,
                    class_name: None,
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
            stmt,
        )?;
    }
    
    state.ir_builder.build_branch(state.builder, exit_block);
    
    // Exit block: call vp_async_context_exit for each item (in reverse order)
    state.builder.position_at_end(exit_block);
    
    // Get the vp_async_context_exit function
    let exit_func = state.module.get_function("vp_async_context_exit")
        .ok_or_else(|| "vp_async_context_exit not declared".to_string())?;
    
    // Call exit for each item in reverse order
    for (i, _item) in items.iter().rev().enumerate() {
        // Re-evaluate or reload context (simplified: just use 0 for exception info)
        let exc_type = state.ir_builder.i64_const(0);
        let exc_val = state.ir_builder.i64_const(0);
        let exc_tb = state.ir_builder.i64_const(0);
        
        // For simplicity, we're not passing the actual context here
        // A full implementation would need to save contexts from enter phase
        let _exit_result = state.ir_builder.build_call(
            state.builder,
            exit_func,
            &[state.ir_builder.i64_const(0).into(), exc_type.into(), exc_val.into(), exc_tb.into()],
            &format!("async_with_exit_result{}", i),
        );
    }
    
    state.ir_builder.build_branch(state.builder, continue_block);
    
    // Continue block: merge point
    state.builder.position_at_end(continue_block);
    
    Ok(())
}

// Counter for async with blocks
static WITH_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

