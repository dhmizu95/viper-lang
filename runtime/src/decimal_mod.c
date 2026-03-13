/**
 * Viper Runtime - Decimal Module
 * 128-bit fixed-point decimal arithmetic
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <ctype.h>
#include <math.h>
#include "viper_stdlib.h"

/* ============================================ */
/* Decimal Structure (128-bit fixed point)      */
/* ============================================ */

/* 
 * Layout: 
 * - sign: 1 bit (0=positive, 1=negative)
 * - exponent: 15 bits (0-28 scale)
 * - coefficient: 112 bits (up to 34 digits)
 */

typedef struct ViperDecimal {
    uint64_t hi;  /* High 64 bits: sign(1) + exponent(15) + coeff_high(48) */
    uint64_t lo;  /* Low 64 bits: coeff_low(64) */
} ViperDecimal;

#define DECIMAL_SIGN_MASK     0x8000000000000000ULL
#define DECIMAL_EXP_MASK      0x7FFF000000000000ULL
#define DECIMAL_COEFF_HI_MASK 0x0000FFFFFFFFFFFFULL
#define DECIMAL_MAX_SCALE     28

/* ============================================ */
/* Helper Functions                             */
/* ============================================ */

static int decimal_get_sign(const ViperDecimal* d) {
    return (d->hi & DECIMAL_SIGN_MASK) ? 1 : 0;
}

static void decimal_set_sign(ViperDecimal* d, int sign) {
    if (sign) {
        d->hi |= DECIMAL_SIGN_MASK;
    } else {
        d->hi &= ~DECIMAL_SIGN_MASK;
    }
}

static int decimal_get_scale(const ViperDecimal* d) {
    return (int)((d->hi & DECIMAL_EXP_MASK) >> 48);
}

static void decimal_set_scale(ViperDecimal* d, int scale) {
    if (scale < 0) scale = 0;
    if (scale > DECIMAL_MAX_SCALE) scale = DECIMAL_MAX_SCALE;
    d->hi = (d->hi & ~DECIMAL_EXP_MASK) | ((uint64_t)scale << 48);
}

static void decimal_set_coeff_hi(ViperDecimal* d, uint64_t coeff) {
    d->hi = (d->hi & ~DECIMAL_COEFF_HI_MASK) | (coeff & DECIMAL_COEFF_HI_MASK);
}

static void decimal_set_coeff(ViperDecimal* d, uint64_t hi, uint64_t lo) {
    decimal_set_coeff_hi(d, hi);
    d->lo = lo;
}

/* ============================================ */
/* Creation Functions                           */
/* ============================================ */

ViperDecimal* vp_decimal_create(void) {
    ViperDecimal* d = (ViperDecimal*)vp_arc_alloc(sizeof(ViperDecimal));
    if (!d) return NULL;
    
    d->hi = 0;
    d->lo = 0;
    decimal_set_scale(d, 0);
    
    return d;
}

ViperDecimal* vp_decimal_from_str(const char* str) {
    if (!str) return NULL;
    
    ViperDecimal* d = vp_decimal_create();
    if (!d) return NULL;
    
    /* Skip whitespace */
    while (isspace(*str)) str++;
    
    /* Handle sign */
    int sign = 0;
    if (*str == '-') {
        sign = 1;
        str++;
    } else if (*str == '+') {
        str++;
    }
    
    /* Parse integer part */
    uint64_t int_part = 0;
    while (isdigit(*str)) {
        int_part = int_part * 10 + (*str - '0');
        str++;
    }
    
    /* Parse fractional part */
    uint64_t frac_part = 0;
    int scale = 0;
    
    if (*str == '.') {
        str++;
        while (isdigit(*str) && scale < DECIMAL_MAX_SCALE) {
            frac_part = frac_part * 10 + (*str - '0');
            str++;
            scale++;
        }
        /* Skip remaining digits */
        while (isdigit(*str)) str++;
    }
    
    /* Combine parts */
    uint64_t coefficient = int_part;
    
    /* Adjust for scale */
    for (int i = 0; i < scale; i++) {
        coefficient = coefficient * 10 + (frac_part / 1000000000000000000ULL);
        frac_part *= 10;
    }
    
    decimal_set_sign(d, sign);
    decimal_set_scale(d, scale);
    decimal_set_coeff(d, 0, coefficient);
    
    return d;
}

ViperDecimal* vp_decimal_from_i64(int64_t value) {
    ViperDecimal* d = vp_decimal_create();
    if (!d) return NULL;
    
    if (value < 0) {
        decimal_set_sign(d, 1);
        value = -value;
    }
    
    decimal_set_coeff(d, 0, (uint64_t)value);
    decimal_set_scale(d, 0);
    
    return d;
}

