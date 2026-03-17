/**
 * Viper Runtime - Math Module
 * Comprehensive math functions from <math.h>
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <float.h>
#include <stdint.h>
#include "viper_stdlib.h"

/* ============================================ */
/* Mathematical Constants                       */
/* ============================================ */

double vp_math_pi(void) {
    return M_PI;
}

double vp_math_e(void) {
    return M_E;
}

double vp_math_tau(void) {
    return 2.0 * M_PI;
}

double vp_math_inf(void) {
    return INFINITY;
}

double vp_math_nan(void) {
    return NAN;
}

/* ============================================ */
/* Basic Math Functions                         */
/* ============================================ */

double vp_math_sqrt(double x) {
    return sqrt(x);
}

double vp_math_cbrt(double x) {
    return cbrt(x);
}

double vp_math_floor(double x) {
    return floor(x);
}

double vp_math_ceil(double x) {
    return ceil(x);
}

double vp_math_trunc(double x) {
    return trunc(x);
}

double vp_math_round(double x) {
    return round(x);
}

double vp_math_abs(double x) {
    return fabs(x);
}

double vp_math_fabs(double x) {
    return fabs(x);
}

/* ============================================ */
/* Power and Logarithm Functions                */
/* ============================================ */

double vp_math_log(double x) {
    return log(x);
}

double vp_math_log2(double x) {
    return log2(x);
}

double vp_math_log10(double x) {
    return log10(x);
}

double vp_math_exp(double x) {
    return exp(x);
}

double vp_math_exp2(double x) {
    return exp2(x);
}

double vp_math_exp10(double x) {
    return pow(10.0, x);
}

double vp_math_pow(double base, double exponent) {
    return pow(base, exponent);
}

double vp_math_pow_i64(int64_t base, int64_t exponent) {
    return pow((double)base, (double)exponent);
}

/* ============================================ */
/* Trigonometric Functions                      */
/* ============================================ */

double vp_math_sin(double x) {
    return sin(x);
}

double vp_math_cos(double x) {
    return cos(x);
}

double vp_math_tan(double x) {
    return tan(x);
}

double vp_math_asin(double x) {
    return asin(x);
}

double vp_math_acos(double x) {
    return acos(x);
}

double vp_math_atan(double x) {
    return atan(x);
}

double vp_math_atan2(double y, double x) {
    return atan2(y, x);
}

/* ============================================ */
/* Hyperbolic Functions                         */
/* ============================================ */

double vp_math_sinh(double x) {
    return sinh(x);
}

double vp_math_cosh(double x) {
    return cosh(x);
}

double vp_math_tanh(double x) {
    return tanh(x);
}

double vp_math_asinh(double x) {
    return asinh(x);
}

double vp_math_acosh(double x) {
    return acosh(x);
}

double vp_math_atanh(double x) {
    return atanh(x);
}

/* ============================================ */
/* Angle Conversion                             */
/* ============================================ */

double vp_math_degrees(double radians) {
    return radians * (180.0 / M_PI);
}

double vp_math_radians(double degrees) {
    return degrees * (M_PI / 180.0);
}

/* ============================================ */
/* Rounding and Remainder Functions             */
/* ============================================ */

double vp_math_fmod(double x, double y) {
    return fmod(x, y);
}

double vp_math_remainder(double x, double y) {
    return remainder(x, y);
}

double vp_math_fma(double x, double y, double z) {
    return fma(x, y, z);
}

int64_t vp_math_ilogb(double x) {
    return ilogb(x);
}

double vp_math_logb(double x) {
    return logb(x);
}

double vp_math_scalbn(double x, int64_t n) {
    return scalbn(x, (int)n);
}

/* ============================================ */
/* Min/Max Functions                            */
/* ============================================ */

double vp_math_fmin(double x, double y) {
    return fmin(x, y);
}

double vp_math_fmax(double x, double y) {
    return fmax(x, y);
}

double vp_math_fdim(double x, double y) {
    return fdim(x, y);
}

/* ============================================ */
/* Floating Point Classification                */
/* ============================================ */

int64_t vp_math_isnan(double x) {
    return isnan(x) ? 1 : 0;
}

int64_t vp_math_isinf(double x) {
    return isinf(x) ? 1 : 0;
}

int64_t vp_math_isfinite(double x) {
    return isfinite(x) ? 1 : 0;
}

int64_t vp_math_isnormal(double x) {
    return isnormal(x) ? 1 : 0;
}

int64_t vp_math_signbit(double x) {
    return signbit(x) ? 1 : 0;
}

int64_t vp_math_fpclassify(double x) {
    return fpclassify(x);
}

/* ============================================ */
/* Error Function                               */
/* ============================================ */

double vp_math_erf(double x) {
    return erf(x);
}

double vp_math_erfc(double x) {
    return erfc(x);
}

/* ============================================ */
/* Gamma Function                               */
/* ============================================ */

double vp_math_tgamma(double x) {
    return tgamma(x);
}

double vp_math_lgamma(double x) {
    return lgamma(x);
}

/* ============================================ */
/* Integer Math                                 */
/* ============================================ */

int64_t vp_math_abs_i64(int64_t x) {
    return x < 0 ? -x : x;
}

