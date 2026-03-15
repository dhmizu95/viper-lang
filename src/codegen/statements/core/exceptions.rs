//! Exception handling statement code generation.
//!
//! This module contains functions for generating LLVM IR for:
//! - raise statements
//! - try/except/finally statements

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;

/// Generate code for raise statement
pub(crate) fn generate_raise<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    exception: Option<&Expr>,
    cause: Option<&Expr>,
) -> crate::codegen::Result<()> {
    // Check if we have a cause (raise X from Y)
    if cause.is_some() {
        return generate_raise_with_cause(state, exception, cause);
    }

    // Get the raise exception function
    let raise_func = state
        .module
        .get_function("viper_raise_exception")
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
                    &format!("exc_type_{}", name),
                );
                type_global.set_initializer(&exc_type);
                let type_ptr = state
                    .builder
                    .build_pointer_cast(
                        type_global.as_pointer_value(),
                        state.context.ptr_type(inkwell::AddressSpace::default()),
                        "exc_type_ptr",
                    )
                    .map_err(|e| format!("Failed to cast exception type: {:?}", e))?;

                // Get message from first argument if present
                let msg_ptr = if let Some(first_arg) = args.first() {
                    let msg_val = crate::codegen::expressions::generate_expr(state, first_arg)?;
                    // Convert to string pointer if it's a string
                    if msg_val.is_pointer_value() {
                        msg_val.into_pointer_value()
                    } else {
                        // Use empty string for non-string messages
                        let empty = state.context.const_string(b"", true);
                        let empty_global =
                            state.module.add_global(empty.get_type(), None, "empty_str");
                        empty_global.set_initializer(&empty);
                        state
                            .builder
                            .build_pointer_cast(
                                empty_global.as_pointer_value(),
                                state.context.ptr_type(inkwell::AddressSpace::default()),
                                "empty_msg",
                            )
                            .map_err(|e| format!("Failed to cast empty string: {:?}", e))?
                    }
                } else {
                    // No message argument
                    let empty = state.context.const_string(b"", true);
                    let empty_global = state.module.add_global(empty.get_type(), None, "empty_str");
                    empty_global.set_initializer(&empty);
                    state
                        .builder
                        .build_pointer_cast(
                            empty_global.as_pointer_value(),
                            state.context.ptr_type(inkwell::AddressSpace::default()),
                            "empty_msg",
                        )
                        .map_err(|e| format!("Failed to cast empty string: {:?}", e))?
                };

                (type_ptr, msg_ptr)
            }
            Expr::Ident(name, _) => {
                // Exception without call: ValueError
                let exc_type = state.context.const_string(name.as_bytes(), true);
                let type_global = state.module.add_global(
                    exc_type.get_type(),
                    None,
                    &format!("exc_type_{}", name),
                );
                type_global.set_initializer(&exc_type);
                let type_ptr = state
                    .builder
                    .build_pointer_cast(
                        type_global.as_pointer_value(),
                        state.context.ptr_type(inkwell::AddressSpace::default()),
                        "exc_type_ptr",
                    )
                    .map_err(|e| format!("Failed to cast exception type: {:?}", e))?;

                // Empty message
                let empty = state.context.const_string(b"", true);
                let empty_global = state.module.add_global(empty.get_type(), None, "empty_str");
                empty_global.set_initializer(&empty);
                let msg_ptr = state
                    .builder
                    .build_pointer_cast(
                        empty_global.as_pointer_value(),
                        state.context.ptr_type(inkwell::AddressSpace::default()),
                        "empty_msg",
                    )
                    .map_err(|e| format!("Failed to cast empty string: {:?}", e))?;

                (type_ptr, msg_ptr)
            }
            _ => {
                // Unknown expression type, use generic Exception
                let exc_type = state.context.const_string(b"Exception", true);
                let type_global =
                    state.module.add_global(exc_type.get_type(), None, "exc_type_generic");
                type_global.set_initializer(&exc_type);
                let type_ptr = state
                    .builder
                    .build_pointer_cast(
                        type_global.as_pointer_value(),
                        state.context.ptr_type(inkwell::AddressSpace::default()),
                        "exc_type_ptr",
                    )
                    .map_err(|e| format!("Failed to cast exception type: {:?}", e))?;

                let empty = state.context.const_string(b"", true);
                let empty_global = state.module.add_global(empty.get_type(), None, "empty_str");
                empty_global.set_initializer(&empty);
                let msg_ptr = state
                    .builder
                    .build_pointer_cast(
                        empty_global.as_pointer_value(),
                        state.context.ptr_type(inkwell::AddressSpace::default()),
                        "empty_msg",
                    )
                    .map_err(|e| format!("Failed to cast empty string: {:?}", e))?;

                (type_ptr, msg_ptr)
            }
        }
    } else {
        // Re-raise current exception
        let reraise_func = state
            .module
            .get_function("viper_reraise_exception")
            .ok_or("viper_reraise_exception function not found")?;
        state
            .builder
            .build_call(reraise_func, &[], "reraise")
            .map_err(|e| format!("Failed to build reraise call: {:?}", e))?;
        return Ok(());
    };

    // Build the raise call
    state
        .builder
        .build_call(raise_func, &[type_ptr.into(), msg_ptr.into()], "raise")
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
) -> crate::codegen::Result<()> {
    // Get the raise with cause function
    let raise_cause_func = state
        .module
        .get_function("viper_raise_with_cause")
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
    let exc_type_ptr = state
        .builder
        .build_pointer_cast(exc_type_global.as_pointer_value(), ptr_type, "exc_type_ptr")
        .map_err(|e| format!("Failed to cast exception type: {:?}", e))?;
    let exc_msg_ptr = state
        .builder
        .build_pointer_cast(exc_msg_global.as_pointer_value(), ptr_type, "exc_msg_ptr")
        .map_err(|e| format!("Failed to cast exception message: {:?}", e))?;
    let cause_type_ptr = state
        .builder
        .build_pointer_cast(cause_type_global.as_pointer_value(), ptr_type, "cause_type_ptr")
        .map_err(|e| format!("Failed to cast cause type: {:?}", e))?;
    let cause_msg_ptr = state
        .builder
        .build_pointer_cast(cause_msg_global.as_pointer_value(), ptr_type, "cause_msg_ptr")
        .map_err(|e| format!("Failed to cast cause message: {:?}", e))?;

    // Build the raise with cause call
    let i64_type = state.context.i64_type();
    state
        .builder
        .build_call(
            raise_cause_func,
            &[
                exc_type_ptr.into(),
                exc_msg_ptr.into(),
                i64_type.const_int(0, false).into(), // code
                cause_type_ptr.into(),
                cause_msg_ptr.into(),
            ],
            "raise_with_cause",
        )
        .map_err(|e| format!("Failed to build raise with cause call: {:?}", e))?;

    // Note: raise never returns
    Ok(())
}

/// Helper to extract exception type and message from an expression
fn extract_exception_info<'ctx>(
    _state: &mut CodeGenState<'_, 'ctx>,
    expr: Option<&Expr>,
) -> crate::codegen::Result<(String, String)> {
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
                Expr::Ident(name, _) => Ok((name.clone(), String::new())),
                _ => Ok(("Exception".to_string(), String::new())),
            }
        }
        None => Ok(("Exception".to_string(), String::new())),
    }
}

/// Generate code for try-except statement
/// NOTE: This is a simplified implementation that just generates the try body
/// Full exception handling with LLVM exception handling is a work in progress
pub(crate) fn generate_try_except<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    body: &[crate::ast::Stmt],
    _handlers: &[crate::ast::ExceptHandler],
    else_body: Option<&[crate::ast::Stmt]>,
    finally_body: Option<&[crate::ast::Stmt]>,
) -> crate::codegen::Result<()> {
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
            state.function_param_names,
            state.function_param_defaults,
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
                state.function_param_names,
                state.function_param_defaults,
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
                state.function_param_names,
                state.function_param_defaults,
                stmt,
            )?;
        }
    }

    Ok(())
}
