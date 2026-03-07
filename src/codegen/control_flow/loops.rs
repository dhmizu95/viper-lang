use crate::ast::{Expr, Stmt};
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{LoopContext, VarInfo, VarType};
use std::sync::atomic::{AtomicUsize, Ordering};

static WHILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate a while loop with optional unrolling for hot loops
pub fn generate_while<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    body: &[Stmt],
    else_body: &Option<Vec<Stmt>>,
) -> Result<(), String> {
    // For now, use simple loop - LLVM's opt will handle unrolling
    generate_while_simple(state, condition, body, else_body)
}

fn generate_while_simple<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    body: &[Stmt],
    else_body: &Option<Vec<Stmt>>,
) -> Result<(), String> {
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let while_num = WHILE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let cond_block = state.context.append_basic_block(func, &format!("while_cond{}", while_num));
    let body_block = state.context.append_basic_block(func, &format!("while_body{}", while_num));
    let else_block = if else_body.is_some() {
        Some(state.context.append_basic_block(func, &format!("while_else{}", while_num)))
    } else {
        None
    };
    let exit_block = state.context.append_basic_block(func, &format!("while_exit{}", while_num));

    state.ir_builder.build_branch(state.builder, cond_block);

    state.builder.position_at_end(cond_block);
    let cond_expr = crate::codegen::expressions::generate_expr(state, condition)?;
    let cond_val = cond_expr.into_int_value();
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

    // If condition is true, go to body; if false, go to else (if exists) or exit
    let next_on_false = else_block.unwrap_or(exit_block);
    state.ir_builder.build_cond_branch(state.builder, cond_i1, body_block, next_on_false);

    state.builder.position_at_end(body_block);
    // Push loop context: break goes to exit, continue goes to condition
    // If there's an else block, we need to track that normal exit goes through else
    state.loop_stack.push(LoopContext::new(exit_block, cond_block));

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

    state.loop_stack.pop();
    // After body completes, loop back to condition
    if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
        state.ir_builder.build_branch(state.builder, cond_block);
    }
    
    // Generate else block if it exists
    if let Some(else_stmts) = else_body {
        state.builder.position_at_end(else_block.unwrap());
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
        // After else block, jump to exit
        state.ir_builder.build_branch(state.builder, exit_block);
    }
    
    state.builder.position_at_end(exit_block);
    Ok(())
}

