//! Math JIT stub registration - BigInt, Decimal, and Math functions

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

pub fn register_math_stubs(ee: &ExecutionEngine, module: &Module) {
    // BigInt runtime functions
    register_stubs!(ee, module, [
        "vp_bigint_from_i64" => super::super::bigint::vp_bigint_from_i64_stub,
        "vp_bigint_from_str" => super::super::bigint::vp_bigint_from_str_stub,
        "vp_bigint_add" => super::super::bigint::vp_bigint_add_stub,
        "vp_bigint_sub" => super::super::bigint::vp_bigint_sub_stub,
        "vp_bigint_mul" => super::super::bigint::vp_bigint_mul_stub,
        "vp_bigint_div" => super::super::bigint::vp_bigint_div_stub,
        "vp_bigint_mod" => super::super::bigint::vp_bigint_mod_stub,
        "vp_bigint_pow" => super::super::bigint::vp_bigint_pow_stub,
        "vp_bigint_powmod" => super::super::bigint::vp_bigint_powmod_stub,
        "vp_bigint_divmod" => super::super::bigint::vp_bigint_divmod_stub,
        "vp_bigint_sqrt" => super::super::bigint::vp_bigint_sqrt_stub,
        "vp_bigint_gcd" => super::super::bigint::vp_bigint_gcd_stub,
        "vp_bigint_lcm" => super::super::bigint::vp_bigint_lcm_stub,
        "vp_bigint_factorial" => super::super::bigint::vp_bigint_factorial_stub,
        "vp_bigint_comb" => super::super::bigint::vp_bigint_comb_stub,
        "vp_bigint_perm" => super::super::bigint::vp_bigint_perm_stub,
        "vp_bigint_neg" => super::super::bigint::vp_bigint_neg_stub,
        "vp_bigint_abs" => super::super::bigint::vp_bigint_abs_stub,
        "vp_bigint_cmp" => super::super::bigint::vp_bigint_cmp_stub,
        "vp_bigint_eq" => super::super::bigint::vp_bigint_eq_stub,
        "vp_bigint_lt" => super::super::bigint::vp_bigint_lt_stub,
        "vp_bigint_le" => super::super::bigint::vp_bigint_le_stub,
        "vp_bigint_gt" => super::super::bigint::vp_bigint_gt_stub,
        "vp_bigint_ge" => super::super::bigint::vp_bigint_ge_stub,
        "vp_bigint_to_str" => super::super::bigint::vp_bigint_to_str_stub,
        "vp_bigint_free" => super::super::bigint::vp_bigint_free_stub,
    ]);

    // Decimal functions
    register_stubs!(ee, module, [
        "vp_decimal_create" => super::super::decimal_mod::vp_decimal_create,
        "vp_decimal_from_str" => super::super::decimal_mod::vp_decimal_from_str,
        "vp_decimal_from_i64" => super::super::decimal_mod::vp_decimal_from_i64,
        "vp_decimal_from_f64" => super::super::decimal_mod::vp_decimal_from_f64,
        "vp_decimal_free" => super::super::decimal_mod::vp_decimal_free,
        "vp_decimal_to_str" => super::super::decimal_mod::vp_decimal_to_str,
        "vp_decimal_to_i64" => super::super::decimal_mod::vp_decimal_to_i64,
        "vp_decimal_to_f64" => super::super::decimal_mod::vp_decimal_to_f64,
        "vp_decimal_add" => super::super::decimal_mod::vp_decimal_add,
        "vp_decimal_sub" => super::super::decimal_mod::vp_decimal_sub,
        "vp_decimal_mul" => super::super::decimal_mod::vp_decimal_mul,
        "vp_decimal_div" => super::super::decimal_mod::vp_decimal_div,
        "vp_decimal_neg" => super::super::decimal_mod::vp_decimal_neg,
        "vp_decimal_abs" => super::super::decimal_mod::vp_decimal_abs,
        "vp_decimal_cmp" => super::super::decimal_mod::vp_decimal_cmp,
        "vp_decimal_eq" => super::super::decimal_mod::vp_decimal_eq,
        "vp_decimal_lt" => super::super::decimal_mod::vp_decimal_lt,
        "vp_decimal_le" => super::super::decimal_mod::vp_decimal_le,
        "vp_decimal_gt" => super::super::decimal_mod::vp_decimal_gt,
        "vp_decimal_ge" => super::super::decimal_mod::vp_decimal_ge,
        "vp_decimal_quantize" => super::super::decimal_mod::vp_decimal_quantize,
        "vp_decimal_round" => super::super::decimal_mod::vp_decimal_round,
        "vp_decimal_floor" => super::super::decimal_mod::vp_decimal_floor,
        "vp_decimal_ceil" => super::super::decimal_mod::vp_decimal_ceil,
        "vp_decimal_get_sign" => super::super::decimal_mod::vp_decimal_get_sign,
        "vp_decimal_get_scale" => super::super::decimal_mod::vp_decimal_get_scale,
        "vp_decimal_is_zero" => super::super::decimal_mod::vp_decimal_is_zero,
        "vp_decimal_is_nan" => super::super::decimal_mod::vp_decimal_is_nan,
        "vp_decimal_is_infinite" => super::super::decimal_mod::vp_decimal_is_infinite,
        "vp_decimal_is_signed" => super::super::decimal_mod::vp_decimal_is_signed,
        "vp_decimal_zero" => super::super::decimal_mod::vp_decimal_zero,
        "vp_decimal_one" => super::super::decimal_mod::vp_decimal_one,
        "vp_decimal_pi" => super::super::decimal_mod::vp_decimal_pi,
        "vp_decimal_e" => super::super::decimal_mod::vp_decimal_e,
    ]);

    // Math functions - basic (from math.rs)
    register_stubs!(ee, module, [
        "vp_math_sqrt" => super::super::math::vp_math_sqrt,
        "vp_math_abs" => super::super::math::vp_math_abs,
        "vp_math_ln" => super::super::math::vp_math_ln,
        "vp_math_floor" => super::super::math::vp_math_floor,
        "vp_math_sin" => super::super::math::vp_math_sin,
        "vp_math_cos" => super::super::math::vp_math_cos,
        "vp_math_tan" => super::super::math::vp_math_tan,
        "vp_math_log2" => super::super::math::vp_math_log2,
        "vp_math_log10" => super::super::math::vp_math_log10,
        "vp_pow" => super::super::math::vp_pow_stub,
        "vp_pow_i64" => super::super::math::vp_pow_i64_stub,
    ]);

    // Math constants (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_pi" => super::super::math_mod::vp_math_pi,
        "vp_math_e" => super::super::math_mod::vp_math_e,
        "vp_math_tau" => super::super::math_mod::vp_math_tau,
        "vp_math_inf" => super::super::math_mod::vp_math_inf,
        "vp_math_nan" => super::super::math_mod::vp_math_nan,
    ]);

    // Math functions - basic continued (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_cbrt" => super::super::math_mod::vp_math_cbrt,
        "vp_math_ceil" => super::super::math_mod::vp_math_ceil,
        "vp_math_trunc" => super::super::math_mod::vp_math_trunc,
        "vp_math_round" => super::super::math_mod::vp_math_round,
        "vp_math_fabs" => super::super::math_mod::vp_math_fabs,
    ]);

    // Power and logarithm (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_exp" => super::super::math_mod::vp_math_exp,
        "vp_math_exp2" => super::super::math_mod::vp_math_exp2,
        "vp_math_exp10" => super::super::math_mod::vp_math_exp10,
        "vp_math_log" => super::super::math_mod::vp_math_log,
        "vp_math_log2" => super::super::math_mod::vp_math_log2,
        "vp_math_log10" => super::super::math_mod::vp_math_log10,
        "vp_math_pow" => super::super::math_mod::vp_math_pow,
        "vp_math_pow_i64" => super::super::math_mod::vp_math_pow_i64,
    ]);

    // Trigonometric (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_sin" => super::super::math_mod::vp_math_sin,
        "vp_math_cos" => super::super::math_mod::vp_math_cos,
        "vp_math_tan" => super::super::math_mod::vp_math_tan,
        "vp_math_asin" => super::super::math_mod::vp_math_asin,
        "vp_math_acos" => super::super::math_mod::vp_math_acos,
        "vp_math_atan" => super::super::math_mod::vp_math_atan,
        "vp_math_atan2" => super::super::math_mod::vp_math_atan2,
    ]);

    // Hyperbolic (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_sinh" => super::super::math_mod::vp_math_sinh,
        "vp_math_cosh" => super::super::math_mod::vp_math_cosh,
        "vp_math_tanh" => super::super::math_mod::vp_math_tanh,
        "vp_math_asinh" => super::super::math_mod::vp_math_asinh,
        "vp_math_acosh" => super::super::math_mod::vp_math_acosh,
        "vp_math_atanh" => super::super::math_mod::vp_math_atanh,
    ]);

    // Angle conversion (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_degrees" => super::super::math_mod::vp_math_degrees,
        "vp_math_radians" => super::super::math_mod::vp_math_radians,
    ]);

    // Rounding and remainder (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_fmod" => super::super::math_mod::vp_math_fmod,
        "vp_math_fmin" => super::super::math_mod::vp_math_fmin,
        "vp_math_fmax" => super::super::math_mod::vp_math_fmax,
    ]);

    // Classification (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_isnan" => super::super::math_mod::vp_math_isnan,
        "vp_math_isinf" => super::super::math_mod::vp_math_isinf,
        "vp_math_isfinite" => super::super::math_mod::vp_math_isfinite,
        "vp_math_isnormal" => super::super::math_mod::vp_math_isnormal,
        "vp_math_signbit" => super::super::math_mod::vp_math_signbit,
    ]);

    // Special functions (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_erf" => super::super::math_mod::vp_math_erf,
        "vp_math_erfc" => super::super::math_mod::vp_math_erfc,
        "vp_math_tgamma" => super::super::math_mod::vp_math_tgamma,
        "vp_math_lgamma" => super::super::math_mod::vp_math_lgamma,
    ]);

    // Integer math (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_abs_i64" => super::super::math_mod::vp_math_abs_i64,
        "vp_math_min_i64" => super::super::math_mod::vp_math_min_i64,
        "vp_math_max_i64" => super::super::math_mod::vp_math_max_i64,
        "vp_math_clamp_i64" => super::super::math_mod::vp_math_clamp_i64,
        "vp_math_gcd" => super::super::math_mod::vp_math_gcd,
        "vp_math_lcm" => super::super::math_mod::vp_math_lcm,
        "vp_math_factorial" => super::super::math_mod::vp_math_factorial,
        "vp_math_factorial_large" => super::super::math_mod::vp_math_factorial_large,
        "vp_math_comb" => super::super::math_mod::vp_math_comb,
        "vp_math_perm" => super::super::math_mod::vp_math_perm,
    ]);

    // Distance functions (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_hypot" => super::super::math_mod::vp_math_hypot,
        "vp_math_dist_2d" => super::super::math_mod::vp_math_dist_2d,
        "vp_math_dist_3d" => super::super::math_mod::vp_math_dist_3d,
    ]);

    // Advanced functions (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_copysign" => super::super::math_mod::vp_math_copysign,
        "vp_math_remainder" => super::super::math_mod::vp_math_remainder,
        "vp_math_fma" => super::super::math_mod::vp_math_fma,
        "vp_math_ilogb" => super::super::math_mod::vp_math_ilogb,
        "vp_math_logb" => super::super::math_mod::vp_math_logb,
        "vp_math_scalbn" => super::super::math_mod::vp_math_scalbn,
        "vp_math_fdim" => super::super::math_mod::vp_math_fdim,
        "vp_math_nextafter" => super::super::math_mod::vp_math_nextafter,
        "vp_math_fpclassify" => super::super::math_mod::vp_math_fpclassify,
    ]);

    // Statistics (from math_mod.rs)
    register_stubs!(ee, module, [
        "vp_math_mean" => super::super::math_mod::vp_math_mean,
        "vp_math_variance" => super::super::math_mod::vp_math_variance,
        "vp_math_stddev" => super::super::math_mod::vp_math_stddev,
    ]);
}
