use super::*;
use crate::ast::{Expr, Stmt, Type};
use crate::codegen::state::CodeGenState;
use crate::utils::mangle_function_name;

pub(crate) fn generate_sync<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    body: &[Stmt],
) -> Result<(), String> {
    // Sync block - execute body and wait for all tasks
    for stmt in body {
        generate_stmt_internal(state, stmt)?;
    }
    // Tasks inside sync block are automatically waited
    if let Some(wait_func) = state.module.get_function("vp_wait_all_tasks") {
        let _ = state.builder.build_call(wait_func, &[], "wait_all");
        Ok(())
    } else {
        Err("vp_wait_all_tasks not declared".to_string())
    }
}

pub(crate) fn generate_task<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    call: &Expr,
    span: &crate::utils::Span,
) -> Result<(), String> {
    // Task spawn - submit function to thread pool for parallel execution
    if let Expr::Call { func, args, .. } = call {
        if let Expr::Ident(name, _) = func.as_ref() {
            // Evaluate all arguments first to get their types
            let mut arg_values = Vec::new();
            for a in args {
                let v = crate::codegen::expressions::generate_expr(state, a)?;
                arg_values.push(v);
            }

            // Compute mangled name from argument types
            let arg_types: Vec<Type> = arg_values.iter().map(|v| {
                // Map LLVM type back to Viper Type for mangling
                use inkwell::values::BasicValueEnum;
                match v {
                    BasicValueEnum::IntValue(_) => Type::I64,
                    BasicValueEnum::FloatValue(_) => Type::F64,
                    BasicValueEnum::PointerValue(_) => Type::Str, // Simplified
                    _ => Type::I64, // Default
                }
            }).collect();

            let mangled_name = mangle_function_name(name, &arg_types);

            // Look up function with mangled name (with fallback)
            let func_val = if let Some(&f) = state.functions.get(&mangled_name) {
                Some(f)
            } else {
                // Fallback: find any function that starts with the name
                state
                    .functions
                    .iter()
                    .find(|(k, _)| k.starts_with(&format!("{}_", name)))
                    .map(|(_, v)| *v)
            };

            if let Some(func_val) = func_val {
                // Create a struct type to pack all arguments
                let arg_types: Vec<_> = arg_values.iter().map(|v| v.get_type().into()).collect();
                let struct_type = state.context.struct_type(&arg_types, false);

                // Generate wrapper function: void wrapper(void* args)
                let void_type = state.context.void_type();
                let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                let wrapper_fn_type = void_type.fn_type(&[ptr_type.into()], false);
                let wrapper_name = format!("__task_wrapper_{}_{}", name, span.start);
                let wrapper_fn = state.module.add_function(&wrapper_name, wrapper_fn_type, None);

                // Generate wrapper body
                let wrapper_entry = state.context.append_basic_block(wrapper_fn, "entry");
                let old_block = state.builder.get_insert_block();

                state.builder.position_at_end(wrapper_entry);
                let arg_ptr = wrapper_fn.get_first_param().unwrap().into_pointer_value();

                // Unpack arguments from the struct
                let mut call_args = Vec::new();
                for (i, arg_val) in arg_values.iter().enumerate() {
                    let gep = state
                        .builder
                        .build_struct_gep(struct_type, arg_ptr, i as u32, "struct_gep")
                        .unwrap();
                    let val = state.builder.build_load(arg_val.get_type(), gep, "arg").unwrap();
                    call_args.push(val.into());
                }

                // Call actual function
                let _ = state.builder.build_call(func_val, &call_args, "call");

                // Free the args struct
                let free_func = state.module.get_function("free").unwrap();
                let _ = state.builder.build_call(free_func, &[arg_ptr.into()], "free");

                let _ = state.builder.build_return(None);

                // Restore builder
                if let Some(ob) = old_block {
                    state.builder.position_at_end(ob);
                }

                // Call malloc to allocate struct on heap
                let malloc_func = state.module.get_function("malloc").unwrap();
                let struct_size = struct_type.size_of().unwrap();
                let malloc_call =
                    state.builder.build_call(malloc_func, &[struct_size.into()], "malloc").unwrap();
                let heap_ptr = match malloc_call.try_as_basic_value() {
                    inkwell::values::ValueKind::Basic(val) => val.into_pointer_value(),
                    _ => panic!("Expected pointer from malloc"),
                };

                // Pack arguments into heap struct
                for (i, arg_val) in arg_values.iter().enumerate() {
                    let gep = state
                        .builder
                        .build_struct_gep(struct_type, heap_ptr, i as u32, "struct_gep")
                        .unwrap();
                    let _ = state.builder.build_store(gep, *arg_val);
                }

                // Submit task to thread pool
                let submit_func = state.module.get_function("vp_submit_task").unwrap();
                let wrapper_fn_ptr = wrapper_fn.as_global_value().as_pointer_value();
                let _ = state.builder.build_call(
                    submit_func,
                    &[wrapper_fn_ptr.into(), heap_ptr.into()],
                    "submit",
                );
                Ok(())
            } else {
                Err(format!("Unknown function for task: {}", name))
            }
        } else {
            // Non-identifier call - just execute inline
            crate::codegen::expressions::generate_expr(state, call)?;
            Ok(())
        }
    } else {
        // Not a call expression - just execute
        crate::codegen::expressions::generate_expr(state, call)?;
        Ok(())
    }
}

