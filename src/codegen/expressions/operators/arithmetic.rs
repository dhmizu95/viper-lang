use crate::ast::BinOp;
use crate::ast::UnaryOp;
use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Generate float binary operation
pub fn generate_float_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let builder = state.builder;
    let lhs = lhs.into_float_value();
    let rhs = rhs.into_float_value();

    match op {
        BinOp::Add => Ok(builder.build_float_add(lhs, rhs, "fadd").expect("fadd").into()),
        BinOp::Sub => Ok(builder.build_float_sub(lhs, rhs, "fsub").expect("fsub").into()),
        BinOp::Mul => Ok(builder.build_float_mul(lhs, rhs, "fmul").expect("fmul").into()),
        BinOp::Div => Ok(builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv").into()),
        BinOp::Eq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "feq")
            .expect("feq")
            .into()),
        BinOp::NotEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::ONE, lhs, rhs, "fne")
            .expect("fne")
            .into()),
        BinOp::Lt => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OLT, lhs, rhs, "flt")
            .expect("flt")
            .into()),
        BinOp::Gt => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OGT, lhs, rhs, "fgt")
            .expect("fgt")
            .into()),
        BinOp::LtEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OLE, lhs, rhs, "fle")
            .expect("fle")
            .into()),
        BinOp::GtEq => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OGE, lhs, rhs, "fge")
            .expect("fge")
            .into()),
        BinOp::Is => Ok(builder
            .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_is")
            .expect("f_is")
            .into()),
        BinOp::IsNot => {
            let eq = builder
                .build_float_compare(inkwell::FloatPredicate::OEQ, lhs, rhs, "f_isnot")
                .expect("f_isnot");
            Ok(builder.build_not(eq, "f_isnot_result").expect("not").into())
        }
        BinOp::FloorDiv => {
            let div = builder.build_float_div(lhs, rhs, "fdiv").expect("fdiv");
            Ok(div.into())
        }
        BinOp::Pow => {
            // Call vp_pow(base, exponent)
            let pow_func = state
                .module
                .get_function("vp_pow")
                .ok_or_else(|| "vp_pow not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(state.builder, pow_func, &[lhs.into(), rhs.into()], "pow")
                .ok_or_else(|| "build call failed".to_string())?;

            Ok(result)
        }
        BinOp::In | BinOp::NotIn => {
            Err("Membership operators not supported for float types".to_string())
        }
        _ => Err(format!("Unsupported float operator: {:?}", op)),
    }
}

/// Generate boolean binary operation
pub fn generate_bool_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lhs = lhs.into_int_value();
    let rhs = rhs.into_int_value();

    match op {
        BinOp::Eq => Ok(state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "bool_eq").into()),
        BinOp::NotEq => {
            let eq = state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "bool_eq");
            Ok(state.builder.build_not(eq, "bool_neq").expect("not").into())
        }
        BinOp::And => Ok(state.builder.build_and(lhs, rhs, "bool_and").expect("and").into()),
        BinOp::Or => Ok(state.builder.build_or(lhs, rhs, "bool_or").expect("or").into()),
        _ => Err(format!("Unsupported boolean operator: {:?}", op)),
    }
}

