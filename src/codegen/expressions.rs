//! Expression code generation for Viper

use crate::ast::{BinOp, Expr, UnaryOp};
use inkwell::values::BasicValueEnum;

use crate::codegen::state::CodeGenState;
use crate::codegen::variables::VarType;

/// Generate code for an expression
pub fn generate_expr<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    expr: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    match expr {
        Expr::Int(n, _) => Ok(state.ir_builder.i64_const(*n).into()),
        Expr::Float(n, _) => Ok(state.ir_builder.f64_const(*n).into()),
        Expr::Bool(b, _) => Ok(state.ir_builder.bool_const(*b).into()),
        Expr::None(_) => Ok(state.ir_builder.i64_const(0).into()),
        Expr::Str(s, _) => Ok(state.ir_builder.string_const(state.module, s).into()),
        Expr::Ident(name, _span) => {
            // First check if it's a global constant
            if let Some(global) = state.global_constants.get(name) {
                // Load the global constant value directly
                let global_ptr = global.as_pointer_value();
                // Build load without explicit type - let LLVM infer it
                let loaded = state.builder.build_load(global_ptr.get_type(), global_ptr, name)
                    .expect("load global constant");
                return Ok(loaded);
            }
            
            // Otherwise check local variables
            if let Some(var_info) = state.variables.get(name) {
                match var_info.var_type {
                    VarType::Float => {
                        let f64_type = state.context.f64_type();
                        Ok(state.builder.build_load(f64_type, var_info.alloca, name).expect("load"))
                    }
                    VarType::Pointer => {
                        let ptr_type = state.context.ptr_type(inkwell::AddressSpace::default());
                        Ok(state.builder.build_load(ptr_type, var_info.alloca, name).expect("load"))
                    }
                    VarType::Int => {
                        let i64_type = state.context.i64_type();
                        Ok(state.builder.build_load(i64_type, var_info.alloca, name).expect("load"))
                    }
                }
            } else {
                Err(format!("Undefined variable: {}", name))
            }
        }
        Expr::List { elements, span: _ } => {
            generate_list(state, elements)
        }
        Expr::Tuple { elements, span: _ } => {
            if elements.is_empty() {
                Ok(state.ir_builder.i64_const(0).into())
            } else {
                generate_expr(state, &elements[0])
            }
        }
        Expr::Dict { pairs: _, span: _ } => {
            Err("Dictionary literals not yet implemented in Phase 2".to_string())
        }
        Expr::Index { obj, index, span: _ } => {
            generate_index(state, obj, index)
        }
        Expr::BinOp { left, op, right, .. } => {
            generate_binop(state, left, op, right)
        }
        Expr::UnaryOp { op, operand, .. } => {
            generate_unary(state, op, operand)
        }
        Expr::Conditional { condition, then_expr, else_expr, span: _ } => {
            generate_conditional(state, condition, then_expr, else_expr)
        }
        Expr::Call { func, args, span } => {
            generate_call(state, func, args, *span)
        }
        Expr::Attribute { obj, attr: _, span: _ } => {
            generate_expr(state, obj)
        }
        _ => Err(format!("Unsupported expression: {:?}", expr)),
    }
}

/// Generate list creation
fn generate_list<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    elements: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let list_func = state.module
        .get_function("vp_list_create")
        .ok_or_else(|| "vp_list_create not declared".to_string())?;

    let list_val = state.ir_builder
        .build_call(state.builder, list_func, &[], "new_list")
        .unwrap();

    let append_func = state.module
        .get_function("vp_list_append")
        .ok_or_else(|| "vp_list_append not declared".to_string())?;

    for (i, elem) in elements.iter().enumerate() {
        let elem_val = generate_expr(state, elem)?;
        let _ = state.ir_builder.build_call(
            state.builder,
            append_func,
            &[list_val.into(), elem_val.into()],
            &format!("list_append_{}", i),
        );
    }

    Ok(list_val)
}

/// Generate index access
fn generate_index<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    index: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let list_val = generate_expr(state, obj)?;
    let index_val = generate_expr(state, index)?.into_int_value();

    let list_get = state.module
        .get_function("vp_list_get")
        .ok_or_else(|| "vp_list_get not declared".to_string())?;

    let result = state.ir_builder
        .build_call(state.builder, list_get, &[list_val.into(), index_val.into()], "list_get")
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}