ViperDecimal* vp_decimal_from_f64(double value) {
    /* Convert through string for precision */
    char buffer[64];
    snprintf(buffer, sizeof(buffer), "%.15g", value);
    return vp_decimal_from_str(buffer);
}

void vp_decimal_free(ViperDecimal* d) {
    if (d) vp_arc_release(d);
}

/* ============================================ */
/* Conversion Functions                         */
/* ============================================ */

char* vp_decimal_to_str(ViperDecimal* d) {
    if (!d) return vp_strdup_slice("0", 1);
    
    char buffer[64];
    char* p = buffer;
    
    /* Sign */
    if (decimal_get_sign(d)) {
        *p++ = '-';
    }
    
    /* Get coefficient */
    uint64_t coeff = d->lo;
    int scale = decimal_get_scale(d);
    
    /* Convert coefficient to string */
    char coeff_str[32];
    if (coeff == 0) {
        strcpy(coeff_str, "0");
    } else {
        char* c = coeff_str + 31;
        *c = '\0';
        uint64_t temp = coeff;
        while (temp > 0) {
            *--c = '0' + (temp % 10);
            temp /= 10;
        }
        memmove(coeff_str, c, strlen(c) + 1);
    }
    
    /* Add decimal point if needed */
    int coeff_len = strlen(coeff_str);
    if (scale > 0 && coeff_len <= scale) {
        *p++ = '0';
        *p++ = '.';
        for (int i = 0; i < scale - coeff_len; i++) {
            *p++ = '0';
        }
        strcpy(p, coeff_str);
    } else if (scale > 0) {
        int int_len = coeff_len - scale;
        memcpy(p, coeff_str, (size_t)int_len);
        p += int_len;
        *p++ = '.';
        strcpy(p, coeff_str + int_len);
    } else {
        strcpy(p, coeff_str);
    }
    
    return vp_strdup_slice(buffer, strlen(buffer));
}

int64_t vp_decimal_to_i64(ViperDecimal* d) {
    if (!d) return 0;
    
    int64_t result = (int64_t)d->lo;
    if (decimal_get_sign(d)) {
        result = -result;
    }
    
    return result;
}

double vp_decimal_to_f64(ViperDecimal* d) {
    if (!d) return 0.0;
    
    char* str = vp_decimal_to_str(d);
    double result = strtod(str, NULL);
    vp_arc_release(str);
    
    return result;
}

/* ============================================ */
/* Arithmetic Operations                        */
/* ============================================ */

ViperDecimal* vp_decimal_add(ViperDecimal* a, ViperDecimal* b) {
    if (!a || !b) return NULL;
    
    /* Simplified: convert to double, add, convert back */
    double result = vp_decimal_to_f64(a) + vp_decimal_to_f64(b);
    return vp_decimal_from_f64(result);
}

ViperDecimal* vp_decimal_sub(ViperDecimal* a, ViperDecimal* b) {
    if (!a || !b) return NULL;
    
    double result = vp_decimal_to_f64(a) - vp_decimal_to_f64(b);
    return vp_decimal_from_f64(result);
}

ViperDecimal* vp_decimal_mul(ViperDecimal* a, ViperDecimal* b) {
    if (!a || !b) return NULL;
    
    double result = vp_decimal_to_f64(a) * vp_decimal_to_f64(b);
    return vp_decimal_from_f64(result);
}

ViperDecimal* vp_decimal_div(ViperDecimal* a, ViperDecimal* b) {
    if (!a || !b) return NULL;
    
    /* Check for division by zero */
    if (b->lo == 0) return NULL;
    
    double result = vp_decimal_to_f64(a) / vp_decimal_to_f64(b);
    return vp_decimal_from_f64(result);
}

ViperDecimal* vp_decimal_neg(ViperDecimal* d) {
    if (!d) return NULL;
    
    ViperDecimal* result = vp_decimal_create();
    if (!result) return NULL;
    
    result->hi = d->hi ^ DECIMAL_SIGN_MASK;
    result->lo = d->lo;
    
    return result;
}

ViperDecimal* vp_decimal_abs(ViperDecimal* d) {
    if (!d) return NULL;
    
    ViperDecimal* result = vp_decimal_create();
    if (!result) return NULL;
    
    result->hi = d->hi & ~DECIMAL_SIGN_MASK;
    result->lo = d->lo;
    
    return result;
}

/* ============================================ */
/* Comparison Operations                        */
/* ============================================ */

