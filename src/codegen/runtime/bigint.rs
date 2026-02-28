//! BigInt runtime function declarations for GMP integration

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare BigInt runtime functions
pub fn declare_bigint_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let bool_type = context.bool_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let void_type = context.void_type();

    // vp_bigint_from_str: Create BigInt from string
    // ViperBigInt* vp_bigint_from_str(const char* str)
    let from_str_fn_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_from_str", from_str_fn_type, None);

    // vp_bigint_from_i64: Create BigInt from i64
    // ViperBigInt* vp_bigint_from_i64(int64_t value)
    let from_i64_fn_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_bigint_from_i64", from_i64_fn_type, None);

    // vp_bigint_from_i64_temp: Create BigInt from i64 for temporary results
    // ViperBigInt* vp_bigint_from_i64_temp(int64_t value)
    module.add_function("vp_bigint_from_i64_temp", from_i64_fn_type, None);

    // vp_bigint_destroy: Destroy BigInt
    // void vp_bigint_destroy(ViperBigInt* bigint)
    let destroy_fn_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_destroy", destroy_fn_type, None);

    // char* vp_bigint_to_str(ViperBigInt* bigint, int base)
    let to_str_fn_type = ptr_type.fn_type(&[ptr_type.into(), context.i32_type().into()], false);
    module.add_function("vp_bigint_to_str", to_str_fn_type, None);

    // vp_bigint_to_i64: Convert BigInt to i64
    // int64_t vp_bigint_to_i64(ViperBigInt* bigint)
    let to_i64_fn_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_to_i64", to_i64_fn_type, None);

    // Arithmetic operations (result, a, b)
    let arithmetic_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
    
    // vp_bigint_add
    module.add_function("vp_bigint_add", arithmetic_fn_type, None);
    
    // vp_bigint_sub
    module.add_function("vp_bigint_sub", arithmetic_fn_type, None);
    
    // vp_bigint_mul
    module.add_function("vp_bigint_mul", arithmetic_fn_type, None);
    
    // vp_bigint_div
    module.add_function("vp_bigint_div", arithmetic_fn_type, None);
    
    // vp_bigint_mod
    module.add_function("vp_bigint_mod", arithmetic_fn_type, None);
    
    // vp_bigint_and
    module.add_function("vp_bigint_and", arithmetic_fn_type, None);
    
    // vp_bigint_or
    module.add_function("vp_bigint_or", arithmetic_fn_type, None);
    
    // vp_bigint_xor
    module.add_function("vp_bigint_xor", arithmetic_fn_type, None);
    
    // vp_bigint_lshift
    module.add_function("vp_bigint_lshift", arithmetic_fn_type, None);
    
    // vp_bigint_rshift
    module.add_function("vp_bigint_rshift", arithmetic_fn_type, None);

    // Comparison operations (return bool)
    let cmp_fn_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    
    // vp_bigint_eq
    module.add_function("vp_bigint_eq", cmp_fn_type, None);
    
    // vp_bigint_lt
    module.add_function("vp_bigint_lt", cmp_fn_type, None);
    
    // vp_bigint_gt
    module.add_function("vp_bigint_gt", cmp_fn_type, None);

    // Power operation
    module.add_function("vp_bigint_pow", arithmetic_fn_type, None);
    
    // Square root
    let sqrt_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_sqrt", sqrt_fn_type, None);

    // Absolute value
    let unary_arith_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_abs", unary_arith_fn_type, None);

    // Negation
    module.add_function("vp_bigint_neg", unary_arith_fn_type, None);

    // Inversion (bitwise NOT)
    module.add_function("vp_bigint_invert", unary_arith_fn_type, None);

    // Boolean checks
    let check_fn_type = bool_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_is_zero", check_fn_type, None);
    module.add_function("vp_bigint_is_negative", check_fn_type, None);

    // Utility
    let sign_fn_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_sign", sign_fn_type, None);

    let bit_len_fn_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_bit_length", bit_len_fn_type, None);

    Ok(())
}