/// Generate binary operation
fn generate_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    if matches!(op, BinOp::And | BinOp::Or) {
        return generate_logical_op(state, left, op, right);
    }

    if matches!(op, BinOp::In | BinOp::NotIn) {
        return generate_membership_op(state, left, op, right);
    }

    // Handle string concatenation with + operator
    if *op == BinOp::Add {
        let lhs_val = generate_expr(state, left)?;
        let rhs_val = generate_expr(state, right)?;
        
        // Check if both operands are strings (pointer types)
        if lhs_val.is_pointer_value() && rhs_val.is_pointer_value() {
            return generate_str_concat(state, lhs_val, rhs_val);
        }
    }

    let lhs_val = generate_expr(state, left)?;
    let rhs_val = generate_expr(state, right)?;

    // Fix #2: Reject pointer values in binary operations
    if lhs_val.is_pointer_value() || rhs_val.is_pointer_value() {
        return Err("Binary operators cannot be applied to pointer values (lists)".to_string());
    }

    // Fix #3: Auto-convert int to float when one operand is float
    if lhs_val.is_float_value() && !rhs_val.is_float_value() {
        // Convert rhs (int) to float
        let rhs_int = rhs_val.into_int_value();
        let rhs_float = state.builder.build_signed_int_to_float(rhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return generate_float_binop(state.builder, lhs_val, rhs_float.into(), op);
    } else if !lhs_val.is_float_value() && rhs_val.is_float_value() {
        // Convert lhs (int) to float
        let lhs_int = lhs_val.into_int_value();
        let lhs_float = state.builder.build_signed_int_to_float(lhs_int, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion");
        return generate_float_binop(state.builder, lhs_float.into(), rhs_val, op);
    } else if lhs_val.is_float_value() {
        return generate_float_binop(state.builder, lhs_val, rhs_val, op);
    } else {
        return generate_int_binop(state, lhs_val, rhs_val, op);
    }
}

/// Generate string concatenation
fn generate_str_concat<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let str_concat = state.module
        .get_function("vp_str_concat")
        .ok_or_else(|| "vp_str_concat not declared".to_string())?;

    let result = state.ir_builder
        .build_call(state.builder, str_concat, &[lhs.into(), rhs.into()], "str_concat")
        .ok_or_else(|| "build call failed".to_string())?;

    Ok(result)
}

/// Generate logical AND/OR with short-circuiting
fn generate_logical_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs_val = generate_expr(state, left)?.into_int_value();

    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let then_block = state.context.append_basic_block(func, "logic_then");
    let end_block = state.context.append_basic_block(func, "logic_end");

    let is_and = *op == BinOp::And;

    state.builder.build_conditional_branch(
        lhs_val,
        if is_and { then_block } else { end_block },
        if is_and { end_block } else { then_block },
    ).expect("branch");

    state.builder.position_at_end(then_block);
    let rhs_val = generate_expr(state, right)?.into_int_value();
    state.builder.build_unconditional_branch(end_block).expect("branch");
    let then_block_end = state.builder.get_insert_block().unwrap();

    state.builder.position_at_end(end_block);
    let phi = state.builder.build_phi(state.context.bool_type(), "logic_result").expect("phi");

    let cond_block = state.builder.get_insert_block().unwrap().get_previous_basic_block().unwrap();
    if is_and {
        phi.add_incoming(&[(&lhs_val, cond_block), (&rhs_val, then_block_end)]);
    } else {
        phi.add_incoming(&[(&lhs_val, cond_block), (&rhs_val, then_block_end)]);
    }

    Ok(phi.as_basic_value())
}

/// Generate membership IN/NOT IN operators
fn generate_membership_op<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    left: &Expr,
    op: &BinOp,
    right: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let value_val = generate_expr(state, left)?;
    let list_val = generate_expr(state, right)?;

    let list_contains = state.module
        .get_function("vp_list_contains")
        .ok_or_else(|| "vp_list_contains not declared".to_string())?;

    let result = state.ir_builder.build_call(
        state.builder,
        list_contains,
        &[list_val.into(), value_val.into()],
        if matches!(op, BinOp::In) { "list_contains" } else { "not_in_contains" },
    );
    let contains_val: BasicValueEnum = result.unwrap_or(state.ir_builder.i64_const(0).into());

    if matches!(op, BinOp::NotIn) {
        Ok(state.builder.build_not(contains_val.into_int_value(), "not_in_result").expect("not").into())
    } else {
        Ok(contains_val)
    }
}

