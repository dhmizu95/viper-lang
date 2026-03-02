/**
 * Viper GMP Bridge Implementation
 * 
 * Arbitrary-precision integer operations using GMP
 * Integrated with Viper's ARC memory management
 */

#include "gmp_bridge.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ============================================ */
/* Core BigInt Operations                       */
/* ============================================ */

ViperBigInt* vp_bigint_from_str(const char* str) {
    if (!str) return NULL;

    ViperBigInt* bigint = (ViperBigInt*)vp_arc_alloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;

    /* Set destructor for ARC cleanup */
    vp_arc_set_destructor(bigint, (void (*)(void*))vp_bigint_destroy);

    /* Initialize GMP value */
    mpz_init(bigint->value);

    /* Determine base from prefix */
    int base = 10;  /* Default to decimal */
    const char* actual_str = str;
    
    if (str[0] == '0' && str[1] != '\0') {
        if (str[1] == 'x' || str[1] == 'X') {
            base = 16;
            actual_str = str + 2;  /* Skip 0x prefix */
        } else if (str[1] == 'o' || str[1] == 'O') {
            base = 8;
            actual_str = str + 2;  /* Skip 0o prefix */
        } else if (str[1] == 'b' || str[1] == 'B') {
            base = 2;
            actual_str = str + 2;  /* Skip 0b prefix */
        } else if (str[1] >= '0' && str[1] <= '7') {
            /* Legacy octal notation (just 0 prefix) */
            base = 8;
            actual_str = str + 1;  /* Skip leading 0 */
        }
    }

    if (mpz_set_str(bigint->value, actual_str, base) != 0) {
        vp_arc_release(bigint); /* This will call destructor and free memory */
        return NULL;
    }

    return bigint;
}

ViperBigInt* vp_bigint_from_i64(int64_t value) {
    ViperBigInt* bigint = (ViperBigInt*)vp_arc_alloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;

    /* Set destructor for ARC cleanup */
    vp_arc_set_destructor(bigint, (void (*)(void*))vp_bigint_destroy);

    /* Initialize and set GMP value */
    mpz_init_set_si(bigint->value, value);

    return bigint;
}

/**
 * Create a BigInt from i64 for internal operation results
 * Returns a BigInt with ref_count=1 (standard allocation)
 * The caller takes ownership - do NOT call retain when assigning
 */
ViperBigInt* vp_bigint_from_i64_temp(int64_t value) {
    /* Just use the standard allocation - caller takes ownership */
    return vp_bigint_from_i64(value);
}

ViperBigInt* vp_bigint_from_u64(uint64_t value) {
    ViperBigInt* bigint = (ViperBigInt*)vp_arc_alloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;
    
    /* Set destructor for ARC cleanup */
    vp_arc_set_destructor(bigint, (void (*)(void*))vp_bigint_destroy);
    
    /* Initialize and set GMP value */
    mpz_init_set_ui(bigint->value, value);
    
    return bigint;
}

void vp_bigint_destroy(ViperBigInt* bigint) {
    if (!bigint) return;
    
    /* Clear GMP resources */
    mpz_clear(bigint->value);
}

char* vp_bigint_to_str(ViperBigInt* bigint, int base) {
    if (!bigint) return NULL;
    if (base < 2 || base > 62) return NULL;

    /* Get string representation from GMP */
    char* gmp_str = mpz_get_str(NULL, base, bigint->value);
    if (!gmp_str) return NULL;
    
    /* Copy to standard malloc'd string so caller can free() it */
    char* str = strdup(gmp_str);
    
    /* Free GMP's internal allocation */
    free(gmp_str);
    
    return str;  /* Caller must free with free() */
}

int64_t vp_bigint_to_i64(ViperBigInt* bigint) {
    if (!bigint) return 0;
    return mpz_get_si(bigint->value);
}

void vp_bigint_abs(ViperBigInt* result, ViperBigInt* operand) {
    if (!result || !operand) return;
    mpz_abs(result->value, operand->value);
}