int64_t vp_math_min_i64(int64_t a, int64_t b) {
    return a < b ? a : b;
}

int64_t vp_math_max_i64(int64_t a, int64_t b) {
    return a > b ? a : b;
}

int64_t vp_math_clamp_i64(int64_t x, int64_t min_val, int64_t max_val) {
    if (x < min_val) return min_val;
    if (x > max_val) return max_val;
    return x;
}

/* ============================================ */
/* GCD and LCM                                  */
/* ============================================ */

int64_t vp_math_gcd(int64_t a, int64_t b) {
    a = a < 0 ? -a : a;
    b = b < 0 ? -b : b;
    
    while (b != 0) {
        int64_t temp = b;
        b = a % b;
        a = temp;
    }
    
    return a;
}

int64_t vp_math_lcm(int64_t a, int64_t b) {
    if (a == 0 || b == 0) return 0;
    
    int64_t gcd_val = vp_math_gcd(a, b);
    return (a / gcd_val) * b;
}

/* ============================================ */
/* Factorial                                    */
/* ============================================ */

int64_t vp_math_factorial(int64_t n) {
    if (n < 0) return -1;  /* Error: negative input */
    if (n <= 1) return 1;
    
    int64_t result = 1;
    for (int64_t i = 2; i <= n; i++) {
        result *= i;
    }
    return result;
}

double vp_math_factorial_large(int64_t n) {
    if (n < 0) return NAN;
    if (n <= 1) return 1.0;
    
    double result = 1.0;
    for (int64_t i = 2; i <= n; i++) {
        result *= (double)i;
    }
    return result;
}

/* ============================================ */
/* Combinatorics                                */
/* ============================================ */

int64_t vp_math_comb(int64_t n, int64_t k) {
    if (k < 0 || k > n) return 0;
    if (k == 0 || k == n) return 1;
    
    /* Use symmetry: C(n,k) = C(n, n-k) */
    if (k > n - k) {
        k = n - k;
    }
    
    int64_t result = 1;
    for (int64_t i = 0; i < k; i++) {
        result = result * (n - i) / (i + 1);
    }
    return result;
}

int64_t vp_math_perm(int64_t n, int64_t k) {
    if (k < 0 || k > n) return 0;
    if (k == 0) return 1;
    
    int64_t result = 1;
    for (int64_t i = 0; i < k; i++) {
        result *= (n - i);
    }
    return result;
}

/* ============================================ */
/* Distance Functions                           */
/* ============================================ */

double vp_math_hypot(double x, double y) {
    return hypot(x, y);
}

double vp_math_dist_2d(double x1, double y1, double x2, double y2) {
    return hypot(x2 - x1, y2 - y1);
}

double vp_math_dist_3d(double x1, double y1, double z1, 
                       double x2, double y2, double z2) {
    double dx = x2 - x1;
    double dy = y2 - y1;
    double dz = z2 - z1;
    return sqrt(dx*dx + dy*dy + dz*dz);
}

/* ============================================ */
/* Special Functions                            */
/* ============================================ */

double vp_math_copysign(double x, double y) {
    return copysign(x, y);
}

double vp_math_nextafter(double x, double y) {
    return nextafter(x, y);
}

int64_t vp_math_modf(double x, double* intpart) {
    double frac = modf(x, intpart);
    int64_t bits;
    /* Return fractional part encoded as bits for precision without violating aliasing rules. */
    memcpy(&bits, &frac, sizeof(bits));
    return bits;
}

/* ============================================ */
/* Statistics Helpers                           */
/* ============================================ */

double vp_math_mean(double* values, int64_t count) {
    if (count <= 0 || !values) return 0.0;
    
    double sum = 0.0;
    for (int64_t i = 0; i < count; i++) {
        sum += values[i];
    }
    return sum / (double)count;
}

double vp_math_variance(double* values, int64_t count) {
    if (count <= 1 || !values) return 0.0;
    
    double mean = vp_math_mean(values, count);
    double sum_sq_diff = 0.0;
    
    for (int64_t i = 0; i < count; i++) {
        double diff = values[i] - mean;
        sum_sq_diff += diff * diff;
    }
    
    return sum_sq_diff / (double)(count - 1);  /* Sample variance */
}

double vp_math_stddev(double* values, int64_t count) {
    return sqrt(vp_math_variance(values, count));
}


/* ============================================ */
/* Integer Math Functions                       */
/* ============================================ */

/**
 * Integer square root using Newton's method.
 * Returns floor(sqrt(n)) for non-negative n.
 * Input is a Viper tagged integer, output is also tagged.
 */
int64_t vp_math_isqrt(int64_t n_tagged) {
    /* Untag the input */
    int64_t n = n_tagged >> 1;
    
    if (n < 0) {
        return 0;  /* Error: negative input, return 0 tagged */
    }
    if (n == 0) {
        return 0;  /* Tagged 0 */
    }

    /* Initial guess */
    int64_t x = n;
    int64_t y = (x + 1) / 2;

    /* Newton's method iteration */
    while (y < x) {
        x = y;
        y = (x + n / x) / 2;
    }

    /* Tag the result */
    return (x << 1);
}