/// Generate float binary operation
fn generate_float_binop<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_float_value();
    let rhs = rhs.into_float_value();
    
    match op {
        BinOp::Add => Ok(builder.build_float_add(lhs, rhs, "fadd").expect("fadd").into()),
        BinOp::Sub => Ok(builder.build_float_sub(lhs, rhs, "fsub").expect("fsub").into()),
        BinOp::Mul => Ok(builder.build_float_mul(lhs, rhs, "fmul").expect("fmul").into()),
        BinOp::Div => Ok(builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv").into()),
        BinOp::Eq => Ok(builder.build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "feq").expect("feq").into()),
        BinOp::NotEq => Ok(builder.build_float_compare(inkwell::FloatPredicate::ONE, lhs, rhs, "fne").expect("fne").into()),
        BinOp::Lt => Ok(builder.build_float_compare(inkwell::FloatPredicate::OLT, lhs, rhs, "flt").expect("flt").into()),
        BinOp::Gt => Ok(builder.build_float_compare(inkwell::FloatPredicate::OGT, lhs, rhs, "fgt").expect("fgt").into()),
        BinOp::LtEq => Ok(builder.build_float_compare(inkwell::FloatPredicate::OLE, lhs, rhs, "fle").expect("fle").into()),
        BinOp::GtEq => Ok(builder.build_float_compare(inkwell::FloatPredicate::OGE, lhs, rhs, "fge").expect("fge").into()),
        BinOp::Is => Ok(builder.build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_is").expect("f_is").into()),
        BinOp::IsNot => {
            let eq = builder.build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_isnot").expect("f_isnot");
            Ok(builder.build_not(eq, "f_isnot_result").expect("not").into())
        }
        BinOp::FloorDiv => {
            let div = builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv");
            Ok(div.into())
        }
        BinOp::Pow => Err("pow for floats not implemented".to_string()),
        BinOp::In | BinOp::NotIn => Err("Membership operators not supported for float types".to_string()),
        _ => Err(format!("Unsupported float operator: {:?}", op)),
    }
}

/// Generate integer binary operation
fn generate_int_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_int_value();
    let rhs = rhs.into_int_value();
    
    match op {
        BinOp::Add => Ok(state.ir_builder.build_add(state.builder, lhs, rhs, "add").into()),
        BinOp::Sub => Ok(state.ir_builder.build_sub(state.builder, lhs, rhs, "sub").into()),
        BinOp::Mul => Ok(state.ir_builder.build_mul(state.builder, lhs, rhs, "mul").into()),
        BinOp::Div => Ok(state.ir_builder.build_div(state.builder, lhs, rhs, "div").into()),
        BinOp::Eq => Ok(state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "eq").into()),
        BinOp::NotEq => {
            let eq = state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "eq");
            Ok(state.builder.build_not(eq, "neq").expect("not").into())
        }
        BinOp::Lt => Ok(state.ir_builder.build_icmp_lt(state.builder, lhs, rhs, "lt").into()),
        BinOp::Gt => Ok(state.builder.build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gt").expect("gt").into()),
        BinOp::LtEq => Ok(state.builder.build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "lte").expect("lte").into()),
        BinOp::GtEq => Ok(state.builder.build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gte").expect("gte").into()),
        BinOp::Is => Ok(state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "is_cmp").into()),
        BinOp::IsNot => {
            let eq = state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "isnot_cmp");
            Ok(state.builder.build_not(eq, "isnot_result").expect("not").into())
        }
        BinOp::Mod => Ok(state.builder.build_int_signed_rem(lhs, rhs, "mod").expect("mod").into()),
        BinOp::FloorDiv => Ok(state.ir_builder.build_div(state.builder, lhs, rhs, "floordiv").into()),
        BinOp::BitAnd => Ok(state.builder.build_and(lhs, rhs, "bitand").expect("bitand").into()),
        BinOp::BitOr => Ok(state.builder.build_or(lhs, rhs, "bitor").expect("bitor").into()),
        BinOp::BitXor => Ok(state.builder.build_xor(lhs, rhs, "bitxor").expect("bitxor").into()),
        BinOp::LShift => Ok(state.builder.build_left_shift(lhs, rhs, "lshift").expect("lshift").into()),
        BinOp::RShift => Ok(state.builder.build_right_shift(lhs, rhs, false, "rshift").expect("rshift").into()),
        BinOp::Pow => Err("pow for ints not implemented".to_string()),
        _ => Err(format!("Unsupported int operator: {:?}", op)),
    }
}

/// Generate unary operation
fn generate_unary<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let val = generate_expr(state, operand)?;
    
    if val.is_float_value() {
        let float_val = val.into_float_value();
        match op {
            UnaryOp::Neg => Ok(state.builder.build_float_neg(float_val, "fneg").expect("fneg").into()),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Not | UnaryOp::Invert => Err(format!("Unary operator {:?} not supported for float types", op)),
        }
    } else {
        let int_val = val.into_int_value();
        match op {
            UnaryOp::Neg => Ok(state.builder.build_int_neg(int_val, "neg").expect("neg").into()),
            UnaryOp::Not => Ok(state.builder.build_not(int_val, "not").expect("not").into()),
            UnaryOp::Pos => Ok(val),
            UnaryOp::Invert => Ok(state.builder.build_xor(int_val, state.context.i64_type().const_all_ones(), "invert").expect("invert").into()),
        }
    }
}

