//! Expression code generation for Viper

use super::*;

use crate::ast::Expr;

use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;

/* ============================================ */
/* Concurrency Builtins (Phase 3)               */
/* ============================================ */

/// Generate chan(size) - create a channel
pub fn generate_chan_create<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "chan() takes 1 argument (capacity), got {}",
            args.len()
        ));
    }

    let size_val = generate_expr(state, &args[0])?;
    let chan_func = state
        .module
        .get_function("vp_chan_create")
        .ok_or_else(|| "vp_chan_create not declared".to_string())?;

    let result = state.ir_builder.build_call(state.builder, chan_func, &[size_val.into()], "chan");
    // Return the pointer value directly
    Ok(result.expect("chan_create"))
}

/// Generate send(chan, value) - send to channel
pub fn generate_chan_send<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 2 {
        return crate::codegen::codegen_error(format!(
            "send() takes 2 arguments (chan, value), got {}",
            args.len()
        ));
    }

    let chan_val = generate_expr(state, &args[0])?;
    let val_val = generate_expr(state, &args[1])?;
    let send_func = state
        .module
        .get_function("vp_chan_send")
        .ok_or_else(|| "vp_chan_send not declared".to_string())?;

    state.ir_builder.build_call(
        state.builder,
        send_func,
        &[chan_val.into(), val_val.into()],
        "send",
    );
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate recv(chan) - receive from channel
pub fn generate_chan_recv<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "recv() takes 1 argument (chan), got {}",
            args.len()
        ));
    }

    let chan_val = generate_expr(state, &args[0])?;
    let recv_func = state
        .module
        .get_function("vp_chan_recv")
        .ok_or_else(|| "vp_chan_recv not declared".to_string())?;

    let result =
        state.ir_builder.build_call(state.builder, recv_func, &[chan_val.into()], "recv_val");
    // Return the pointer value directly (received value is a pointer)
    Ok(result.expect("recv"))
}

/// Generate WaitGroup() - create a wait group
pub fn generate_waitgroup_create<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if !args.is_empty() {
        return crate::codegen::codegen_error(format!(
            "WaitGroup() takes no arguments, got {}",
            args.len()
        ));
    }

    let wg_func = state
        .module
        .get_function("vp_waitgroup_create")
        .ok_or_else(|| "vp_waitgroup_create not declared".to_string())?;

    let result = state.ir_builder.build_call(state.builder, wg_func, &[], "wg");
    // Return the pointer value directly
    Ok(result.expect("wg_create"))
}

/// Generate add(wg, n) - add to wait group
pub fn generate_waitgroup_add<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 2 {
        return crate::codegen::codegen_error(format!(
            "add() takes 2 arguments (wg, n), got {}",
            args.len()
        ));
    }

    let wg_val = generate_expr(state, &args[0])?;
    let n_val = generate_expr(state, &args[1])?;
    let add_func = state
        .module
        .get_function("vp_waitgroup_add")
        .ok_or_else(|| "vp_waitgroup_add not declared".to_string())?;

    state.ir_builder.build_call(state.builder, add_func, &[wg_val.into(), n_val.into()], "wg_add");
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate done(wg) - signal done on wait group
pub fn generate_waitgroup_done<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "done() takes 1 argument (wg), got {}",
            args.len()
        ));
    }

    let wg_val = generate_expr(state, &args[0])?;
    let done_func = state
        .module
        .get_function("vp_waitgroup_done")
        .ok_or_else(|| "vp_waitgroup_done not declared".to_string())?;

    state.ir_builder.build_call(state.builder, done_func, &[wg_val.into()], "wg_done");
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate wait(wg) - wait on wait group
pub fn generate_waitgroup_wait<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    if args.len() != 1 {
        return crate::codegen::codegen_error(format!(
            "wait() takes 1 argument (wg), got {}",
            args.len()
        ));
    }

    let wg_val = generate_expr(state, &args[0])?;
    let wait_func = state
        .module
        .get_function("vp_waitgroup_wait")
        .ok_or_else(|| "vp_waitgroup_wait not declared".to_string())?;

    state.ir_builder.build_call(state.builder, wait_func, &[wg_val.into()], "wg_wait");
    Ok(state.ir_builder.i64_const(0).into())
}

/// Generate await expression - suspend until future is ready
pub fn generate_await<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    future: &Expr,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    // For now, generate a simple call to vp_future_await
    // A full implementation would transform the async function into a state machine
    let future_val = generate_expr(state, future)?;
    let future_ptr = if future_val.is_pointer_value() {
        future_val.into_pointer_value()
    } else if future_val.is_int_value() {
        state
            .builder
            .build_int_to_ptr(
                future_val.into_int_value(),
                state.context.ptr_type(inkwell::AddressSpace::default()),
                "future_ptr",
            )
            .map_err(|e| format!("Failed to cast future to pointer: {:?}", e))?
    } else {
        return crate::codegen::codegen_error("await target must be a Future".to_string());
    };

    let await_func = state
        .module
        .get_function("vp_future_await")
        .ok_or_else(|| "vp_future_await not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        await_func,
        &[future_ptr.into()],
        "await_result",
    );

    Ok(result.unwrap())
}