int64_t vp_decimal_cmp(ViperDecimal* a, ViperDecimal* b) {
    if (!a || !b) return 0;
    
    double da = vp_decimal_to_f64(a);
    double db = vp_decimal_to_f64(b);
    
    if (da < db) return -1;
    if (da > db) return 1;
    return 0;
}

int64_t vp_decimal_eq(ViperDecimal* a, ViperDecimal* b) {
    return vp_decimal_cmp(a, b) == 0 ? 1 : 0;
}

int64_t vp_decimal_lt(ViperDecimal* a, ViperDecimal* b) {
    return vp_decimal_cmp(a, b) < 0 ? 1 : 0;
}

int64_t vp_decimal_le(ViperDecimal* a, ViperDecimal* b) {
    return vp_decimal_cmp(a, b) <= 0 ? 1 : 0;
}

int64_t vp_decimal_gt(ViperDecimal* a, ViperDecimal* b) {
    return vp_decimal_cmp(a, b) > 0 ? 1 : 0;
}

int64_t vp_decimal_ge(ViperDecimal* a, ViperDecimal* b) {
    return vp_decimal_cmp(a, b) >= 0 ? 1 : 0;
}

/* ============================================ */
/* Rounding                                     */
/* ============================================ */

ViperDecimal* vp_decimal_quantize(ViperDecimal* d, int scale) {
    if (!d) return NULL;
    
    /* Simplified: just change scale */
    ViperDecimal* result = vp_decimal_create();
    if (!result) return NULL;
    
    result->hi = d->hi;
    result->lo = d->lo;
    decimal_set_scale(result, scale);
    
    return result;
}

ViperDecimal* vp_decimal_round(ViperDecimal* d, int places) {
    if (!d) return NULL;
    
    /* Simplified implementation */
    return vp_decimal_quantize(d, places);
}

ViperDecimal* vp_decimal_floor(ViperDecimal* d) {
    if (!d) return NULL;
    
    double val = vp_decimal_to_f64(d);
    double floored = floor(val);
    return vp_decimal_from_f64(floored);
}

ViperDecimal* vp_decimal_ceil(ViperDecimal* d) {
    if (!d) return NULL;
    
    double val = vp_decimal_to_f64(d);
    double ceiled = ceil(val);
    return vp_decimal_from_f64(ceiled);
}

/* ============================================ */
/* Properties                                   */
/* ============================================ */

int64_t vp_decimal_get_sign(ViperDecimal* d) {
    return d ? decimal_get_sign(d) : 0;
}

int64_t vp_decimal_get_scale(ViperDecimal* d) {
    return d ? decimal_get_scale(d) : 0;
}

int64_t vp_decimal_is_zero(ViperDecimal* d) {
    return d && d->lo == 0 ? 1 : 0;
}

int64_t vp_decimal_is_nan(ViperDecimal* d) {
    /* Simplified: no NaN support yet */
    (void)d;
    return 0;
}

int64_t vp_decimal_is_infinite(ViperDecimal* d) {
    /* Simplified: no infinity support yet */
    (void)d;
    return 0;
}

int64_t vp_decimal_is_signed(ViperDecimal* d) {
    return d ? decimal_get_sign(d) : 0;
}

/* ============================================ */
/* Constants                                    */
/* ============================================ */

ViperDecimal* vp_decimal_zero(void) {
    return vp_decimal_from_i64(0);
}

ViperDecimal* vp_decimal_one(void) {
    return vp_decimal_from_i64(1);
}

ViperDecimal* vp_decimal_pi(void) {
    return vp_decimal_from_str("3.141592653589793238462643383279");
}

ViperDecimal* vp_decimal_e(void) {
    return vp_decimal_from_str("2.718281828459045235360287471352");
}

/* ============================================ */
/* Context (for future precision control)       */
/* ============================================ */

typedef struct ViperDecimalContext {
    int precision;
    int rounding;
    int trap_errors;
} ViperDecimalContext;

ViperDecimalContext* vp_decimal_context_create(void) {
    ViperDecimalContext* ctx = (ViperDecimalContext*)vp_arc_alloc(sizeof(ViperDecimalContext));
    if (!ctx) return NULL;
    
    ctx->precision = 28;
    ctx->rounding = 4;  /* ROUND_HALF_EVEN */
    ctx->trap_errors = 1;
    
    return ctx;
}

void vp_decimal_context_free(ViperDecimalContext* ctx) {
    if (ctx) vp_arc_release(ctx);
}

void vp_decimal_set_context(ViperDecimalContext* ctx) {
    /* Would set global context */
    (void)ctx;
}

ViperDecimalContext* vp_decimal_get_context(void) {
    static ViperDecimalContext default_ctx = {28, 4, 1};
    return &default_ctx;
}
