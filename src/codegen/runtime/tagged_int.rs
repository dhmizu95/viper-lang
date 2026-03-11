//! Tagged integer code generation with automatic overflow detection
//!
//! Tagged integers use the LSB to distinguish small ints from BigInt:
//! - LSB = 0: Small integer (i63, stored as value << 1)
//! - LSB = 1: BigInt pointer (pointer | 1)

use crate::codegen::state::CodeGenState;
use inkwell::values::BasicValueEnum;

/// Declare tagged integer runtime functions with optimization attributes
pub fn declare_tagged_int_functions<'ctx>(
    context: &'ctx inkwell::context::Context,
    module: &inkwell::module::Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let void_type = context.void_type();

    // TaggedInt operations return TaggedInt (i64)
    let tagged_op_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);

    // Helper to add aggressive optimization attributes to functions
    let add_opt_attrs = |func: inkwell::values::FunctionValue<'ctx>| {
        // alwaysinline: Force inlining for better performance
        func.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context.create_string_attribute("alwaysinline", ""),
        );
        // willreturn: Function always returns (no infinite loops)
        func.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context.create_string_attribute("willreturn", ""),
        );
        //nounwind: Function doesn't throw exceptions
        func.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context.create_string_attribute("nounwind", ""),
        );
        //uwtable: Function has unwind table
        func.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context.create_string_attribute("uwtable", ""),
        );
    };

    // tagged_int_add
    let func = module.add_function("tagged_int_add", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_sub
    let func = module.add_function("tagged_int_sub", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_mul
    let func = module.add_function("tagged_int_mul", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_div
    let func = module.add_function("tagged_int_div", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_mod
    let func = module.add_function("tagged_int_mod", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_pow
    let func = module.add_function("tagged_int_pow", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_lshift
    let func = module.add_function("tagged_int_lshift", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_rshift
    let func = module.add_function("tagged_int_rshift", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_bitand
    let func = module.add_function("tagged_int_bitand", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_bitor
    let func = module.add_function("tagged_int_bitor", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_bitxor
    let func = module.add_function("tagged_int_bitxor", tagged_op_type, None);
    add_opt_attrs(func);

    // tagged_int_neg (unary)
    let tagged_unary_type = i64_type.fn_type(&[i64_type.into()], false);
    let func = module.add_function("tagged_int_neg", tagged_unary_type, None);
    add_opt_attrs(func);

    // Comparison functions return bool (i1)
    let cmp_type = context.bool_type().fn_type(&[i64_type.into(), i64_type.into()], false);

    // tagged_int_eq
    let func = module.add_function("tagged_int_eq", cmp_type, None);
    add_opt_attrs(func);

    // tagged_int_lt
    let func = module.add_function("tagged_int_lt", cmp_type, None);
    add_opt_attrs(func);

    // tagged_int_gt
    let func = module.add_function("tagged_int_gt", cmp_type, None);
    add_opt_attrs(func);

    // tagged_int_cmp returns i64 (-1, 0, 1)
    let cmp_ret_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let func = module.add_function("tagged_int_cmp", cmp_ret_type, None);
    add_opt_attrs(func);

    // Utility functions
    // tagged_int_from_i64
    let func = module.add_function("tagged_int_from_i64", tagged_unary_type, None);
    add_opt_attrs(func);

    // tagged_int_from_str (has side effects - parses string)
    let from_str_type = i64_type.fn_type(&[ptr_type.into()], false);
    let func = module.add_function("tagged_int_from_str", from_str_type, None);
    // Only add alwaysinline and willreturn - this function has memory effects
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("willreturn", ""),
    );

    // tagged_int_to_str returns char* (allocates memory)
    let to_str_type = ptr_type.fn_type(&[i64_type.into()], false);
    let func = module.add_function("tagged_int_to_str", to_str_type, None);
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("willreturn", ""),
    );

    // tagged_int_print (has side effects - I/O)
    let print_type = void_type.fn_type(&[i64_type.into()], false);
    let func = module.add_function("tagged_int_print", print_type, None);
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    // tagged_int_free (has side effects - frees memory)
    let func = module.add_function("tagged_int_free", tagged_unary_type, None);
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    Ok(())
}

/// Generate tagged integer addition with overflow detection
pub fn generate_tagged_int_add<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_add")
        .ok_or_else(|| "tagged_int_add not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_add")
        .expect("tagged_int_add call");

    Ok(result.into())
}

/// Generate tagged integer subtraction with overflow detection
pub fn generate_tagged_int_sub<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_sub")
        .ok_or_else(|| "tagged_int_sub not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_sub")
        .expect("tagged_int_sub call");

    Ok(result.into())
}

/// Generate tagged integer multiplication with overflow detection
pub fn generate_tagged_int_mul<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_mul")
        .ok_or_else(|| "tagged_int_mul not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_mul")
        .expect("tagged_int_mul call");

    Ok(result.into())
}

/// Generate tagged integer exponentiation
pub fn generate_tagged_int_pow<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_pow")
        .ok_or_else(|| "tagged_int_pow not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_pow")
        .expect("tagged_int_pow call");

    Ok(result.into())
}

/// Generate tagged integer left shift
pub fn generate_tagged_int_lshift<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_lshift")
        .ok_or_else(|| "tagged_int_lshift not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_lshift")
        .expect("tagged_int_lshift call");

    Ok(result.into())
}

/// Generate tagged integer right shift
pub fn generate_tagged_int_rshift<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_rshift")
        .ok_or_else(|| "tagged_int_rshift not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_rshift")
        .expect("tagged_int_rshift call");

    Ok(result.into())
}

/// Generate tagged integer bitwise AND
pub fn generate_tagged_int_bitand<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_bitand")
        .ok_or_else(|| "tagged_int_bitand not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_bitand")
        .expect("tagged_int_bitand call");

    Ok(result.into())
}

