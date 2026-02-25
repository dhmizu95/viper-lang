//! Statement code generation for Viper

use crate::ast::{BinOp, Expr, Stmt};
use inkwell::context::Context;
use inkwell::values::{FunctionValue, GlobalValue};
use std::collections::HashMap;

use crate::codegen::builder::IRBuilder;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{LoopContext, VarInfo, VarStorage, VarType};
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
        escape_analyzer,
        current_function,
    );

    generate_stmt_internal(&mut state, stmt)
}

/// Internal statement generation
fn generate_stmt_internal<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    stmt: &Stmt,
) -> Result<(), String> {
    match stmt {
        Stmt::Expr(expr) => {
            crate::codegen::expressions::generate_expr(state, expr)?;
        }
        Stmt::Assign { target, value, .. } => {
            generate_assign(state, target, value)?;
        }
        Stmt::AugAssign {
            target, op, value, ..
        } => {
            generate_aug_assign(state, target, op, value)?;
        }
        Stmt::Declare {
            name,
            value,
            mutable,
            ..
        } => {
            generate_declare(state, name, *mutable, value)?;
        }
        Stmt::Return { value, .. } => {
            return crate::codegen::control_flow::generate_return(state, value);
        }
        Stmt::If {
            condition,
            body,
            elif_blocks,
            else_body,
            ..
        } => {
            return crate::codegen::control_flow::generate_if(
                state,
                condition,
                body,
                elif_blocks,
                else_body,
            );
        }
        Stmt::While {
            condition, body, ..
        } => {
            return crate::codegen::control_flow::generate_while(state, condition, body);
        }
        Stmt::For {
            target, iter, body, ..
        } => {
            return crate::codegen::control_flow::generate_for(state, target, iter, body);
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
            // Sync block - execute body and wait for all tasks
            for stmt in body {
                generate_stmt_internal(state, stmt)?;
            }
            // Tasks inside sync block are automatically waited
        }
        Stmt::Task { call, span } => {
            // Task spawn - submit function to thread pool for parallel execution
            if let Expr::Call { func, args, .. } = call {
                if let Expr::Ident(name, _) = func.as_ref() {
                    if let Some(func_val) = state.functions.get(name) {
                        // Evaluate all arguments first
                        let arg_values: Vec<_> = args
                            .iter()
                            .map(|a| {
                                crate::codegen::expressions::generate_expr(state, a)
                                    .map(|v| v.into())
                            })
                            .collect::<Result<_, _>>()?;

                        // For parallel execution, we need to:
                        // 1. Create a wrapper that captures the arguments
                        // 2. Submit it to the thread pool
                        //
                        // For simplicity, we'll call vp_submit_task with a wrapper
                        // The wrapper will call the actual function with captured args
                        //
                        // For now, run inline but note this is where parallel spawn goes
                        let func_ptr = *func_val;
                        let _result = state.ir_builder.build_call(
                            state.builder,
                            func_ptr,
                            &arg_values,
                            &format!("task_{}", span.start),
                        );
                    } else {
                        return Err(format!("Unknown function for task: {}", name));
                    }
                } else {
                    // Non-identifier call - just execute inline
                    crate::codegen::expressions::generate_expr(state, call)?;
                }
            } else {
                // Not a call expression - just execute
                crate::codegen::expressions::generate_expr(state, call)?;
            }
        }
        Stmt::Chan { size, .. } => {
            // Channel creation - call runtime function
            let size_val = crate::codegen::expressions::generate_expr(state, size)?;
            let chan_func = state
                .module
                .get_function("vp_chan_create")
                .ok_or("vp_chan_create not declared")?;
            state
                .builder
                .build_call(chan_func, &[size_val.into()], "chan")
                .expect("call vp_chan_create");
            // Channel is a pointer type - the result is already a pointer value
        }
        Stmt::Send { chan, value, .. } => {
            // Channel send
            let chan_val = crate::codegen::expressions::generate_expr(state, chan)?;
            let val_val = crate::codegen::expressions::generate_expr(state, value)?;
            let send_func = state
                .module
                .get_function("vp_chan_send")
                .ok_or("vp_chan_send not declared")?;
            state
                .builder
                .build_call(send_func, &[chan_val.into(), val_val.into()], "")
                .expect("call vp_chan_send");
        }
        Stmt::Recv { chan, .. } => {
            // Channel receive - returns value from channel
            let chan_val = crate::codegen::expressions::generate_expr(state, chan)?;
            let recv_func = state
                .module
                .get_function("vp_chan_recv")
                .ok_or("vp_chan_recv not declared")?;
            state
                .builder
                .build_call(recv_func, &[chan_val.into()], "recv_val")
                .expect("call vp_chan_recv");
            // Return value type depends on channel element type (handled by type checker)
        }
        Stmt::WaitGroup { .. } => {
            // WaitGroup creation - returns a pointer to WaitGroup struct
            let wg_func = state
                .module
                .get_function("vp_waitgroup_create")
                .ok_or("vp_waitgroup_create not declared")?;
            state
                .builder
                .build_call(wg_func, &[], "wg")
                .expect("call vp_waitgroup_create");
            // WaitGroup is a pointer type
        }
        Stmt::WgAdd { wg, n, .. } => {
            // WaitGroup add
            let wg_val = crate::codegen::expressions::generate_expr(state, wg)?;
            let n_val = crate::codegen::expressions::generate_expr(state, n)?;
            let add_func = state
                .module
                .get_function("vp_waitgroup_add")
                .ok_or("vp_waitgroup_add not declared")?;
            state
                .builder
                .build_call(add_func, &[wg_val.into(), n_val.into()], "")
                .expect("call vp_waitgroup_add");
        }
        Stmt::WgDone { wg, .. } => {
            // WaitGroup done
            let wg_val = crate::codegen::expressions::generate_expr(state, wg)?;
            let done_func = state
                .module
                .get_function("vp_waitgroup_done")
                .ok_or("vp_waitgroup_done not declared")?;
            state
                .builder
                .build_call(done_func, &[wg_val.into()], "")
                .expect("call vp_waitgroup_done");
        }
        Stmt::WgWait { wg, .. } => {
            // WaitGroup wait
            let wg_val = crate::codegen::expressions::generate_expr(state, wg)?;
            let wait_func = state
                .module
                .get_function("vp_waitgroup_wait")
                .ok_or("vp_waitgroup_wait not declared")?;
            state
                .builder
                .build_call(wait_func, &[wg_val.into()], "")
                .expect("call vp_waitgroup_wait");
        }
        _ => {}
    }
    Ok(())
}

