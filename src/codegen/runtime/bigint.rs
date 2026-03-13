//! BigInt runtime function declarations for GMP integration

use inkwell::context::Context;
use inkwell::module::Module;

/// Declare BigInt runtime functions with optimization attributes
pub fn declare_bigint_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let i64_type = context.i64_type();
    let bool_type = context.bool_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let void_type = context.void_type();

    // Helper for adding optimization attributes (bigint has memory effects, so no readnone)
    let add_opt_attrs = |func: inkwell::values::FunctionValue<'ctx>| {
        func.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context.create_string_attribute("alwaysinline", ""),
        );
        func.add_attribute(
            inkwell::attributes::AttributeLoc::Function,
            context.create_string_attribute("willreturn", ""),
        );
    };

    // vp_bigint_from_str: Create BigInt from string
    // ViperBigInt* vp_bigint_from_str(const char* str)
    let from_str_fn_type = ptr_type.fn_type(&[ptr_type.into()], false);
    let func = module.add_function("vp_bigint_from_str", from_str_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_from_i64: Create BigInt from i64
    // ViperBigInt* vp_bigint_from_i64(int64_t value)
    let from_i64_fn_type = ptr_type.fn_type(&[i64_type.into()], false);
    let func = module.add_function("vp_bigint_from_i64", from_i64_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_from_i64_temp: Create BigInt from i64 for temporary results
    // ViperBigInt* vp_bigint_from_i64_temp(int64_t value)
    let func = module.add_function("vp_bigint_from_i64_temp", from_i64_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_destroy: Destroy BigInt
    // void vp_bigint_destroy(ViperBigInt* bigint)
    let destroy_fn_type = void_type.fn_type(&[ptr_type.into()], false);
    let func = module.add_function("vp_bigint_destroy", destroy_fn_type, None);
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    // char* vp_bigint_to_str(ViperBigInt* bigint, int base)
    let to_str_fn_type = ptr_type.fn_type(&[ptr_type.into(), context.i32_type().into()], false);
    let func = module.add_function("vp_bigint_to_str", to_str_fn_type, None);
    func.add_attribute(
        inkwell::attributes::AttributeLoc::Function,
        context.create_string_attribute("alwaysinline", ""),
    );

    // vp_bigint_to_i64: Convert BigInt to i64
    // int64_t vp_bigint_to_i64(ViperBigInt* bigint)
    let to_i64_fn_type = i64_type.fn_type(&[ptr_type.into()], false);
    let func = module.add_function("vp_bigint_to_i64", to_i64_fn_type, None);
    add_opt_attrs(func);

    // Arithmetic operations (result, a, b)
    let arithmetic_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
    
    // vp_bigint_add
    let func = module.add_function("vp_bigint_add", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_sub
    let func = module.add_function("vp_bigint_sub", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_mul
    let func = module.add_function("vp_bigint_mul", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_div
    let func = module.add_function("vp_bigint_div", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_mod
    let func = module.add_function("vp_bigint_mod", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_and
    let func = module.add_function("vp_bigint_and", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_or
    let func = module.add_function("vp_bigint_or", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_xor
    let func = module.add_function("vp_bigint_xor", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_lshift
    let func = module.add_function("vp_bigint_lshift", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_rshift
    let func = module.add_function("vp_bigint_rshift", arithmetic_fn_type, None);
    add_opt_attrs(func);

    // Comparison operations (return bool)
    let cmp_fn_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    
    // vp_bigint_eq
    let func = module.add_function("vp_bigint_eq", cmp_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_lt
    let func = module.add_function("vp_bigint_lt", cmp_fn_type, None);
    add_opt_attrs(func);
    
    // vp_bigint_gt
    let func = module.add_function("vp_bigint_gt", cmp_fn_type, None);
    add_opt_attrs(func);

    // Power operation
    let func = module.add_function("vp_bigint_pow", arithmetic_fn_type, None);
    add_opt_attrs(func);
    
    // Square root
    let sqrt_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    let func = module.add_function("vp_bigint_sqrt", sqrt_fn_type, None);
    add_opt_attrs(func);

    // Absolute value
    let unary_arith_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    let func = module.add_function("vp_bigint_abs", unary_arith_fn_type, None);
    add_opt_attrs(func);

    // Negation
    let func = module.add_function("vp_bigint_neg", unary_arith_fn_type, None);
    add_opt_attrs(func);

    // Inversion (bitwise NOT)
    let func = module.add_function("vp_bigint_invert", unary_arith_fn_type, None);
    add_opt_attrs(func);

    // Boolean checks
    let check_fn_type = bool_type.fn_type(&[ptr_type.into()], false);
    let func = module.add_function("vp_bigint_is_zero", check_fn_type, None);
    add_opt_attrs(func);
    let func = module.add_function("vp_bigint_is_negative", check_fn_type, None);
    add_opt_attrs(func);

    // Utility
    let sign_fn_type = i64_type.fn_type(&[ptr_type.into()], false);
    let func = module.add_function("vp_bigint_sign", sign_fn_type, None);
    add_opt_attrs(func);

    let bit_len_fn_type = i64_type.fn_type(&[ptr_type.into()], false);
    let func = module.add_function("vp_bigint_bit_length", bit_len_fn_type, None);
    add_opt_attrs(func);

    // Math operations (result, a, b) or (result, n)
    // vp_bigint_gcd (result, a, b)
    let func = module.add_function("vp_bigint_gcd", arithmetic_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_lcm (result, a, b)
    let func = module.add_function("vp_bigint_lcm", arithmetic_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_factorial (result, n) - 2 arguments
    let unary_arith_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    let func = module.add_function("vp_bigint_factorial", unary_arith_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_comb (result, n, k)
    let func = module.add_function("vp_bigint_comb", arithmetic_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_perm (result, n, k)
    let func = module.add_function("vp_bigint_perm", arithmetic_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_powmod (result, base, exp, mod) - 4 arguments
    let powmod_fn_type = void_type.fn_type(&[ptr_type.into(), ptr_type.into(), ptr_type.into(), ptr_type.into()], false);
    let func = module.add_function("vp_bigint_powmod", powmod_fn_type, None);
    add_opt_attrs(func);

    // vp_bigint_min and vp_bigint_max (result, a, b)
    let func = module.add_function("vp_bigint_min", arithmetic_fn_type, None);
    add_opt_attrs(func);
    let func = module.add_function("vp_bigint_max", arithmetic_fn_type, None);
    add_opt_attrs(func);

    Ok(())
}
