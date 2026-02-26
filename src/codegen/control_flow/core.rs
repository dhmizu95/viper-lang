use crate::ast::Expr;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::LoopContext;

/// Generate a return statement
pub fn generate_return<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    value: &Option<Expr>,
) -> Result<(), String> {
    if let Some(val) = value {
        // If it's an explicit None return, and the function returns void,
        // treat it as a void return.
        if matches!(val, Expr::None(_)) {
            let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            if func.get_type().get_return_type().is_none() {
                state.ir_builder.build_return(state.builder, None);
                return Ok(());
            }
        }
        
        let v = crate::codegen::expressions::generate_expr(state, val)?;
        state.ir_builder.build_return(state.builder, Some(&v));
    } else {
        state.ir_builder.build_return(state.builder, None);
    }
    Ok(())
}

/// Generate a break statement
pub fn generate_break<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &crate::codegen::builder::IRBuilder<'ctx>,
    loop_stack: &[LoopContext<'ctx>],
) -> Result<(), String> {
    if let Some(loop_ctx) = loop_stack.last() {
        ir_builder.build_branch(builder, loop_ctx.break_block);
        Ok(())
    } else {
        Err("break statement outside of loop".to_string())
    }
}

/// Generate a continue statement
pub fn generate_continue<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &crate::codegen::builder::IRBuilder<'ctx>,
    loop_stack: &[LoopContext<'ctx>],
) -> Result<(), String> {
    if let Some(loop_ctx) = loop_stack.last() {
        ir_builder.build_branch(builder, loop_ctx.continue_block);
        Ok(())
    } else {
        Err("continue statement outside of loop".to_string())
    }
}