/// Generate a for loop using the iterator protocol (__iter__ and __next__)
/// This supports custom iterable classes that implement the iterator protocol.
fn generate_for_with_iterator<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    iter: &Expr,
    body: &[Stmt],
    else_body: &Option<Vec<Stmt>>,
) -> Result<(), String> {
    let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let for_num = WHILE_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // Blocks for iterator protocol
    let iter_block = state.context.append_basic_block(func_ctx, &format!("for_iter{}", for_num));
    let next_block = state.context.append_basic_block(func_ctx, &format!("for_next{}", for_num));
    let check_block = state.context.append_basic_block(func_ctx, &format!("for_check{}", for_num));
    let body_block = state.context.append_basic_block(func_ctx, &format!("for_body{}", for_num));
    let exit_block = state.context.append_basic_block(func_ctx, &format!("for_exit{}", for_num));
    let else_block = else_body.as_ref().map(|_| {
        state.context.append_basic_block(func_ctx, &format!("for_else{}", for_num))
    });
    
    state.ir_builder.build_branch(state.builder, iter_block);
    
    // Iterator block: call __iter__() on the iterable
    state.builder.position_at_end(iter_block);
    let iter_val = crate::codegen::expressions::generate_expr(state, iter)?;
    
    // Call __iter__ method to get iterator
    // For now, we assume the iterator is the same object (common pattern)
    // Store iterator in alloca
    let iterator_alloca = state.builder.build_alloca(
        state.context.ptr_type(inkwell::AddressSpace::default()),
        "iterator",
    ).expect("alloca");
    state.builder.build_store(iterator_alloca, iter_val.into_pointer_value()).expect("store");
    
    state.ir_builder.build_branch(state.builder, next_block);
    
    // Next block: call __next__() and check for StopIteration
    state.builder.position_at_end(next_block);
    
    // Load iterator
    let iterator_ptr = state.builder.build_load(
        state.context.ptr_type(inkwell::AddressSpace::default()),
        iterator_alloca,
        "iterator",
    ).expect("load").into_pointer_value();
    
    // Call __next__ method on iterator
    // For now, use a simple approach: call vp_iterator_next which returns (value, done_flag)
    // Full implementation would call iterator.__next__() and catch StopIteration
    let iterator_next_func = state.module.get_function("vp_iterator_next");
    
    if let Some(next_func) = iterator_next_func {
        // Call vp_iterator_next(iterator_ptr) -> struct { value: i64, done: i1 }
        let result_val = state.ir_builder.build_call(
            state.builder,
            next_func,
            &[iterator_ptr.into()],
            "next_result",
        );
        
        // Extract value and done flag from result struct
        let result_ptr = result_val.expect("next result").into_pointer_value();
        
        // Load done flag (second field of struct)
        let done_ptr = unsafe {
            state.builder.build_in_bounds_gep(
                state.context.i64_type(),
                result_ptr,
                &[state.ir_builder.i64_const(1)],
                "done_ptr",
            )
        }.expect("gep done");
        
        let done_val = state.builder.build_load(
            state.context.bool_type(),
            done_ptr,
            "done",
        ).expect("load done").into_int_value();
        
        state.ir_builder.build_branch(state.builder, check_block);
        
        // Check block: if done, exit; otherwise go to body
        state.builder.position_at_end(check_block);
        state.ir_builder.build_cond_branch(
            state.builder,
            state.builder.build_not(done_val, "not_done").expect("not"),
            body_block,
            else_block.unwrap_or(exit_block),
        );
        
        // Body block
        state.builder.position_at_end(body_block);
        
        // Load value from result struct (first field)
        let value_ptr = unsafe {
            state.builder.build_in_bounds_gep(
                state.context.i64_type(),
                result_ptr,
                &[state.ir_builder.i64_const(0)],
                "value_ptr",
            )
        }.expect("gep value");
        
        let value = state.builder.build_load(
            state.context.i64_type(),
            value_ptr,
            "value",
        ).expect("load value");
        
        // Bind value to target variable
        if let Expr::Ident(target_name, _) = target {
            let target_alloca = state.builder.build_alloca(
                state.context.i64_type(),
                target_name,
            ).expect("alloca target");
            state.builder.build_store(target_alloca, value).expect("store target");
            
            state.variables.insert(
                target_name.clone(),
                VarInfo::new_stack(target_alloca, VarType::Int),
            );
        }
        
        // Push loop context
        state.loop_stack.push(LoopContext::new(
            else_block.unwrap_or(exit_block),  // break target
            next_block,  // continue target (get next item)
        ));
        
        // Generate body statements
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
        
        state.loop_stack.pop();
        
        // Jump back to get next item
        if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
            state.ir_builder.build_branch(state.builder, next_block);
        }
        
        // Else block (if exists)
        if let Some(else_blk) = else_block {
            state.builder.position_at_end(else_blk);
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
            state.ir_builder.build_branch(state.builder, exit_block);
        }
        
        // Exit block
        state.builder.position_at_end(exit_block);
        
        Ok(())
    } else {
        // Fall back to simple iteration if vp_iterator_next not available
        Err("Iterator protocol requires vp_iterator_next runtime function".to_string())
    }
}