/// Generate assignment statement
fn generate_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    value: &Expr,
) -> Result<(), String> {
    if let Expr::Ident(name, _) = target {
        let val = crate::codegen::expressions::generate_expr(state, value)?;

        // Check if the value is a stack-allocated array (should not use ARC)
        let is_stack_array = matches!(value, Expr::Array { .. });

        if let Some(var_info) = state.variables.get(name) {
            // Check if we're replacing a reference type value
            let old_is_ref = var_info.var_type == VarType::Pointer;
            let old_needs_arc = state.needs_arc(name);

            // Update existing variable
            match &var_info.storage {
                VarStorage::Stack(alloca) => {
                    // Release old value if it was a reference type needing ARC
                    if old_is_ref && old_needs_arc {
                        let old_val = state
                            .builder
                            .build_load(
                                state.context.ptr_type(inkwell::AddressSpace::default()),
                                *alloca,
                                &format!("{}_old", name),
                            )
                            .expect("load old value");
                        state.build_release(old_val, &format!("{}_old", name));
                    }
                    state.builder.build_store(*alloca, val).expect("store");
                }
                VarStorage::Register(_) => {
                    // For scalar types, just keep register allocation -
                    // we replace the register value
                }
            }

            // Retain new value if it's a reference type that escapes (but not stack arrays)
            let is_ref_type = val.is_pointer_value();
            let needs_arc = state.needs_arc(name);
            if is_ref_type && needs_arc && !is_stack_array {
                state.build_retain(val, name);
            }
        } else {
            let ty = val.get_type();

            // Determine if this is a reference type (but not stack arrays)
            let is_ref_type = val.is_pointer_value() && !is_stack_array;

            // Set reference type flag in escape analyzer
            state.set_reference_type(name, is_ref_type);

            let var_type = if val.is_float_value() {
                VarType::Float
            } else if val.is_pointer_value() {
                VarType::Pointer
            } else {
                VarType::Int
            };

            // For scalar types (int, float), always use register allocation
            // to avoid LLVM dominance issues with loop variables
            let is_scalar = !is_ref_type;

            if is_scalar {
                // Use register allocation for scalars - no need for stack
                state
                    .variables
                    .insert(name.clone(), VarInfo::new_register(val, var_type));
            } else {
                // For reference types, use stack allocation
                // Note: We used to try to put this in the entry block, but that causes
                // issues with basic block structure. Instead, we rely on escape analysis
                // to ensure reference types don't cause dominance issues.
                let alloca = state.builder.build_alloca(ty, name).expect("alloca");
                state.builder.build_store(alloca, val).expect("store");
                state
                    .variables
                    .insert(name.clone(), VarInfo::new_stack(alloca, var_type));

                // Insert ARC retain if this is a reference type that escapes (but not stack arrays)
                if is_ref_type {
                    state.build_retain(val, name);
                }
            }
        }
    } else if let Expr::Index { obj, index, .. } = target {
        let obj_val = crate::codegen::expressions::generate_expr(state, obj)?;
        let index_val = crate::codegen::expressions::generate_expr(state, index)?.into_int_value();
        let value_val = crate::codegen::expressions::generate_expr(state, value)?;

        // Check if this is an array (pointer) or list
        if obj_val.is_pointer_value() {
            // Array index assignment using GEP and store
            let obj_ptr = obj_val.into_pointer_value();
            let elem_type = value_val.get_type();

            let elem_ptr = unsafe {
                state
                    .builder
                    .build_in_bounds_gep(elem_type, obj_ptr, &[index_val], "array_elem")
            }
            .map_err(|e| format!("Failed to build array index GEP: {:?}", e))?;

            state
                .builder
                .build_store(elem_ptr, value_val)
                .map_err(|e| format!("Failed to store array element: {:?}", e))?;
        } else {
            // List index assignment using runtime function
            let list_set = state
                .module
                .get_function("vp_list_set")
                .ok_or_else(|| "vp_list_set not declared".to_string())?;
            state.ir_builder.build_call(
                state.builder,
                list_set,
                &[obj_val.into(), index_val.into(), value_val.into()],
                "list_set",
            );
        }
    }
    Ok(())
}

