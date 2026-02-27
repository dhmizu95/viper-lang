use inkwell::context::Context;
use inkwell::module::Module;

/// Declare BigInt runtime functions
pub fn declare_bigint_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let void_type = context.void_type();
    let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let bool_type = context.bool_type();

    // VpBigInt* vp_bigint_from_i64(int64_t v)
    let from_i64_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_bigint_from_i64", from_i64_type, None);

    // VpBigInt* vp_bigint_from_str(const char* s)
    let from_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_from_str", from_str_type, None);

    // VpBigInt* vp_bigint_add(VpBigInt* a, VpBigInt* b)
    let add_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_add", add_type, None);

    // VpBigInt* vp_bigint_sub(VpBigInt* a, VpBigInt* b)
    let sub_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_sub", sub_type, None);

    // VpBigInt* vp_bigint_mul(VpBigInt* a, VpBigInt* b)
    let mul_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_mul", mul_type, None);

    // VpBigInt* vp_bigint_div(VpBigInt* a, VpBigInt* b)
    let div_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_div", div_type, None);

    // VpBigInt* vp_bigint_mod(VpBigInt* a, VpBigInt* b)
    let mod_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_mod", mod_type, None);

    // VpBigInt* vp_bigint_pow(VpBigInt* base, VpBigInt* exp)
    let pow_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_pow", pow_type, None);

    // VpBigInt* vp_bigint_neg(VpBigInt* a)
    let neg_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_neg", neg_type, None);

    // VpBigInt* vp_bigint_abs(VpBigInt* a)
    let abs_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_abs", abs_type, None);

    // int vp_bigint_cmp(VpBigInt* a, VpBigInt* b)
    let cmp_type = i64_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_cmp", cmp_type, None);

    // bool vp_bigint_eq(VpBigInt* a, VpBigInt* b)
    let eq_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_eq", eq_type, None);

    // bool vp_bigint_lt(VpBigInt* a, VpBigInt* b)
    let lt_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_lt", lt_type, None);

    // bool vp_bigint_le(VpBigInt* a, VpBigInt* b)
    let le_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_le", le_type, None);

    // bool vp_bigint_gt(VpBigInt* a, VpBigInt* b)
    let gt_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_gt", gt_type, None);

    // bool vp_bigint_ge(VpBigInt* a, VpBigInt* b)
    let ge_type = bool_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_ge", ge_type, None);

    // char* vp_bigint_to_str(VpBigInt* a)
    let to_str_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_to_str", to_str_type, None);

    // void vp_bigint_free(VpBigInt* a)
    let free_type = void_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_free", free_type, None);

    Ok(())
}