/// Generate a for loop (supports range(), list iteration, and iterator protocol)
pub fn generate_for<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    iter: &Expr,
    body: &[Stmt],
    else_body: &Option<Vec<Stmt>>,
    is_async: bool,
) -> Result<(), String> {
    if is_async {
        return generate_async_for(state, target, iter, body);
    }

    // Check for iterator protocol: if iterable has __iter__ method, use iterator
    if let Expr::Ident(iter_name, _) = iter {
        if let Some(var_type) = state.var_types.get(iter_name) {
            if matches!(var_type, crate::ast::Type::Instance(_)) {
                return generate_for_with_iterator(state, target, iter, body, else_body);
            }
        }
    }

    // Handle range() specially
    if let Expr::Call { func, args, .. } = iter {
        if let Expr::Ident(name, _) = func.as_ref() {
            if name == "range" {
                let (start_val, end_val, step_val) = match args.len() {
                    0 => return Err("range expected at least 1 argument, got 0".to_string()),
                    1 => (
                        state.ir_builder.i64_const(0),
                        crate::codegen::expressions::generate_expr(state, &args[0])?
                            .into_int_value(),
                        state.ir_builder.i64_const(1),
                    ),
                    2 => (
                        crate::codegen::expressions::generate_expr(state, &args[0])?
                            .into_int_value(),
                        crate::codegen::expressions::generate_expr(state, &args[1])?
                            .into_int_value(),
                        state.ir_builder.i64_const(1),
                    ),
                    _ => (
                        crate::codegen::expressions::generate_expr(state, &args[0])?
                            .into_int_value(),
                        crate::codegen::expressions::generate_expr(state, &args[1])?
                            .into_int_value(),
                        crate::codegen::expressions::generate_expr(state, &args[2])?
                            .into_int_value(),
                    ),
                };

                let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
                let for_num = WHILE_COUNTER.fetch_add(1, Ordering::SeqCst);
                let init_block =
                    state.context.append_basic_block(func_ctx, &format!("for_init{}", for_num));
                let cond_block =
                    state.context.append_basic_block(func_ctx, &format!("for_cond{}", for_num));
                let body_block =
                    state.context.append_basic_block(func_ctx, &format!("for_body{}", for_num));
                let step_block =
                    state.context.append_basic_block(func_ctx, &format!("for_step{}", for_num));
                let else_block = if else_body.is_some() {
                    Some(state.context.append_basic_block(func_ctx, &format!("for_else{}", for_num)))
                } else {
                    None
                };
                let exit_block =
                    state.context.append_basic_block(func_ctx, &format!("for_exit{}", for_num));

                state.ir_builder.build_branch(state.builder, init_block);
                state.builder.position_at_end(init_block);
                let counter = state
                    .builder
                    .build_alloca(state.context.i64_type(), "for_counter")
                    .expect("alloca");
                state.builder.build_store(counter, start_val).expect("store");
                state.ir_builder.build_branch(state.builder, cond_block);

                state.builder.position_at_end(cond_block);
                let counter_val = state
                    .builder
                    .build_load(state.context.i64_type(), counter, "counter_val")
                    .expect("load")
                    .into_int_value();
                let cond =
                    state.ir_builder.build_icmp_lt(state.builder, counter_val, end_val, "for_cond");
                // If condition is true, go to body; if false, go to else (if exists) or exit
                let next_on_false = else_block.unwrap_or(exit_block);
                state.ir_builder.build_cond_branch(state.builder, cond, body_block, next_on_false);

                state.builder.position_at_end(body_block);

                // Push loop context for break/continue support
                // continue should jump to step block to increment counter
                // break jumps to exit (skipping else)
                state.loop_stack.push(LoopContext::new(exit_block, step_block));

                let old_var = if let Expr::Ident(target_name, _) = target {
                    // Try to construct a new VarInfo. Use state.variables.insert which returns the old value
                    state.variables.insert(
                        target_name.clone(),
                        VarInfo {
                            storage: crate::codegen::variables::VarStorage::Stack(counter),
                            var_type: VarType::Int,
                            class_name: None,
                            closure_value_ptr: None,
                        },
                    )
                } else {
                    None
                };

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

                state.loop_stack.pop();

                if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    state.ir_builder.build_branch(state.builder, step_block);
                }

                state.builder.position_at_end(step_block);
                let counter_val = state
                    .builder
                    .build_load(state.context.i64_type(), counter, "counter_val")
                    .expect("load")
                    .into_int_value();
                let next_val = state.ir_builder.build_add(
                    state.builder,
                    counter_val,
                    step_val,
                    "next_counter",
                );
                state.builder.build_store(counter, next_val).expect("store");
                if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    state.ir_builder.build_branch(state.builder, cond_block);
                }

                // Generate else block if it exists
                if let Some(else_stmts) = else_body {
                    state.builder.position_at_end(else_block.unwrap());
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
                    // After else block, jump to exit
                    state.ir_builder.build_branch(state.builder, exit_block);
                }

                state.builder.position_at_end(exit_block);

                // Restore the original shadowed variable, if any
                if let Expr::Ident(target_name, _) = target {
                    if let Some(old) = old_var {
                        state.variables.insert(target_name.clone(), old);
                    } else {
                        state.variables.remove(target_name);
                    }
                }

                return Ok(());
            }
        }
    }

    // Default to list/iterable iteration
    let iter_val = crate::codegen::expressions::generate_expr(state, iter)?;

    // Call vp_list_len to get length
    let list_len_func = state
        .module
        .get_function("vp_list_len")
        .ok_or_else(|| "vp_list_len not declared".to_string())?;

    let end_val = state
        .ir_builder
        .build_call(state.builder, list_len_func, &[iter_val.into()], "len")
        .unwrap()
        .into_int_value();

    let start_val = state.ir_builder.i64_const(0);

    let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let for_num = WHILE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let init_block = state.context.append_basic_block(func_ctx, &format!("for_init{}", for_num));
    let cond_block = state.context.append_basic_block(func_ctx, &format!("for_cond{}", for_num));
    let body_block = state.context.append_basic_block(func_ctx, &format!("for_body{}", for_num));
    let step_block = state.context.append_basic_block(func_ctx, &format!("for_step{}", for_num));
    let exit_block = state.context.append_basic_block(func_ctx, &format!("for_exit{}", for_num));

    state.ir_builder.build_branch(state.builder, init_block);
    state.builder.position_at_end(init_block);
    let counter =
        state.builder.build_alloca(state.context.i64_type(), "for_counter").expect("alloca");
    state.builder.build_store(counter, start_val).expect("store");
    state.ir_builder.build_branch(state.builder, cond_block);

    state.builder.position_at_end(cond_block);
    let counter_val = state
        .builder
        .build_load(state.context.i64_type(), counter, "counter_val")
        .expect("load")
        .into_int_value();
    let cond = state.ir_builder.build_icmp_lt(state.builder, counter_val, end_val, "for_cond");
    state.ir_builder.build_cond_branch(state.builder, cond, body_block, exit_block);

    state.builder.position_at_end(body_block);

    // Check if iterable is a list of floats
    let mut iter_type = crate::codegen::expressions::core::infer_expr_type(iter);
    if let Expr::Ident(name, _) = iter {
        if let Some(t) = state.var_types.get(name) {
            iter_type = t.clone();
        }
    }

    let is_float_list = match &iter_type {
        crate::ast::Type::List(inner) => match &**inner {
            crate::ast::Type::F64 => true,
            crate::ast::Type::Var(n) if n == "float" || n == "f64" => true,
            _ => false,
        },
        crate::ast::Type::GenericApp { name, type_args } if (name == "list" || name == "List") && type_args.len() == 1 => {
            match &type_args[0] {
                crate::ast::Type::F64 => true,
                crate::ast::Type::Var(n) if n == "float" || n == "f64" => true,
                _ => false,
            }
        }
        _ => false,
    };

    let func_name = if is_float_list { "vp_list_get_f64" } else { "vp_list_get" };
    let list_get_func = state
        .module
        .get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;
        
    let item_val = state
        .ir_builder
        .build_call(
            state.builder,
            list_get_func,
            &[iter_val.into(), counter_val.into()],
            "item_val",
        )
        .unwrap();

    // Bind item_val to target (like 'val')
    let old_var = if let Expr::Ident(target_name, _) = target {
        if is_float_list {
            let val_alloca =
                state.builder.build_alloca(state.context.f64_type(), &target_name).expect("alloca");
            state.builder.build_store(val_alloca, item_val.into_float_value()).expect("store");

            state.variables.insert(
                target_name.clone(),
                VarInfo {
                    storage: crate::codegen::variables::VarStorage::Stack(val_alloca),
                    var_type: VarType::Float,
                    class_name: None,
                    closure_value_ptr: None,
                },
            )
        } else {
            let val_alloca =
                state.builder.build_alloca(state.context.i64_type(), &target_name).expect("alloca");
            state.builder.build_store(val_alloca, item_val.into_int_value()).expect("store");

            state.variables.insert(
                target_name.clone(),
                VarInfo {
                    storage: crate::codegen::variables::VarStorage::Stack(val_alloca),
                    var_type: VarType::Int,
                    class_name: None,
                    closure_value_ptr: None,
                },
            )
        }
    } else {
        None
    };

    // Push loop context for break/continue support
    // continue should jump to step block to increment counter
    state.loop_stack.push(LoopContext::new(exit_block, step_block));

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

    state.loop_stack.pop();

    if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
        state.ir_builder.build_branch(state.builder, step_block);
    }

    state.builder.position_at_end(step_block);
    let counter_val = state
        .builder
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
    if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
        state.ir_builder.build_branch(state.builder, cond_block);
    }

    state.builder.position_at_end(exit_block);

    // Restore the original shadowed variable, if any
    if let Expr::Ident(target_name, _) = target {
        if let Some(old) = old_var {
            state.variables.insert(target_name.clone(), old);
        } else {
            state.variables.remove(target_name);
        }
    }

    Ok(())
}

