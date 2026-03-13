// Math module stubs for JIT - Phase 2
use std::f64::consts;

pub extern "C" fn vp_math_pi() -> f64 {
    consts::PI
}

pub extern "C" fn vp_math_e() -> f64 {
    consts::E
}

pub extern "C" fn vp_math_tau() -> f64 {
    consts::TAU
}

pub extern "C" fn vp_math_inf() -> f64 {
    f64::INFINITY
}

pub extern "C" fn vp_math_nan() -> f64 {
    f64::NAN
}

pub extern "C" fn vp_math_cbrt(x: f64) -> f64 {
    x.cbrt()
}

pub extern "C" fn vp_math_ceil(x: f64) -> f64 {
    x.ceil()
}

pub extern "C" fn vp_math_trunc(x: f64) -> f64 {
    x.trunc()
}

pub extern "C" fn vp_math_round(x: f64) -> f64 {
    x.round()
}

pub extern "C" fn vp_math_fabs(x: f64) -> f64 {
    x.abs()
}

pub extern "C" fn vp_math_exp(x: f64) -> f64 {
    x.exp()
}

pub extern "C" fn vp_math_exp2(x: f64) -> f64 {
    x.exp2()
}

pub extern "C" fn vp_math_exp10(x: f64) -> f64 {
    10f64.powf(x)
}

pub extern "C" fn vp_math_log(x: f64) -> f64 {
    x.ln()
}

pub extern "C" fn vp_math_log2(x: f64) -> f64 {
    x.log2()
}

pub extern "C" fn vp_math_log10(x: f64) -> f64 {
    x.log10()
}