/// Generate integer binary operation
pub fn generate_int_binop<'ctx>(
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
        BinOp::Gt => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SGT, lhs, rhs, "gt")
            .expect("gt")
            .into()),
        BinOp::LtEq => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SLE, lhs, rhs, "lte")
            .expect("lte")
            .into()),
        BinOp::GtEq => Ok(state
            .builder
            .build_int_compare(inkwell::IntPredicate::SGE, lhs, rhs, "gte")
            .expect("gte")
            .into()),
        BinOp::Is => Ok(state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "is_cmp").into()),
        BinOp::IsNot => {
            let eq = state.ir_builder.build_icmp_eq(state.builder, lhs, rhs, "isnot_cmp");
            Ok(state.builder.build_not(eq, "isnot_result").expect("not").into())
        }
        BinOp::Mod => Ok(state.builder.build_int_signed_rem(lhs, rhs, "mod").expect("mod").into()),
        BinOp::FloorDiv => {
            Ok(state.ir_builder.build_div(state.builder, lhs, rhs, "floordiv").into())
        }
        BinOp::BitAnd => Ok(state.builder.build_and(lhs, rhs, "bitand").expect("bitand").into()),
        BinOp::BitOr => Ok(state.builder.build_or(lhs, rhs, "bitor").expect("bitor").into()),
        BinOp::BitXor => Ok(state.builder.build_xor(lhs, rhs, "bitxor").expect("bitxor").into()),
        BinOp::LShift => {
            Ok(state.builder.build_left_shift(lhs, rhs, "lshift").expect("lshift").into())
        }
        BinOp::RShift => {
            Ok(state.builder.build_right_shift(lhs, rhs, false, "rshift").expect("rshift").into())
        }
        BinOp::Pow => {
            // Call vp_pow_i64(base, exponent)
            let pow_func = state
                .module
                .get_function("vp_pow_i64")
                .ok_or_else(|| "vp_pow_i64 not declared".to_string())?;

            let result = state
                .ir_builder
                .build_call(state.builder, pow_func, &[lhs.into(), rhs.into()], "pow")
                .ok_or_else(|| "build call failed".to_string())?;

            Ok(result.into())
        }
        _ => Err(format!("Unsupported int operator: {:?}", op)),
    }
}

/// Generate tagged integer binary operation
pub fn generate_tagged_int_binop<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> Result<BasicValueEnum<'ctx>, String> {
    match op {
        BinOp::Add => crate::codegen::runtime::tagged_int::generate_tagged_int_add(state, lhs, rhs),
        BinOp::Sub => crate::codegen::runtime::tagged_int::generate_tagged_int_sub(state, lhs, rhs),
        BinOp::Mul => crate::codegen::runtime::tagged_int::generate_tagged_int_mul(state, lhs, rhs),
        BinOp::Div | BinOp::FloorDiv => crate::codegen::runtime::tagged_int::generate_tagged_int_div(state, lhs, rhs),
        BinOp::Mod => crate::codegen::runtime::tagged_int::generate_tagged_int_mod(state, lhs, rhs),
        BinOp::Eq => crate::codegen::runtime::tagged_int::generate_tagged_int_eq(state, lhs, rhs),
        BinOp::NotEq => {
            let eq = crate::codegen::runtime::tagged_int::generate_tagged_int_eq(state, lhs, rhs)?;
            Ok(state.builder.build_not(eq.into_int_value(), "neq").expect("not").into())
        }
        BinOp::Lt => crate::codegen::runtime::tagged_int::generate_tagged_int_lt(state, lhs, rhs),
        BinOp::Gt => crate::codegen::runtime::tagged_int::generate_tagged_int_gt(state, lhs, rhs),
        BinOp::LtEq => {
            let gt = crate::codegen::runtime::tagged_int::generate_tagged_int_gt(state, lhs, rhs)?;
            Ok(state.builder.build_not(gt.into_int_value(), "lte").expect("not").into())
        }
        BinOp::GtEq => {
            let lt = crate::codegen::runtime::tagged_int::generate_tagged_int_lt(state, lhs, rhs)?;
            Ok(state.builder.build_not(lt.into_int_value(), "gte").expect("not").into())
        }
        BinOp::Pow => crate::codegen::runtime::tagged_int::generate_tagged_int_pow(state, lhs, rhs),
        _ => Err(format!("Unsupported tagged int operator: {:?}", op)),
    }
}

/// Generate tagged integer unary operation
pub fn generate_tagged_int_unary<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    op: &UnaryOp,
    operand: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    match op {
        UnaryOp::Neg => crate::codegen::runtime::tagged_int::generate_tagged_int_neg(state, operand),
        UnaryOp::Pos => Ok(operand),
        _ => Err(format!("Unsupported tagged int unary operator: {:?}", op)),
    }
}
