use crate::ast::BinOp;
use inkwell::values::BasicValueEnum;

/// Generate pointer binary operation (identity comparison)
pub fn generate_pointer_binop<'ctx>(
    builder: &inkwell::builder::Builder<'ctx>,
    context: &'ctx inkwell::context::Context,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
    op: &BinOp,
) -> crate::codegen::Result<BasicValueEnum<'ctx>> {
    let lhs = lhs.into_pointer_value();
    let rhs = rhs.into_pointer_value();

    // Convert pointers to i64 for comparison (works on 64-bit systems)
    let intptr_type = context.i64_type();

    let lhs_int = builder
        .build_ptr_to_int(lhs, intptr_type, "lhs_int")
        .expect("Failed to convert lhs pointer to int");
    let rhs_int = builder
        .build_ptr_to_int(rhs, intptr_type, "rhs_int")
        .expect("Failed to convert rhs pointer to int");

    match op {
        BinOp::Eq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::EQ, lhs_int, rhs_int, "ptr_eq")
            .expect("ptr_eq")
            .into()),
        BinOp::NotEq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::NE, lhs_int, rhs_int, "ptr_neq")
            .expect("ptr_neq")
            .into()),
        BinOp::Is => Ok(builder
            .build_int_compare(inkwell::IntPredicate::EQ, lhs_int, rhs_int, "ptr_is")
            .expect("ptr_is")
            .into()),
        BinOp::IsNot => Ok(builder
            .build_int_compare(inkwell::IntPredicate::NE, lhs_int, rhs_int, "ptr_isnot")
            .expect("ptr_isnot")
            .into()),
        BinOp::Lt => Ok(builder
            .build_int_compare(inkwell::IntPredicate::ULT, lhs_int, rhs_int, "ptr_lt")
            .expect("ptr_lt")
            .into()),
        BinOp::Gt => Ok(builder
            .build_int_compare(inkwell::IntPredicate::UGT, lhs_int, rhs_int, "ptr_gt")
            .expect("ptr_gt")
            .into()),
        BinOp::LtEq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::ULE, lhs_int, rhs_int, "ptr_lte")
            .expect("ptr_lte")
            .into()),
        BinOp::GtEq => Ok(builder
            .build_int_compare(inkwell::IntPredicate::UGE, lhs_int, rhs_int, "ptr_gte")
            .expect("ptr_gte")
            .into()),
        _ => crate::codegen::codegen_error(format!("Unsupported pointer operator: {:?}", op)),
    }
}