/// Generate augmented assignment statement
fn generate_aug_assign<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    target: &Expr,
    op: &BinOp,
    value: &Expr,
) -> Result<(), String> {
    if let Expr::Ident(name, _) = target {
        if let Some(var_info) = state.variables.get(name) {
            let var_type = var_info.var_type;
            let is_scalar = matches!(var_type, VarType::Int | VarType::Float);

            // Get current value
            let current = if is_scalar {
                // For scalars in registers, just use the value directly
                if let VarStorage::Register(val) = &var_info.storage {
                    *val
                } else {
                    // For scalars in stack, load it
                    if let VarStorage::Stack(alloca) = &var_info.storage {
                        match var_type {
                            VarType::Float => {
                                let f64_type = state.context.f64_type();
                                state
                                    .builder
                                    .build_load(f64_type, *alloca, name)
                                    .expect("load")
                            }
                            VarType::Int => {
                                let i64_type = state.context.i64_type();
                                state
                                    .builder
                                    .build_load(i64_type, *alloca, name)
                                    .expect("load")
                            }
                            _ => return Err("Invalid var type".to_string()),
                        }
                    } else {
                        return Err("Invalid storage".to_string());
                    }
                }
            } else {
                // For pointers, load from stack
                if let VarStorage::Stack(alloca) = &var_info.storage {
                    let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                    state
                        .builder
                        .build_load(ptr_type, *alloca, name)
                        .expect("load")
                } else {
                    return Err("Reference types must be stack allocated".to_string());
                }
            };

            let new_val = crate::codegen::expressions::generate_expr(state, value)?;

            let result: inkwell::values::BasicValueEnum<'ctx> = if var_type == VarType::Float {
                let lhs = current.into_float_value();
                let rhs = new_val.into_float_value();
                match op {
                    BinOp::Add => state
                        .builder
                        .build_float_add(lhs, rhs, "fadd")
                        .expect("fadd"),
                    BinOp::Sub => state
                        .builder
                        .build_float_sub(lhs, rhs, "fsub")
                        .expect("fsub"),
                    BinOp::Mul => state
                        .builder
                        .build_float_mul(lhs, rhs, "fmul")
                        .expect("fmul"),
                    BinOp::Div => state
                        .builder
                        .build_float_div(lhs, rhs, "fdiv")
                        .expect("fdiv"),
                    _ => {
                        return Err(format!(
                            "Unsupported augmented assignment operator for float: {:?}",
                            op
                        ))
                    }
                }
                .into()
            } else {
                let lhs = current.into_int_value();
                let rhs = new_val.into_int_value();
                match op {
                    BinOp::Add => state.ir_builder.build_add(state.builder, lhs, rhs, "add"),
                    BinOp::Sub => state.ir_builder.build_sub(state.builder, lhs, rhs, "sub"),
                    BinOp::Mul => state.ir_builder.build_mul(state.builder, lhs, rhs, "mul"),
                    BinOp::Div => state.ir_builder.build_div(state.builder, lhs, rhs, "div"),
                    BinOp::Mod => state
                        .builder
                        .build_int_signed_rem(lhs, rhs, "mod")
                        .expect("mod"),
                    BinOp::FloorDiv => {
                        state
                            .ir_builder
                            .build_div(state.builder, lhs, rhs, "floordiv")
                    }
                    _ => {
                        return Err(format!(
                            "Unsupported augmented assignment operator for int: {:?}",
                            op
                        ))
                    }
                }
                .into()
            };

            // Store result back
            if let Some(var_info) = state.variables.get(name) {
                match &var_info.storage {
                    VarStorage::Stack(alloca) => {
                        state.builder.build_store(*alloca, result).expect("store");
                    }
                    VarStorage::Register(_) => {
                        // For register allocation, just update the value
                        state
                            .variables
                            .insert(name.clone(), VarInfo::new_register(result, var_type));
                    }
                }
            }
        } else {
            return Err(format!(
                "Undefined variable in augmented assignment: {}",
                name
            ));
        }
    }
    Ok(())
}