/// Generate ternary conditional expression
fn generate_conditional<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    condition: &Expr,
    then_expr: &Expr,
    else_expr: &Expr,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state.builder.get_insert_block().unwrap().get_parent().unwrap();
    let cond_val = generate_expr(state, condition)?.into_int_value();

    let then_block = state.context.append_basic_block(func, "ternary_then");
    let else_block = state.context.append_basic_block(func, "ternary_else");
    let merge_block = state.context.append_basic_block(func, "ternary_end");

    let cond_i1 = if cond_val.get_type().get_bit_width() == 1 {
        cond_val
    } else {
        state.builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                cond_val,
                state.context.i64_type().const_zero(),
                "ternary_cond",
            )
            .expect("ternary_cond")
    };

    state.ir_builder.build_cond_branch(state.builder, cond_i1, then_block, else_block);

    state.builder.position_at_end(then_block);
    let then_val = generate_expr(state, then_expr)?;
    let then_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(else_block);
    let else_val = generate_expr(state, else_expr)?;
    let else_block_end = state.builder.get_insert_block().unwrap();
    state.ir_builder.build_branch(state.builder, merge_block);

    state.builder.position_at_end(merge_block);
    let phi = state.builder.build_phi(then_val.get_type(), "ternary_result").expect("phi");
    phi.add_incoming(&[(&then_val, then_block_end), (&else_val, else_block_end)]);

    Ok(phi.as_basic_value())
}

/// Generate function/method call
fn generate_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    func: &Expr,
    args: &[Expr],
    _span: crate::utils::Span,
) -> Result<BasicValueEnum<'ctx>, String> {
    if let Expr::Attribute { obj, attr, .. } = func {
        return generate_method_call(state, obj, attr, args);
    }

    if let Expr::Ident(name, _) = func {
        if name == "print" {
            return generate_print_call(state, args);
        }

        if name == "len" {
            return generate_len_call(state, args);
        }

        // Math builtins
        if name == "sqrt" || name == "abs" || name == "ln" || name == "floor" {
            return generate_math_builtin(state, name, args);
        }

        if let Some(&func_val) = state.functions.get(name) {
            let arg_values: Vec<_> = args
                .iter()
                .map(|a| generate_expr(state, a).map(|v| v.into()))
                .collect::<Result<_, _>>()?;

            let result = state.ir_builder.build_call(state.builder, func_val, &arg_values, "call");
            return Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()));
        }

        return Err(format!("Unknown function: {}", name));
    }

    Err(format!("Unknown function: {:?}", func))
}

/// Generate print call
fn generate_print_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.is_empty() {
        return Ok(state.ir_builder.i64_const(0).into());
    }

    let val = generate_expr(state, &args[0])?;
    
    if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 64 {
        let print_func = state.module.get_function("vp_print_i64").ok_or_else(|| "vp_print_i64 not declared".to_string())?;
        state.builder.build_call(print_func, &[val.into()], "print_i64").expect("vp_print_i64");

        let newline_func = state.module.get_function("vp_print_newline").ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state.builder.build_call(newline_func, &[], "print_newline").expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else if val.is_float_value() {
        let print_func = state.module.get_function("vp_print_f64").ok_or_else(|| "vp_print_f64 not declared".to_string())?;
        state.builder.build_call(print_func, &[val.into()], "print_f64").expect("vp_print_f64");

        let newline_func = state.module.get_function("vp_print_newline").ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state.builder.build_call(newline_func, &[], "print_newline").expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else if val.is_int_value() && val.get_type().into_int_type().get_bit_width() == 1 {
        let print_func = state.module.get_function("vp_print_bool").ok_or_else(|| "vp_print_bool not declared".to_string())?;
        state.builder.build_call(print_func, &[val.into()], "print_bool").expect("vp_print_bool");

        let newline_func = state.module.get_function("vp_print_newline").ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state.builder.build_call(newline_func, &[], "print_newline").expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else if val.is_pointer_value() {
        let print_func = state.module.get_function("vp_print_str").ok_or_else(|| "vp_print_str not declared".to_string())?;
        state.builder.build_call(print_func, &[val.into()], "print_str").expect("vp_print_str");

        let newline_func = state.module.get_function("vp_print_newline").ok_or_else(|| "vp_print_newline not declared".to_string())?;
        state.builder.build_call(newline_func, &[], "print_newline").expect("vp_print_newline");

        return Ok(state.ir_builder.i64_const(0).into());
    } else {
        return Err(format!("print() does not support type {:?}", val.get_type()));
    }
}

/// Generate len() call
fn generate_len_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("len() takes exactly 1 argument, got {}", args.len()));
    }

    let obj_val = generate_expr(state, &args[0])?;
    let list_len = state.module.get_function("vp_list_len").ok_or_else(|| "vp_list_len not declared".to_string())?;
    let result = state.ir_builder.build_call(state.builder, list_len, &[obj_val.into()], "list_len");
    Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
}

