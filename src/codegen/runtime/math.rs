use inkwell::context::Context;
use inkwell::module::Module;

/// Declare math builtin runtime functions
pub fn declare_math_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let f64_type = context.f64_type();
    let i64_type = context.i64_type();

    // sqrt(x) - square root
    let sqrt_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_sqrt", sqrt_type, None);

    // abs(x) - absolute value for floats
    let abs_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_abs", abs_type, None);

    // abs_i64(x) - absolute value for integers
    let abs_i64_type = i64_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_math_abs_i64", abs_i64_type, None);

    // ln(x) - natural logarithm
    let ln_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_ln", ln_type, None);

    // floor(x) - floor function
    let floor_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_floor", floor_type, None);

    // sin(x) - sine function
    let sin_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_sin", sin_type, None);

    // cos(x) - cosine function
    let cos_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_cos", cos_type, None);

    // tan(x) - tangent function
    let tan_type = f64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_math_tan", tan_type, None);

    // pow(base, exponent) - power function for floats
    let pow_type = f64_type.fn_type(&[f64_type.into(), f64_type.into()], false);
    module.add_function("vp_pow", pow_type, None);

    // pow_i64(base, exponent) - power function for integers
    let pow_i64_type = i64_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    module.add_function("vp_pow_i64", pow_i64_type, None);

    Ok(())
}

/// Declare hash builtin runtime functions
pub fn declare_hash_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> crate::codegen::Result<()> {
    let i64_type = context.i64_type();
    let f64_type = context.f64_type();
    let bool_type = context.bool_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

    // hash(i64) -> i64
    let hash_i64_type = i64_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_hash_i64", hash_i64_type, None);

    // hash(f64) -> i64
    let hash_f64_type = i64_type.fn_type(&[f64_type.into()], false);
    module.add_function("vp_hash_f64", hash_f64_type, None);

    // hash(bool) -> i64
    let hash_bool_type = i64_type.fn_type(&[bool_type.into()], false);
    module.add_function("vp_hash_bool", hash_bool_type, None);

    // hash(str) -> i64
    let hash_str_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_hash_str", hash_str_type, None);

    // hash(None) -> i64 (no arguments)
    let hash_none_type = i64_type.fn_type(&[], false);
    module.add_function("vp_hash_none", hash_none_type, None);

    Ok(())
}
