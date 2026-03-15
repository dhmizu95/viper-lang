// Math builtins stubs for JIT
pub extern "C" fn vp_math_sqrt(x: f64) -> f64 {
    x.sqrt()
}

pub extern "C" fn vp_math_abs(x: f64) -> f64 {
    x.abs()
}

pub extern "C" fn vp_math_ln(x: f64) -> f64 {
    x.ln()
}

pub extern "C" fn vp_math_floor(x: f64) -> f64 {
    x.floor()
}

pub extern "C" fn vp_math_sin(x: f64) -> f64 {
    x.sin()
}

pub extern "C" fn vp_math_cos(x: f64) -> f64 {
    x.cos()
}

pub extern "C" fn vp_math_tan(x: f64) -> f64 {
    x.tan()
}

pub extern "C" fn vp_math_log2(x: f64) -> f64 {
    x.log2()
}

pub extern "C" fn vp_math_log10(x: f64) -> f64 {
    x.log10()
}

pub extern "C" fn vp_math_sqrt_stub(x: f64) -> f64 {
    x.sqrt()
}
pub extern "C" fn vp_math_abs_stub(x: f64) -> f64 {
    x.abs()
}
pub extern "C" fn vp_math_ln_stub(x: f64) -> f64 {
    x.ln()
}
pub extern "C" fn vp_math_floor_stub(x: f64) -> f64 {
    x.floor()
}
pub extern "C" fn vp_pow_stub(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}
pub extern "C" fn vp_pow_i64_stub(base: i64, exponent: i64) -> i64 {
    if exponent < 0 {
        panic!("Negative exponent not supported for integer power");
    }
    if exponent == 0 {
        return 1;
    }

    let mut result = 1;
    let mut b = base;
    let mut e = exponent;

    while e > 0 {
        if e & 1 == 1 {
            result *= b;
        }
        b *= b;
        e >>= 1;
    }

    result
}