pub extern "C" fn vp_math_pow(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

pub extern "C" fn vp_math_pow_i64(base: i64, exponent: i64) -> f64 {
    (base as f64).powf(exponent as f64)
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

pub extern "C" fn vp_math_asin(x: f64) -> f64 {
    x.asin()
}

pub extern "C" fn vp_math_acos(x: f64) -> f64 {
    x.acos()
}

pub extern "C" fn vp_math_atan(x: f64) -> f64 {
    x.atan()
}

pub extern "C" fn vp_math_atan2(y: f64, x: f64) -> f64 {
    y.atan2(x)
}

pub extern "C" fn vp_math_sinh(x: f64) -> f64 {
    x.sinh()
}

pub extern "C" fn vp_math_cosh(x: f64) -> f64 {
    x.cosh()
}

pub extern "C" fn vp_math_tanh(x: f64) -> f64 {
    x.tanh()
}

pub extern "C" fn vp_math_asinh(x: f64) -> f64 {
    x.asinh()
}

pub extern "C" fn vp_math_acosh(x: f64) -> f64 {
    x.acosh()
}

pub extern "C" fn vp_math_atanh(x: f64) -> f64 {
    x.atanh()
}

pub extern "C" fn vp_math_degrees(radians: f64) -> f64 {
    radians.to_degrees()
}

pub extern "C" fn vp_math_radians(degrees: f64) -> f64 {
    degrees.to_radians()
}

pub extern "C" fn vp_math_fmod(x: f64, y: f64) -> f64 {
    x % y
}

pub extern "C" fn vp_math_fmin(x: f64, y: f64) -> f64 {
    x.min(y)
}

pub extern "C" fn vp_math_fmax(x: f64, y: f64) -> f64 {
    x.max(y)
}

pub extern "C" fn vp_math_isnan(x: f64) -> i64 {
    if x.is_nan() {
        1
    } else {
        0
    }
}

pub extern "C" fn vp_math_isinf(x: f64) -> i64 {
    if x.is_infinite() {
        1
    } else {
        0
    }
}

pub extern "C" fn vp_math_isfinite(x: f64) -> i64 {
    if x.is_finite() {
        1
    } else {
        0
    }
}

pub extern "C" fn vp_math_isnormal(x: f64) -> i64 {
    if x.is_normal() {
        1
    } else {
        0
    }
}

pub extern "C" fn vp_math_signbit(x: f64) -> i64 {
    if x.is_sign_negative() {
        1
    } else {
        0
    }
}

pub extern "C" fn vp_math_erf(x: f64) -> f64 {
    // Approximation of error function using Abramowitz and Stegun formula 7.1.26
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + 0.3275911 * x);
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;

    let erf = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * erf
}

pub extern "C" fn vp_math_erfc(x: f64) -> f64 {
    1.0 - vp_math_erf(x)
}

pub extern "C" fn vp_math_tgamma(x: f64) -> f64 {
    // Gamma function approximation
    if x <= 0.0 && x.fract() == 0.0 {
        return f64::NAN;
    }
    // Use Lanczos approximation for small values
    if x < 0.5 {
        return std::f64::consts::PI / (x * (-x).sin() * vp_math_tgamma(1.0 - x));
    }
    // For x >= 0.5, use simple approximation
    let mut result = 1.0;
    let mut n = x;
    while n > 2.0 {
        n -= 1.0;
        result *= n;
    }
    result
}

pub extern "C" fn vp_math_lgamma(x: f64) -> f64 {
    vp_math_tgamma(x).ln()
}

pub extern "C" fn vp_math_abs_i64(x: i64) -> i64 {
    x.abs()
}

pub extern "C" fn vp_math_min_i64(a: i64, b: i64) -> i64 {
    if a < b {
        a
    } else {
        b
    }
}

pub extern "C" fn vp_math_max_i64(a: i64, b: i64) -> i64 {
    if a > b {
        a
    } else {
        b
    }
}

pub extern "C" fn vp_math_clamp_i64(x: i64, min_val: i64, max_val: i64) -> i64 {
    if x < min_val {
        min_val
    } else if x > max_val {
        max_val
    } else {
        x
    }
}

pub extern "C" fn vp_math_gcd(a: i64, b: i64) -> i64 {
    let mut a = a.abs();
    let mut b = b.abs();
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

pub extern "C" fn vp_math_lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    (a / vp_math_gcd(a, b)) * b
}

pub extern "C" fn vp_math_factorial(n: i64) -> i64 {
    if n < 0 {
        return -1;
    }
    if n <= 1 {
        return 1;
    }
    let mut result = 1;
    for i in 2..=n {
        result *= i;
    }
    result
}

pub extern "C" fn vp_math_factorial_large(n: i64) -> f64 {
    if n < 0 {
        return f64::NAN;
    }
    if n <= 1 {
        return 1.0;
    }
    let mut result = 1.0;
    for i in 2..=n {
        result *= i as f64;
    }
    result
}

pub extern "C" fn vp_math_comb(n: i64, k: i64) -> i64 {
    if k < 0 || k > n {
        return 0;
    }
    if k == 0 || k == n {
        return 1;
    }
    let mut k = k;
    if k > n - k {
        k = n - k;
    }
    let mut result = 1;
    for i in 0..k {
        result = result * (n - i) / (i + 1);
    }
    result
}

pub extern "C" fn vp_math_perm(n: i64, k: i64) -> i64 {
    if k < 0 || k > n {
        return 0;
    }
    if k == 0 {
        return 1;
    }
    let mut result = 1;
    for i in 0..k {
        result *= n - i;
    }
    result
}

pub extern "C" fn vp_math_hypot(x: f64, y: f64) -> f64 {
    x.hypot(y)
}

pub extern "C" fn vp_math_dist_2d(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (x2 - x1).hypot(y2 - y1)
}

pub extern "C" fn vp_math_dist_3d(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let dz = z2 - z1;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub extern "C" fn vp_math_copysign(x: f64, y: f64) -> f64 {
    x.copysign(y)
}

pub extern "C" fn vp_math_remainder(x: f64, y: f64) -> f64 {
    // IEEE remainder
    let n = (x / y).round();
    x - n * y
}

pub extern "C" fn vp_math_fma(x: f64, y: f64, z: f64) -> f64 {
    x.mul_add(y, z)
}

pub extern "C" fn vp_math_ilogb(x: f64) -> i64 {
    if x == 0.0 {
        return -1;
    }
    if x.is_nan() {
        return -1;
    }
    if x.is_infinite() {
        return i32::MAX as i64;
    }
    x.log2() as i64
}

pub extern "C" fn vp_math_logb(x: f64) -> f64 {
    x.log2()
}

pub extern "C" fn vp_math_scalbn(x: f64, n: i64) -> f64 {
    x * 2f64.powi(n as i32)
}

pub extern "C" fn vp_math_fdim(x: f64, y: f64) -> f64 {
    if x > y {
        x - y
    } else {
        0.0
    }
}

pub extern "C" fn vp_math_nextafter(x: f64, y: f64) -> f64 {
    if x == y {
        return y;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }

    let bits = x.to_bits();
    if (x < y) == (bits & (1u64 << 63) == 0) {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    }
}

pub extern "C" fn vp_math_fpclassify(x: f64) -> i64 {
    if x.is_nan() {
        0
    } else if x.is_infinite() {
        1
    } else if x == 0.0 {
        2
    } else if x.is_subnormal() {
        3
    } else {
        4
    } // normal
}

pub extern "C" fn vp_math_mean(values: *const f64, count: i64) -> f64 {
    if count <= 0 || values.is_null() {
        return 0.0;
    }
    unsafe {
        let mut sum = 0.0;
        for i in 0..count as isize {
            sum += *values.offset(i);
        }
        sum / count as f64
    }
}

pub extern "C" fn vp_math_variance(values: *const f64, count: i64) -> f64 {
    if count <= 1 || values.is_null() {
        return 0.0;
    }
    unsafe {
        let mean = vp_math_mean(values, count);
        let mut sum_sq_diff = 0.0;
        for i in 0..count as isize {
            let diff = *values.offset(i) - mean;
            sum_sq_diff += diff * diff;
        }
        sum_sq_diff / (count - 1) as f64
    }
}

pub extern "C" fn vp_math_stddev(values: *const f64, count: i64) -> f64 {
    vp_math_variance(values, count).sqrt()
}
