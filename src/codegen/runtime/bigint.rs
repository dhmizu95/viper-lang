//! BigInt runtime function declarations for LLVM codegen

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::AddressSpace;

pub fn declare_bigint_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> Result<(), String> {
    let ptr_type = context.ptr_type(AddressSpace::default());
    let i64_type = context.i64_type();
    let i32_type = context.i32_type();
    let i8_ptr_type = context.ptr_type(AddressSpace::default());
    let void_type = context.void_type();

    // Constructor: from i64
    let from_i64_type = ptr_type.fn_type(&[i64_type.into()], false);
    module.add_function("vp_bigint_from_i64", from_i64_type, None);

    // Constructor: from string
    let from_str_type = ptr_type.fn_type(&[i8_ptr_type.into()], false);
    module.add_function("vp_bigint_from_str", from_str_type, None);

    // Arithmetic operations
    let binop_type = ptr_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_add", binop_type, None);
    module.add_function("vp_bigint_sub", binop_type, None);
    module.add_function("vp_bigint_mul", binop_type, None);

    // True division returns f64
    let div_type = context.f64_type().fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_div", div_type, None);

    // Floor division returns BigInt
    module.add_function("vp_bigint_floor_div", binop_type, None);
    module.add_function("vp_bigint_mod", binop_type, None);

    // Power operations
    // pow(base, exp) - exp is u64
    let pow_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bigint_pow", pow_type, None);

    // pow_mod(base, exp, mod)
    module.add_function("vp_bigint_pow_mod", binop_type, None);

    // Bitwise operations
    module.add_function("vp_bigint_and", binop_type, None);
    module.add_function("vp_bigint_or", binop_type, None);
    module.add_function("vp_bigint_xor", binop_type, None);

    // Unary bitwise NOT
    let unop_type = ptr_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_not", unop_type, None);

    // Shift operations - shift amount is u64
    let shift_type = ptr_type.fn_type(&[ptr_type.into(), i64_type.into()], false);
    module.add_function("vp_bigint_shl", shift_type, None);
    module.add_function("vp_bigint_shr", shift_type, None);

    // Comparison - returns i32 (-1, 0, 1)
    let cmp_type = i32_type.fn_type(&[ptr_type.into(), ptr_type.into()], false);
    module.add_function("vp_bigint_cmp", cmp_type, None);

    // Methods
    let bitlen_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_bit_length", bitlen_type, None);

    // Conversion
    let to_i64_type = i64_type.fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_to_i64", to_i64_type, None);

    let to_f64_type = context.f64_type().fn_type(&[ptr_type.into()], false);
    module.add_function("vp_bigint_to_f64", to_f64_type, None);

    // ARC memory management (stubs for now)
    module.add_function("vp_bigint_retain", void_type.fn_type(&[ptr_type.into()], false), None);
    module.add_function("vp_bigint_release", void_type.fn_type(&[ptr_type.into()], false), None);

    // Print
    module.add_function("vp_bigint_print", void_type.fn_type(&[ptr_type.into()], false), None);

    Ok(())
}