/// Generate math builtin function calls (sqrt, abs, ln, floor)
fn generate_math_builtin<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    if args.len() != 1 {
        return Err(format!("{}() takes exactly 1 argument, got {}", name, args.len()));
    }

    let arg_val = generate_expr(state, &args[0])?;
    
    // Convert to float if necessary
    let arg_float = if arg_val.is_float_value() {
        arg_val.into_float_value()
    } else {
        let int_val = arg_val.into_int_value();
        state.builder.build_signed_int_to_float(int_val, state.context.f64_type(), "int_to_float")
            .expect("int to float conversion")
    };

    let func_name = match name {
        "sqrt" => "vp_math_sqrt",
        "abs" => "vp_math_abs",
        "ln" => "vp_math_ln",
        "floor" => "vp_math_floor",
        _ => return Err(format!("Unknown math builtin: {}", name)),
    };

    let math_func = state.module.get_function(func_name)
        .ok_or_else(|| format!("{} not declared", func_name))?;

    let result = state.ir_builder.build_call(state.builder, math_func, &[arg_float.into()], "math_result");
    Ok(result.unwrap_or(state.ir_builder.f64_const(0.0).into()))
}

/// Generate method call
fn generate_method_call<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    obj: &Expr,
    method_name: &str,
    args: &[Expr],
) -> Result<BasicValueEnum<'ctx>, String> {
    let obj_val = generate_expr(state, obj)?;

    match method_name {
        "append" => {
            if args.len() != 1 {
                return Err(format!("append() takes exactly 1 argument, got {}", args.len()));
            }
            let val = generate_expr(state, &args[0])?.into_int_value();
            let list_append = state.module.get_function("vp_list_append").ok_or_else(|| "vp_list_append not declared".to_string())?;
            state.ir_builder.build_call(state.builder, list_append, &[obj_val.into(), val.into()], "list_append");
            Ok(state.ir_builder.i64_const(0).into())
        }
        "insert" => {
            if args.len() != 2 {
                return Err(format!("insert() takes exactly 2 arguments, got {}", args.len()));
            }
            let index = generate_expr(state, &args[0])?.into_int_value();
            let val = generate_expr(state, &args[1])?.into_int_value();
            let list_insert = state.module.get_function("vp_list_insert").ok_or_else(|| "vp_list_insert not declared".to_string())?;
            state.ir_builder.build_call(state.builder, list_insert, &[obj_val.into(), index.into(), val.into()], "list_insert");
            Ok(state.ir_builder.i64_const(0).into())
        }
        "remove" => {
            if args.len() != 1 {
                return Err(format!("remove() takes exactly 1 argument, got {}", args.len()));
            }
            let index = generate_expr(state, &args[0])?.into_int_value();
            let list_remove = state.module.get_function("vp_list_remove").ok_or_else(|| "vp_list_remove not declared".to_string())?;
            let result = state.ir_builder.build_call(state.builder, list_remove, &[obj_val.into(), index.into()], "list_remove");
            Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
        }
        "pop" => {
            if !args.is_empty() {
                return Err(format!("pop() takes no arguments, got {}", args.len()));
            }
            let list_pop = state.module.get_function("vp_list_pop").ok_or_else(|| "vp_list_pop not declared".to_string())?;
            let result = state.ir_builder.build_call(state.builder, list_pop, &[obj_val.into()], "list_pop");
            Ok(result.unwrap_or(state.ir_builder.i64_const(0).into()))
        }
        "clear" => {
            if !args.is_empty() {
                return Err(format!("clear() takes no arguments, got {}", args.len()));
            }
            let list_clear = state.module.get_function("vp_list_clear").ok_or_else(|| "vp_list_clear not declared".to_string())?;
            state.ir_builder.build_call(state.builder, list_clear, &[obj_val.into()], "list_clear");
            Ok(state.ir_builder.i64_const(0).into())
        }
        "len" => Err("len() is a builtin function, not a method".to_string()),
        _ => Err(format!("Unknown method: {}", method_name)),
    }
}
