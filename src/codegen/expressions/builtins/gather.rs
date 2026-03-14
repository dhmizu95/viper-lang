//! Gather builtin for async/await - wait for multiple futures concurrently

use crate::ast::Expr;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate gather(f1, f2, ...) call
/// Takes variadic futures and returns a list of results
pub fn generate_gather_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let i64_type = state.context.i64_type();
    
    if args.is_empty() {
        // Return empty list
        let list_create = state.module.get_function("vp_list_create")
            .ok_or_else(|| "vp_list_create not found".to_string())?;
        return Ok(state.ir_builder.build_call(
            state.builder,
            list_create,
            &[state.ir_builder.i64_const(0).into()],
            "empty_list",
        ).unwrap());
    }
    
    // Generate all future values first
    let mut future_vals = Vec::new();
    for arg in args {
        let val = crate::codegen::expressions::generate_expr(state, arg)?;
        future_vals.push(val);
    }
    
    // Allocate array on stack to hold future pointers (as i64)
    let futures_array_type = i64_type.array_type(future_vals.len() as u32);
    let futures_array = state.builder.build_alloca(futures_array_type, "futures_array")
        .expect("alloca futures array");
    
    // Store each future pointer in the array (as i64)
    for (i, future_val) in future_vals.iter().enumerate() {
        let gep = unsafe {
            state
                .builder
                .build_in_bounds_gep(
                    futures_array_type,
                    futures_array,
                    &[
                        state.context.i32_type().const_zero(),
                        state.context.i32_type().const_int(i as u64, false),
                    ],
                    &format!("future_gep{}", i),
                )
                .unwrap()
        };
        
        // Convert pointer to i64 for the gather function
        let future_i64 = if future_val.is_pointer_value() {
            state.builder.build_ptr_to_int(
                future_val.into_pointer_value(),
                i64_type,
                &format!("future_i64{}", i),
            ).unwrap()
        } else {
            future_val.into_int_value()
        };
        
        state.builder.build_store(gep, future_i64).expect("store future");
    }
    
    // Get pointer to first element (as i64 pointer)
    let futures_ptr = unsafe {
        state
            .builder
            .build_in_bounds_gep(
                futures_array_type,
                futures_array,
                &[
                    state.context.i32_type().const_zero(),
                    state.context.i32_type().const_zero(),
                ],
                "futures_ptr",
            )
            .unwrap()
    };
    
    // Call vp_future_gather(futures_ptr, count)
    let gather_func = state.module.get_function("vp_future_gather")
        .ok_or_else(|| "vp_future_gather not found".to_string())?;

    // Convert pointer to i64 for the gather function
    let futures_ptr_i64 = state.builder.build_ptr_to_int(
        futures_ptr,
        i64_type,
        "futures_ptr_i64",
    ).unwrap();

    let results_ptr = state.ir_builder.build_call(
        state.builder,
        gather_func,
        &[futures_ptr_i64.into(), state.ir_builder.i64_const(args.len() as i64).into()],
        "gather_results",
    ).unwrap().into_int_value();
    
    // Create result list
    let list_create = state.module.get_function("vp_list_create")
        .ok_or_else(|| "vp_list_create not found".to_string())?;

    let result_list = state.ir_builder.build_call(
        state.builder,
        list_create,
        &[],
        "result_list",
    ).unwrap().into_pointer_value();
    
    // Unpack results and append to list
    let i64_ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
    let results_array_ptr = state.builder.build_int_to_ptr(
        results_ptr,
        i64_ptr_type,
        "results_array_ptr",
    ).unwrap();
    
    for i in 0..args.len() {
        let result_gep = unsafe {
            state
                .builder
                .build_in_bounds_gep(
                    i64_type,
                    results_array_ptr,
                    &[state.context.i32_type().const_int(i as u64, false)],
                    &format!("result_gep{}", i),
                )
                .unwrap()
        };
        
        let result_val = state.builder.build_load(
            i64_type,
            result_gep,
            &format!("result{}", i),
        ).unwrap();
        
        // Append to list
        let list_append = state.module.get_function("vp_list_append")
            .ok_or_else(|| "vp_list_append not found".to_string())?;
        
        state.ir_builder.build_call(
            state.builder,
            list_append,
            &[result_list.into(), result_val.into()],
            &format!("append{}", i),
        );
    }
    
    // Free the results array
    let gather_free = state.module.get_function("vp_future_gather_free")
        .ok_or_else(|| "vp_future_gather_free not found".to_string())?;
    
    state.ir_builder.build_call(
        state.builder,
        gather_free,
        &[results_ptr.into(), state.ir_builder.i64_const(args.len() as i64).into()],
        "free_results",
    );
    
    Ok(result_list.into())
}