/// Generate an async for loop
/// async for x in async_iter:
///     body
///
/// This generates code that:
/// 1. Calls vp_async_iter(async_iter) to get the async iterator
/// 2. In a loop, calls vp_async_next(iterator) to get the next item
/// 3. Breaks when -1 (StopAsyncIteration) is returned
pub fn generate_async_for<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    iter: &Expr,
    body: &[Stmt],
) -> Result<(), String> {
    let func_ctx = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let for_num = WHILE_COUNTER.fetch_add(1, Ordering::SeqCst);

    let init_block =
        state.context.append_basic_block(func_ctx, &format!("async_for_init{}", for_num));
    let cond_block =
        state.context.append_basic_block(func_ctx, &format!("async_for_cond{}", for_num));
    let body_block =
        state.context.append_basic_block(func_ctx, &format!("async_for_body{}", for_num));
    let step_block =
        state.context.append_basic_block(func_ctx, &format!("async_for_step{}", for_num));
    let exit_block =
        state.context.append_basic_block(func_ctx, &format!("async_for_exit{}", for_num));

    // Branch to init block
    state.ir_builder.build_branch(state.builder, init_block);

    // Init block: create the async iterator
    state.builder.position_at_end(init_block);

    // Check if iter is a call to async_range(...)
    let iterator = if let Expr::Call { func, args, .. } = iter {
        if let Expr::Ident(name, _) = func.as_ref() {
            if name == "async_range" {
                // Call vp_async_range_create(start, end, step)
                let range_create_func = state
                    .module
                    .get_function("vp_async_range_create")
                    .ok_or_else(|| "vp_async_range_create not declared".to_string())?;

                let (start_val, end_val, step_val) = match args.len() {
                    1 => (
                        state.ir_builder.i64_const(0),
                        crate::codegen::expressions::generate_expr(state, &args[0])?
                            .into_int_value(),
                        state.ir_builder.i64_const(1),
                    ),
                    2 => (
                        crate::codegen::expressions::generate_expr(state, &args[0])?
                            .into_int_value(),
                        crate::codegen::expressions::generate_expr(state, &args[1])?
                            .into_int_value(),
                        state.ir_builder.i64_const(1),
                    ),
                    3 => (
                        crate::codegen::expressions::generate_expr(state, &args[0])?
                            .into_int_value(),
                        crate::codegen::expressions::generate_expr(state, &args[1])?
                            .into_int_value(),
                        crate::codegen::expressions::generate_expr(state, &args[2])?
                            .into_int_value(),
                    ),
                    _ => return Err("async_range expects 1-3 arguments".to_string()),
                };

                let iter_ptr = state
                    .ir_builder
                    .build_call(
                        state.builder,
                        range_create_func,
                        &[start_val.into(), end_val.into(), step_val.into()],
                        "async_range_iter",
                    )
                    .ok_or_else(|| "async_range_create failed".to_string())?;
                iter_ptr.into_pointer_value()
            } else {
                // Not async_range, call vp_async_iter on the expression result
                let iter_val = crate::codegen::expressions::generate_expr(state, iter)?;
                let aiter_func = state
                    .module
                    .get_function("vp_async_iter")
                    .ok_or_else(|| "vp_async_iter not declared".to_string())?;
                state
                    .ir_builder
                    .build_call(state.builder, aiter_func, &[iter_val.into()], "async_iterator")
                    .ok_or_else(|| "vp_async_iter failed".to_string())?
                    .into_pointer_value()
            }
        } else {
            // Not an identifier, call vp_async_iter on the expression result
            let iter_val = crate::codegen::expressions::generate_expr(state, iter)?;
            let aiter_func = state
                .module
                .get_function("vp_async_iter")
                .ok_or_else(|| "vp_async_iter not declared".to_string())?;
            state
                .ir_builder
                .build_call(state.builder, aiter_func, &[iter_val.into()], "async_iterator")
                .ok_or_else(|| "vp_async_iter failed".to_string())?
                .into_pointer_value()
        }
    } else {
        // Not a call expression, call vp_async_iter on the expression result
        let iter_val = crate::codegen::expressions::generate_expr(state, iter)?;
        let aiter_func = state
            .module
            .get_function("vp_async_iter")
            .ok_or_else(|| "vp_async_iter not declared".to_string())?;
        state
            .ir_builder
            .build_call(state.builder, aiter_func, &[iter_val.into()], "async_iterator")
            .ok_or_else(|| "vp_async_iter failed".to_string())?
            .into_pointer_value()
    };

    // Store iterator in alloca
    let iterator_ptr =
        state.builder.build_alloca(iterator.get_type(), "iterator_ptr").expect("alloca");
    state.builder.build_store(iterator_ptr, iterator).expect("store");

    state.ir_builder.build_branch(state.builder, cond_block);

    // Cond block: call __anext__ and check for StopAsyncIteration
    state.builder.position_at_end(cond_block);
    let iterator_val =
        state.builder.build_load(iterator.get_type(), iterator_ptr, "iterator_val").expect("load");

    // Call __anext__ on the iterator
    let anext_func = state
        .module
        .get_function("vp_async_next")
        .ok_or_else(|| "vp_async_next not declared".to_string())?;
    let next_future = state
        .ir_builder
        .build_call(state.builder, anext_func, &[iterator_val.into()], "anext_future")
        .ok_or_else(|| "async for: anext failed".to_string())?;

    // The result is directly the next value (or 0 for StopAsyncIteration)
    // We don't need to await since vp_async_next returns directly
    let item = next_future;

    // Check if item is -1 (StopAsyncIteration)
    let is_not_done = state
        .builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            item.into_int_value(),
            state.ir_builder.i64_const(-1),
            "is_not_done",
        )
        .expect("icmp");

    state.ir_builder.build_cond_branch(state.builder, is_not_done, body_block, exit_block);

    // Body block: execute the loop body with the item
    state.builder.position_at_end(body_block);

    // Allocate storage for the iteration variable and store the value
    let item_alloca = if let Expr::Ident(target_name, _) = target {
        let alloca =
            state.builder.build_alloca(state.context.i64_type(), &target_name).expect("alloca");
        state.builder.build_store(alloca, item.into_int_value()).expect("store");
        Some((target_name.clone(), alloca))
    } else {
        None
    };

    // Bind the target variable to the item
    let old_var = if let Some((target_name, alloca)) = &item_alloca {
        state.variables.insert(
            target_name.clone(),
            VarInfo {
                storage: crate::codegen::variables::VarStorage::Stack(*alloca),
                var_type: VarType::Int,
                class_name: None,
                closure_value_ptr: None,
            },
        )
    } else {
        None
    };

    // Push loop context for break/continue support
    // continue should jump to step block for next iteration
    state.loop_stack.push(LoopContext::new(exit_block, step_block));

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

    state.loop_stack.pop();

    if state.builder.get_insert_block().unwrap().get_terminator().is_none() {
        state.ir_builder.build_branch(state.builder, step_block);
    }

    // Step block: continue to next iteration
    state.builder.position_at_end(step_block);
    state.ir_builder.build_branch(state.builder, cond_block);

    // Exit block
    state.builder.position_at_end(exit_block);

    // Restore the original shadowed variable, if any
    if let Expr::Ident(target_name, _) = target {
        if let Some(old) = old_var {
            state.variables.insert(target_name.clone(), old);
        } else {
            state.variables.remove(target_name);
        }
    }

    Ok(())
}