pub(crate) fn generate_chan<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    size: &Expr,
) -> Result<(), String> {
    // Channel creation - call runtime function
    let size_val = crate::codegen::expressions::generate_expr(state, size)?;
    let chan_func =
        state.module.get_function("vp_chan_create").ok_or("vp_chan_create not declared")?;
    state.builder.build_call(chan_func, &[size_val.into()], "chan").expect("call vp_chan_create");
    Ok(())
}

pub(crate) fn generate_send<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    chan: &Expr,
    value: &Expr,
) -> Result<(), String> {
    // Channel send
    let chan_val = crate::codegen::expressions::generate_expr(state, chan)?;
    let val_val = crate::codegen::expressions::generate_expr(state, value)?;
    let send_func = state.module.get_function("vp_chan_send").ok_or("vp_chan_send not declared")?;
    state
        .builder
        .build_call(send_func, &[chan_val.into(), val_val.into()], "")
        .expect("call vp_chan_send");
    Ok(())
}

pub(crate) fn generate_recv<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    chan: &Expr,
) -> Result<(), String> {
    // Channel receive - returns value from channel
    let chan_val = crate::codegen::expressions::generate_expr(state, chan)?;
    let recv_func = state.module.get_function("vp_chan_recv").ok_or("vp_chan_recv not declared")?;
    state.builder.build_call(recv_func, &[chan_val.into()], "recv_val").expect("call vp_chan_recv");
    Ok(())
}

pub(crate) fn generate_waitgroup<'ctx>(state: &mut CodeGenState<'_, 'ctx>) -> Result<(), String> {
    // WaitGroup creation - returns a pointer to WaitGroup struct
    let wg_func = state
        .module
        .get_function("vp_waitgroup_create")
        .ok_or("vp_waitgroup_create not declared")?;
    state.builder.build_call(wg_func, &[], "wg").expect("call vp_waitgroup_create");
    Ok(())
}

pub(crate) fn generate_wg_add<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    wg: &Expr,
    n: &Expr,
) -> Result<(), String> {
    // WaitGroup add
    let wg_val = crate::codegen::expressions::generate_expr(state, wg)?;
    let n_val = crate::codegen::expressions::generate_expr(state, n)?;
    let add_func =
        state.module.get_function("vp_waitgroup_add").ok_or("vp_waitgroup_add not declared")?;
    state
        .builder
        .build_call(add_func, &[wg_val.into(), n_val.into()], "")
        .expect("call vp_waitgroup_add");
    Ok(())
}

pub(crate) fn generate_wg_done<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    wg: &Expr,
) -> Result<(), String> {
    // WaitGroup done
    let wg_val = crate::codegen::expressions::generate_expr(state, wg)?;
    let done_func =
        state.module.get_function("vp_waitgroup_done").ok_or("vp_waitgroup_done not declared")?;
    state.builder.build_call(done_func, &[wg_val.into()], "").expect("call vp_waitgroup_done");
    Ok(())
}

pub(crate) fn generate_wg_wait<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    wg: &Expr,
) -> Result<(), String> {
    // WaitGroup wait
    let wg_val = crate::codegen::expressions::generate_expr(state, wg)?;
    let wait_func =
        state.module.get_function("vp_waitgroup_wait").ok_or("vp_waitgroup_wait not declared")?;
    state.builder.build_call(wait_func, &[wg_val.into()], "").expect("call vp_waitgroup_wait");
    Ok(())
}
