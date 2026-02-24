//! Statement code generation for Viper

use crate::ast::{BinOp, Expr, Stmt};
use inkwell::context::Context;
use inkwell::values::FunctionValue;
use std::collections::HashMap;

use crate::codegen::builder::IRBuilder;
use crate::codegen::state::CodeGenState;
use crate::codegen::variables::{VarInfo, VarType, LoopContext};

/// Generate code for a statement
pub fn generate_stmt<'ctx>(
    context: &'ctx Context,
    module: &inkwell::module::Module<'ctx>,
    builder: &inkwell::builder::Builder<'ctx>,
    ir_builder: &IRBuilder<'ctx>,
    variables: &mut HashMap<String, VarInfo<'ctx>>,
    functions: &HashMap<String, FunctionValue<'ctx>>,
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
        loop_stack,
    );
    
    match stmt {
        Stmt::Expr(expr) => {
            crate::codegen::expressions::generate_expr(&mut state, expr)?;
        }
        Stmt::Assign { target, value, .. } => {
            generate_assign(&mut state, target, value)?;
        }
        Stmt::AugAssign { target, op, value, .. } => {
            generate_aug_assign(&mut state, target, op, value)?;
        }
        Stmt::Declare { name, value, .. } => {
            generate_declare(&mut state, name, value)?;
        }
        Stmt::Return { value, .. } => {
            return crate::codegen::control_flow::generate_return(&mut state, value);
        }
        Stmt::If { condition, body, elif_blocks, else_body, .. } => {
            return crate::codegen::control_flow::generate_if(
                &mut state,
                condition,
                body,
                elif_blocks,
                else_body,
            );
        }
        Stmt::While { condition, body, .. } => {
            return crate::codegen::control_flow::generate_while(
                &mut state,
                condition,
                body,
            );
        }
        Stmt::For { target, iter, body, .. } => {
            return crate::codegen::control_flow::generate_for(
                &mut state,
                target,
                iter,
                body,
            );
        }
        Stmt::Function { .. } => {
            // Already handled in first pass
        }
        Stmt::Break(_) => {
            return crate::codegen::control_flow::generate_break(
                state.builder, state.ir_builder, state.loop_stack
            );
        }
        Stmt::Continue(_) => {
            return crate::codegen::control_flow::generate_continue(
                state.builder, state.ir_builder, state.loop_stack
            );
        }
        Stmt::Pass(_) => {
            // No-op
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
        
        if let Some(var_info) = state.variables.get(name) {
            state.builder.build_store(var_info.alloca, val).expect("store");
        } else {
            let ty = val.get_type();
            let alloca = state.builder.build_alloca(ty, name).expect("alloca");
            state.builder.build_store(alloca, val).expect("store");
            let var_type = if val.is_float_value() {
                VarType::Float
            } else if val.is_pointer_value() {
                VarType::Pointer
            } else {
                VarType::Int
            };
            state.variables.insert(name.clone(), VarInfo { alloca, var_type });
        }
    } else if let Expr::Index { obj, index, .. } = target {
        let list_val = crate::codegen::expressions::generate_expr(state, obj)?;
        let index_val = crate::codegen::expressions::generate_expr(state, index)?.into_int_value();
        let value_val = crate::codegen::expressions::generate_expr(state, value)?.into_int_value();

        let list_set = state.module.get_function("vp_list_set").ok_or_else(|| "vp_list_set not declared".to_string())?;
        state.ir_builder.build_call(state.builder, list_set, &[list_val.into(), index_val.into(), value_val.into()], "list_set");
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
            let alloca = var_info.alloca;
            let var_type = var_info.var_type;
            
            let current = match var_type {
                VarType::Float => {
                    let f64_type = state.context.f64_type();
                    state.builder.build_load(f64_type, alloca, name).expect("load")
                }
                VarType::Int => {
                    let i64_type = state.context.i64_type();
                    state.builder.build_load(i64_type, alloca, name).expect("load")
                }
                VarType::Pointer => {
                    return Err(format!("Cannot perform augmented assignment on pointer variable '{}'", name));
                }
            };

            let new_val = crate::codegen::expressions::generate_expr(state, value)?;

            let result: inkwell::values::BasicValueEnum<'ctx> = if var_type == VarType::Float {
                let lhs = current.into_float_value();
                let rhs = new_val.into_float_value();
                match op {
                    BinOp::Add => state.builder.build_float_add(lhs, rhs, "fadd").expect("fadd"),
                    BinOp::Sub => state.builder.build_float_sub(lhs, rhs, "fsub").expect("fsub"),
                    BinOp::Mul => state.builder.build_float_mul(lhs, rhs, "fmul").expect("fmul"),
                    BinOp::Div => state.builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv"),
                    _ => return Err(format!("Unsupported augmented assignment operator for float: {:?}", op)),
                }.into()
            } else {
                let lhs = current.into_int_value();
                let rhs = new_val.into_int_value();
                match op {
                    BinOp::Add => state.ir_builder.build_add(state.builder, lhs, rhs, "add"),
                    BinOp::Sub => state.ir_builder.build_sub(state.builder, lhs, rhs, "sub"),
                    BinOp::Mul => state.ir_builder.build_mul(state.builder, lhs, rhs, "mul"),
                    BinOp::Div => state.ir_builder.build_div(state.builder, lhs, rhs, "div"),
                    BinOp::Mod => state.builder.build_int_signed_rem(lhs, rhs, "mod").expect("mod"),
                    BinOp::FloorDiv => state.ir_builder.build_div(state.builder, lhs, rhs, "floordiv"),
                    _ => return Err(format!("Unsupported augmented assignment operator for int: {:?}", op)),
                }.into()
            };

            state.builder.build_store(alloca, result).expect("store");
        } else {
            return Err(format!("Undefined variable in augmented assignment: {}", name));
        }
    }
    Ok(())
}

/// Generate variable declaration
fn generate_declare<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    value: &Option<Expr>,
) -> Result<(), String> {
    if let Some(val) = value {
        let val = crate::codegen::expressions::generate_expr(state, val)?;
        let ty = val.get_type();
        let alloca = state.builder.build_alloca(ty, name).expect("alloca");
        state.builder.build_store(alloca, val).expect("store");
        let var_type = if val.is_float_value() {
            VarType::Float
        } else if val.is_pointer_value() {
            VarType::Pointer
        } else {
            VarType::Int
        };
        state.variables.insert(name.to_string(), VarInfo { alloca, var_type });
    }
    Ok(())
}