/// Generate tagged integer bitwise OR
pub fn generate_tagged_int_bitor<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_bitor")
        .ok_or_else(|| "tagged_int_bitor not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_bitor")
        .expect("tagged_int_bitor call");

    Ok(result.into())
}

/// Generate tagged integer bitwise XOR
pub fn generate_tagged_int_bitxor<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_bitxor")
        .ok_or_else(|| "tagged_int_bitxor not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_bitxor")
        .expect("tagged_int_bitxor call");

    Ok(result.into())
}

/// Generate tagged integer division
pub fn generate_tagged_int_div<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_div")
        .ok_or_else(|| "tagged_int_div not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_div")
        .expect("tagged_int_div call");

    Ok(result.into())
}

/// Generate tagged integer modulo
pub fn generate_tagged_int_mod<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_mod")
        .ok_or_else(|| "tagged_int_mod not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_mod")
        .expect("tagged_int_mod call");

    Ok(result.into())
}

/// Generate tagged integer negation
pub fn generate_tagged_int_neg<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    operand: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_neg")
        .ok_or_else(|| "tagged_int_neg not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[operand.into()], "tagged_neg")
        .expect("tagged_int_neg call");

    Ok(result.into())
}

/// Generate tagged integer equality comparison
pub fn generate_tagged_int_eq<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_eq")
        .ok_or_else(|| "tagged_int_eq not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_eq")
        .expect("tagged_int_eq call");

    Ok(result.into())
}

/// Generate tagged integer less-than comparison
pub fn generate_tagged_int_lt<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_lt")
        .ok_or_else(|| "tagged_int_lt not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_lt")
        .expect("tagged_int_lt call");

    Ok(result.into())
}

/// Generate tagged integer greater-than comparison
pub fn generate_tagged_int_gt<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    lhs: BasicValueEnum<'ctx>,
    rhs: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_gt")
        .ok_or_else(|| "tagged_int_gt not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[lhs.into(), rhs.into()], "tagged_gt")
        .expect("tagged_int_gt call");

    Ok(result.into())
}

/// Convert i64 to tagged integer
pub fn generate_tagged_int_from_i64<'ctx>(
    state: &mut CodeGenState<'_, 'ctx>,
    value: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, String> {
    let func = state
        .module
        .get_function("tagged_int_from_i64")
        .ok_or_else(|| "tagged_int_from_i64 not declared".to_string())?;

    let result = state
        .ir_builder
        .build_call(state.builder, func, &[value.into()], "tagged_from_i64")
        .expect("tagged_int_from_i64 call");

    Ok(result.into())
}
