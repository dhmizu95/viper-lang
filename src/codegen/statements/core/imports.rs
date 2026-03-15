//! Import and context management statement code generation.
//!
//! This module contains functions for generating LLVM IR for:
//! - import statements
//! - from import statements
//! - with statements (sync and async)

use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarStorage, VarType};
use inkwell::values::{BasicMetadataValueEnum, BasicValueEnum};

/// Counter for with blocks
static WITH_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Generate code for import statement: import module [as alias]
pub(crate) fn generate_import<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    module_name: &str,
    alias: Option<&str>,
) -> crate::codegen::Result<()> {
    // For now, imports are handled at the semantic level
    // The module is loaded and its symbols are available
    // We just need to ensure the module is in the registry

    let import_name = alias.unwrap_or(module_name);

    // Create a module object to represent the imported module
    // This allows accessing module.func() syntax
    // For now, we create a simple marker that the module exists

    // Create a global string for the module name
    let module_name_str = state.ir_builder.string_const(state.module, module_name);

    // Create a global to hold the module reference (accessible from all functions)
    let i8_ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
    let module_global =
        state.module.add_global(i8_ptr_type, None, &format!("__module_{}", import_name));
    module_global.set_initializer(&module_name_str);
    module_global.set_constant(false);
    module_global.set_unnamed_addr(false);

    // Add to global constants so it's accessible from all functions
    state.global_constants.insert(import_name.to_string(), module_global);

    Ok(())
}

/// Generate code for from import statement: from module import name1, name2 [as alias]
pub(crate) fn generate_from_import<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    module_name: &str,
    names: &[(String, Option<String>)],
) -> crate::codegen::Result<()> {
    // For each imported name, create a reference to the module's symbol
    for (name, alias) in names {
        let import_name = alias.as_deref().unwrap_or(name);

        // Create a global placeholder for the imported symbol
        // The actual symbol will be resolved at runtime through the module system
        let i8_ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
        let symbol_marker = state.module.add_global(
            i8_ptr_type,
            None,
            &format!("__from_import_{}_{}", module_name, import_name),
        );
        symbol_marker.set_initializer(&i8_ptr_type.const_null());
    }

    Ok(())
}

/// Generate code for a sync with statement
/// with expr as var:
///     body
pub(crate) fn generate_sync_with<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    items: &[crate::ast::WithItem],
    body: &[crate::ast::Stmt],
) -> crate::codegen::Result<()> {
    let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let with_num = WITH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // Create blocks for control flow
    let enter_block =
        state.context.append_basic_block(func_ctx, &format!("with_enter{}", with_num));
    let body_block = state.context.append_basic_block(func_ctx, &format!("with_body{}", with_num));
    let exit_block = state.context.append_basic_block(func_ctx, &format!("with_exit{}", with_num));
    let continue_block =
        state.context.append_basic_block(func_ctx, &format!("with_continue{}", with_num));

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

            state.variables.insert(
                var_name.clone(),
                VarInfo {
                    storage: VarStorage::Stack(var_alloca),
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
            state.function_param_names,
            state.function_param_defaults,
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
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
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
) -> crate::codegen::Result<()> {
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
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    use crate::ast::Type;
    use crate::codegen::oop::with_class_registry;

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
    let func_val = state
        .functions
        .get(&mangled_name)
        .copied()
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

    // Return appropriate default value based on return type
    if let Some(call_result) = result {
        Ok(call_result.into())
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
pub(crate) fn generate_async_with<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    items: &[crate::ast::WithItem],
    body: &[crate::ast::Stmt],
) -> crate::codegen::Result<()> {
    let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let with_num = WITH_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    // Create blocks for each phase
    let enter_block =
        state.context.append_basic_block(func_ctx, &format!("async_with_enter{}", with_num));
    let body_block =
        state.context.append_basic_block(func_ctx, &format!("async_with_body{}", with_num));
    let exit_block =
        state.context.append_basic_block(func_ctx, &format!("async_with_exit{}", with_num));
    let continue_block =
        state.context.append_basic_block(func_ctx, &format!("async_with_continue{}", with_num));

    // Branch to enter block
    state.ir_builder.build_branch(state.builder, enter_block);

    // Enter block: call __aenter__ for each item
    state.builder.position_at_end(enter_block);

    // Store context managers for exit phase
    let mut context_managers: Vec<(inkwell::values::BasicValueEnum<'ctx>, Option<String>)> =
        Vec::new();

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
                    storage: VarStorage::Stack(var_alloca),
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
            state.function_param_names,
            state.function_param_defaults,
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
) -> crate::codegen::Result<inkwell::values::BasicValueEnum<'ctx>> {
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
) -> crate::codegen::Result<()> {
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