void vp_bigint_neg(ViperBigInt* result, ViperBigInt* operand) {
    if (!result || !operand) return;
    mpz_neg(result->value, operand->value);
}

/* ============================================ */
/* Arithmetic Operations                        */
/* ============================================ */

void vp_bigint_add(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_add(result->value, a->value, b->value);
}

void vp_bigint_sub(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_sub(result->value, a->value, b->value);
}

void vp_bigint_mul(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_mul(result->value, a->value, b->value);
}

void vp_bigint_div(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    /* Check for division by zero */
    if (mpz_sgn(b->value) == 0) {
        fprintf(stderr, "Error: Division by zero in BigInt division\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    mpz_tdiv_q(result->value, a->value, b->value);
}

void vp_bigint_mod(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    /* Check for division by zero */
    if (mpz_sgn(b->value) == 0) {
        fprintf(stderr, "Error: Modulo by zero in BigInt operation\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    mpz_tdiv_r(result->value, a->value, b->value);
}

void vp_bigint_divmod(ViperBigInt* quotient, ViperBigInt* remainder, 
                      ViperBigInt* a, ViperBigInt* b) {
    if (!quotient || !remainder || !a || !b) return;
    /* Check for division by zero */
    if (mpz_sgn(b->value) == 0) {
        fprintf(stderr, "Error: Division by zero in BigInt divmod\n");
        mpz_set_ui(quotient->value, 0);
        mpz_set_ui(remainder->value, 0);
        return;
    }
    mpz_tdiv_qr(quotient->value, remainder->value, a->value, b->value);
}

void vp_bigint_pow(ViperBigInt* result, ViperBigInt* base, ViperBigInt* exp) {
    if (!result || !base || !exp) return;
    
    /* Check for negative exponent */
    if (mpz_sgn(exp->value) < 0) {
        fprintf(stderr, "Error: Negative exponent in BigInt power\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    
    /* Convert exponent to unsigned long if possible */
    if (mpz_fits_ulong_p(exp->value)) {
        mpz_pow_ui(result->value, base->value, mpz_get_ui(exp->value));
    } else {
        /* Use mpz_powm for large exponents (mod 0 = no modulus) */
        mpz_t temp_mod;
        mpz_init_set_ui(temp_mod, 0);
        mpz_powm(result->value, base->value, exp->value, temp_mod);
        mpz_clear(temp_mod);
    }
}

void vp_bigint_sqrt(ViperBigInt* result, ViperBigInt* a) {
    if (!result || !a) return;
    /* Check for negative operand */
    if (mpz_sgn(a->value) < 0) {
        fprintf(stderr, "Error: Square root of negative number in BigInt\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    mpz_sqrt(result->value, a->value);
}

/* ============================================ */
/* Mixed Arithmetic (BigInt + native types)     */
/* ============================================ */

void vp_bigint_add_i64(ViperBigInt* result, ViperBigInt* a, int64_t b) {
    if (!result || !a) return;
    
    if (b >= 0) {
        mpz_add_ui(result->value, a->value, (unsigned long)b);
    } else {
        mpz_sub_ui(result->value, a->value, (unsigned long)(-b));
    }
}

void vp_bigint_sub_i64(ViperBigInt* result, ViperBigInt* a, int64_t b) {
    if (!result || !a) return;
    
    if (b >= 0) {
        mpz_sub_ui(result->value, a->value, (unsigned long)b);
    } else {
        mpz_add_ui(result->value, a->value, (unsigned long)(-b));
    }
}

void vp_bigint_mul_i64(ViperBigInt* result, ViperBigInt* a, int64_t b) {
    if (!result || !a) return;
    
    if (b >= 0) {
        mpz_mul_ui(result->value, a->value, (unsigned long)b);
    } else {
        mpz_mul_si(result->value, a->value, b);
    }
}

void vp_bigint_div_i64(ViperBigInt* result, ViperBigInt* a, int64_t b) {
    if (!result || !a) return;
    
    if (b == 0) {
        fprintf(stderr, "Error: Division by zero in BigInt/i64 division\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    
    if (b >= 0) {
        mpz_tdiv_q_ui(result->value, a->value, (unsigned long)b);
    } else {
        // For negative divisor, use absolute value then negate
        mpz_tdiv_q_ui(result->value, a->value, (unsigned long)(-b));
        mpz_neg(result->value, result->value);
    }
}

/* ============================================ */
/* Bitwise Operations                           */
/* ============================================ */

void vp_bigint_and(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_and(result->value, a->value, b->value);
}

void vp_bigint_or(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_ior(result->value, a->value, b->value);
}

void vp_bigint_xor(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_xor(result->value, a->value, b->value);
}

void vp_bigint_lshift(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    
    /* Check for negative shift */
    if (mpz_sgn(b->value) < 0) {
        fprintf(stderr, "Error: Negative shift in BigInt lshift\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    
    /* Convert shift amount to unsigned long if possible */
    if (mpz_fits_ulong_p(b->value)) {
        mpz_mul_2exp(result->value, a->value, mpz_get_ui(b->value));
    } else {
        /* For very large shifts, use mpz_pow_2exp and multiply */
        mpz_t shift_factor;
        mpz_init(shift_factor);
        mpz_ui_pow_ui(shift_factor, 2, mpz_get_ui(b->value));
        mpz_mul(result->value, a->value, shift_factor);
        mpz_clear(shift_factor);
    }
}

void vp_bigint_invert(ViperBigInt* result, ViperBigInt* a) {
    if (!result || !a) return;
    mpz_com(result->value, a->value);
}

void vp_bigint_rshift(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    
    /* Check for negative shift */
    if (mpz_sgn(b->value) < 0) {
        fprintf(stderr, "Error: Negative shift in BigInt rshift\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    
    /* Convert shift amount to unsigned long if possible */
    if (mpz_fits_ulong_p(b->value)) {
        mpz_tdiv_q_2exp(result->value, a->value, mpz_get_ui(b->value));
    } else {
        /* For very large shifts, result is 0 */
        mpz_set_ui(result->value, 0);
    }
}

/* ============================================ */
/* Comparison Operations                        */
/* ============================================ */

int vp_bigint_cmp(ViperBigInt* a, ViperBigInt* b) {
    if (!a || !b) return 0;
    return mpz_cmp(a->value, b->value);
}

int vp_bigint_cmp_i64(ViperBigInt* a, int64_t b) {
    if (!a) return 0;
    return mpz_cmp_si(a->value, b);
}

bool vp_bigint_eq(ViperBigInt* a, ViperBigInt* b) {
    if (!a || !b) return false;
    return mpz_cmp(a->value, b->value) == 0;
}

bool vp_bigint_lt(ViperBigInt* a, ViperBigInt* b) {
    if (!a || !b) return false;
    return mpz_cmp(a->value, b->value) < 0;
}

bool vp_bigint_gt(ViperBigInt* a, ViperBigInt* b) {
    if (!a || !b) return false;
    return mpz_cmp(a->value, b->value) > 0;
}

bool vp_bigint_is_zero(ViperBigInt* a) {
    if (!a) return true;
    return mpz_sgn(a->value) == 0;
}

bool vp_bigint_is_negative(ViperBigInt* a) {
    if (!a) return false;
    return mpz_sgn(a->value) < 0;
}

/* ============================================ */
/* Utility Operations                           */
/* ============================================ */

size_t vp_bigint_bit_length(ViperBigInt* a) {
    if (!a) return 0;
    return mpz_sizeinbase(a->value, 2);
}

int vp_bigint_sign(ViperBigInt* a) {
    if (!a) return 0;
    return mpz_sgn(a->value);
}

void vp_bigint_copy(ViperBigInt* dest, ViperBigInt* src) {
    if (!dest || !src) return;
    mpz_set(dest->value, src->value);
}

uint64_t vp_bigint_hash(ViperBigInt* a) {
    if (!a) return 0;
    
    /* Simple hash: combine size and low bits */
    uint64_t hash = 0;
    size_t size = mpz_size(a->value);
    
    /* Mix in the size */
    hash = size * 0x9e3779b97f4a7c15ULL;
    
    /* Mix in the low 64 bits */
    if (size > 0) {
        uint64_t low_bits = mpz_get_ui(a->value);
        hash ^= low_bits;
        hash *= 0x100000001b3ULL;
    }
    
    /* Mix in the sign */
    if (mpz_sgn(a->value) < 0) {
        hash ^= 0x8000000000000000ULL;
    }
    
    /* Final mixing */
    hash ^= hash >> 33;
    hash *= 0xff51afd7ed558ccdULL;
    hash ^= hash >> 33;
    hash *= 0xc4ceb9fe1a85ec53ULL;
    hash ^= hash >> 33;
    
    return hash;
}

void vp_bigint_min(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    if (mpz_cmp(a->value, b->value) <= 0) {
        mpz_set(result->value, a->value);
    } else {
        mpz_set(result->value, b->value);
    }
}

void vp_bigint_max(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    if (mpz_cmp(a->value, b->value) >= 0) {
        mpz_set(result->value, a->value);
    } else {
        mpz_set(result->value, b->value);
    }
}

/* ============================================ */
/* Math Operations                              */
/* ============================================ */

void vp_bigint_gcd(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_gcd(result->value, a->value, b->value);
}

void vp_bigint_lcm(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    if (!result || !a || !b) return;
    mpz_lcm(result->value, a->value, b->value);
}

void vp_bigint_factorial(ViperBigInt* result, ViperBigInt* n) {
    if (!result || !n) return;
    if (mpz_sgn(n->value) < 0) {
        fprintf(stderr, "Error: factorial of negative number\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    if (mpz_fits_ulong_p(n->value)) {
        mpz_fac_ui(result->value, mpz_get_ui(n->value));
    } else {
        fprintf(stderr, "Error: factorial input too large\n");
        mpz_set_ui(result->value, 0);
    }
}

void vp_bigint_comb(ViperBigInt* result, ViperBigInt* n, ViperBigInt* k) {
    if (!result || !n || !k) return;
    if (mpz_sgn(n->value) < 0 || mpz_sgn(k->value) < 0) {
        mpz_set_ui(result->value, 0);
        return;
    }
    if (mpz_cmp(k->value, n->value) > 0) {
        mpz_set_ui(result->value, 0);
        return;
    }
    if (mpz_fits_ulong_p(k->value)) {
        mpz_bin_ui(result->value, n->value, mpz_get_ui(k->value));
    } else {
        mpz_t n_minus_k;
        mpz_init(n_minus_k);
        mpz_sub(n_minus_k, n->value, k->value);
        if (mpz_fits_ulong_p(n_minus_k)) {
            mpz_bin_ui(result->value, n->value, mpz_get_ui(n_minus_k));
        } else {
            fprintf(stderr, "Error: comb k too large\n");
            mpz_set_ui(result->value, 0);
        }
        mpz_clear(n_minus_k);
    }
}

void vp_bigint_perm(ViperBigInt* result, ViperBigInt* n, ViperBigInt* k) {
    if (!result || !n || !k) return;
    if (mpz_sgn(n->value) < 0 || mpz_sgn(k->value) < 0 || mpz_cmp(k->value, n->value) > 0) {
        mpz_set_ui(result->value, 0);
        return;
    }
    if (mpz_fits_ulong_p(k->value)) {
        unsigned long k_ul = mpz_get_ui(k->value);
        mpz_set_ui(result->value, 1);
        mpz_t temp_n;
        mpz_init_set(temp_n, n->value);
        for (unsigned long i = 0; i < k_ul; ++i) {
            mpz_mul(result->value, result->value, temp_n);
            mpz_sub_ui(temp_n, temp_n, 1);
        }
        mpz_clear(temp_n);
    } else {
        fprintf(stderr, "Error: perm k too large\n");
        mpz_set_ui(result->value, 0);
    }
}

void vp_bigint_powmod(ViperBigInt* result, ViperBigInt* base, ViperBigInt* exp, ViperBigInt* mod) {
    if (!result || !base || !exp || !mod) return;
    if (mpz_sgn(exp->value) < 0) {
        fprintf(stderr, "Error: powmod negative exponent\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    if (mpz_sgn(mod->value) == 0) {
        fprintf(stderr, "Error: powmod division by zero\n");
        mpz_set_ui(result->value, 0);
        return;
    }
    mpz_powm(result->value, base->value, exp->value, mod->value);
}

/* ============================================ */
/* JIT Stub Aliases (_c suffix for Rust FFI)    */
/* ============================================ */
/* These are aliases for the JIT stub registry in Rust */

ViperBigInt* vp_bigint_from_str_c(const char* s) {
    return vp_bigint_from_str(s);
}

ViperBigInt* vp_bigint_from_i64_c(int64_t v) {
    return vp_bigint_from_i64(v);
}

const char* vp_bigint_to_str_c(ViperBigInt* bigint, int base) {
    return vp_bigint_to_str(bigint, base);
}

void vp_bigint_add_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_add(result, a, b);
}

void vp_bigint_sub_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_sub(result, a, b);
}

void vp_bigint_mul_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_mul(result, a, b);
}

void vp_bigint_div_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_div(result, a, b);
}

void vp_bigint_mod_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_mod(result, a, b);
}

void vp_bigint_pow_c(ViperBigInt* result, ViperBigInt* base, ViperBigInt* exp) {
    vp_bigint_pow(result, base, exp);
}

void vp_bigint_sqrt_c(ViperBigInt* result, ViperBigInt* a) {
    vp_bigint_sqrt(result, a);
}

void vp_bigint_abs_c(ViperBigInt* result, ViperBigInt* a) {
    vp_bigint_abs(result, a);
}

void vp_bigint_neg_c(ViperBigInt* result, ViperBigInt* a) {
    vp_bigint_neg(result, a);
}

void vp_bigint_invert_c(ViperBigInt* result, ViperBigInt* a) {
    vp_bigint_invert(result, a);
}

void vp_bigint_and_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_and(result, a, b);
}

void vp_bigint_or_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_or(result, a, b);
}

void vp_bigint_xor_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_xor(result, a, b);
}

void vp_bigint_lshift_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_lshift(result, a, b);
}

void vp_bigint_rshift_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_rshift(result, a, b);
}

bool vp_bigint_eq_c(ViperBigInt* a, ViperBigInt* b) {
    return vp_bigint_eq(a, b);
}

bool vp_bigint_lt_c(ViperBigInt* a, ViperBigInt* b) {
    return vp_bigint_lt(a, b);
}

bool vp_bigint_gt_c(ViperBigInt* a, ViperBigInt* b) {
    return vp_bigint_gt(a, b);
}

void vp_bigint_min_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_min(result, a, b);
}

void vp_bigint_max_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_max(result, a, b);
}

void vp_bigint_gcd_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_gcd(result, a, b);
}

void vp_bigint_lcm_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b) {
    vp_bigint_lcm(result, a, b);
}

void vp_bigint_factorial_c(ViperBigInt* result, ViperBigInt* n) {
    vp_bigint_factorial(result, n);
}

void vp_bigint_comb_c(ViperBigInt* result, ViperBigInt* n, ViperBigInt* k) {
    vp_bigint_comb(result, n, k);
}

void vp_bigint_perm_c(ViperBigInt* result, ViperBigInt* n, ViperBigInt* k) {
    vp_bigint_perm(result, n, k);
}

void vp_bigint_powmod_c(ViperBigInt* result, ViperBigInt* base, ViperBigInt* exp, ViperBigInt* mod) {
    vp_bigint_powmod(result, base, exp, mod);
}