/// Generate variable declaration
fn generate_declare<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    mutable: bool,
    value: &Option<Expr>,
) -> Result<(), String> {
    if let Some(val) = value {
        let val = crate::codegen::expressions::generate_expr(state, val)?;
        let ty = val.get_type();

        // Use escape analysis to determine allocation strategy
        let can_stack_alloc = state.can_stack_allocate(name);

        // Determine if this is a reference type (pointer)
        // Chan[T] and WaitGroup are always pointer types
        let is_ref_type = val.is_pointer_value();

        // Set reference type flag in escape analyzer
        state.set_reference_type(name, is_ref_type);

        let var_type = if val.is_float_value() {
            VarType::Float
        } else if val.is_pointer_value() {
            VarType::Pointer
        } else {
            VarType::Int
        };

        // For scalar types (int, float), use stack allocation if mutable
        // to allow reassignment in loops
        let is_scalar = !is_ref_type;
        let use_stack = !can_stack_alloc || is_scalar || mutable;

        if !use_stack {
            // Use SSA register allocation for non-escaping variables or non-mutable scalars
            state
                .variables
                .insert(name.to_string(), VarInfo::new_register(val, var_type));
        } else {
            // Use stack allocation (alloca) for escaping variables or mutable scalars
            // Create alloca in function entry block to satisfy LLVM dominance
            let func = state
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();
            let entry_block = func.get_first_basic_block().unwrap();
            let old_builder_pos = state.builder.get_insert_block();

            state.builder.position_at_end(entry_block);
            let alloca = state.builder.build_alloca(ty, name).expect("alloca");

            // Restore builder position
            if let Some(pos) = old_builder_pos {
                state.builder.position_at_end(pos);
            }

            state.builder.build_store(alloca, val).expect("store");
            state
                .variables
                .insert(name.to_string(), VarInfo::new_stack(alloca, var_type));

            // Insert ARC retain if this is a reference type that escapes
            if is_ref_type {
                state.build_retain(val, name);
            }
        }
    }
    Ok(())
}
