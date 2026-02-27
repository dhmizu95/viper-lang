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
    
    ViperBigInt* bigint = (ViperBigInt*)malloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;
    
    /* Initialize ARC header */
    bigint->ref_count = 1;
    bigint->destructor = (void (*)(void*))vp_bigint_destroy;
    bigint->flags = 0;
    memset(bigint->reserved, 0, sizeof(bigint->reserved));
    
    /* Initialize GMP value */
    mpz_init(bigint->value);
    
    /* Parse string (auto-detect base: 0x for hex, 0b for binary, etc.) */
    if (mpz_set_str(bigint->value, str, 0) != 0) {
        mpz_clear(bigint->value);
        free(bigint);
        return NULL;
    }
    
    return bigint;
}

ViperBigInt* vp_bigint_from_i64(int64_t value) {
    ViperBigInt* bigint = (ViperBigInt*)malloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;
    
    /* Initialize ARC header */
    bigint->ref_count = 1;
    bigint->destructor = (void (*)(void*))vp_bigint_destroy;
    bigint->flags = 0;
    memset(bigint->reserved, 0, sizeof(bigint->reserved));
    
    /* Initialize and set GMP value */
    mpz_init_set_si(bigint->value, value);
    
    return bigint;
}

ViperBigInt* vp_bigint_from_u64(uint64_t value) {
    ViperBigInt* bigint = (ViperBigInt*)malloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;
    
    /* Initialize ARC header */
    bigint->ref_count = 1;
    bigint->destructor = (void (*)(void*))vp_bigint_destroy;
    bigint->flags = 0;
    memset(bigint->reserved, 0, sizeof(bigint->reserved));
    
    /* Initialize and set GMP value */
    mpz_init_set_ui(bigint->value, value);
    
    return bigint;
}

void vp_bigint_destroy(ViperBigInt* bigint) {
    if (!bigint) return;
    
    /* Clear GMP value */
    mpz_clear(bigint->value);
    
    /* Free memory */
    free(bigint);
}

char* vp_bigint_to_str(ViperBigInt* bigint, int base) {
    if (!bigint) return NULL;
    if (base < 2 || base > 62) return NULL;
    
    /* Get string representation from GMP */
    char* str = mpz_get_str(NULL, base, bigint->value);
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
        mpz_tdiv_q_si(result->value, a->value, b);
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
