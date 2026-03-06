/**
 * Viper Tagged Integer Implementation
 * 
 * Provides automatic promotion from small integers to BigInt on overflow.
 */

#include "tagged_int.h"
#include <stdlib.h>
#include <string.h>

/* ============================================ */
/* Internal Helper Functions                    */
/* ============================================ */

/**
 * Allocate a new BigInt for TaggedInt promotion
 */
static ViperBigInt* alloc_bigint_for_tagged(void) {
    /* Use malloc directly for standalone usage */
    ViperBigInt* bigint = (ViperBigInt*)malloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;
    
    /* Initialize GMP value */
    mpz_init(bigint->value);
    return bigint;
}

/* ============================================ */
/* TaggedInt Core Operations                    */
/* ============================================ */

TaggedInt tagged_int_promote_to_bigint(TaggedInt value) {
    if (tagged_int_is_bigint(value)) {
        return value;  /* Already BigInt */
    }
    
    int64_t small_val = tagged_int_get_small(value);
    ViperBigInt* bigint = alloc_bigint_for_tagged();
    if (!bigint) {
        /* Fallback: return as small int if allocation fails */
        return value;
    }
    
    mpz_set_si(bigint->value, small_val);
    return tagged_int_from_bigint(bigint);
}

ViperBigInt* tagged_int_to_bigint(TaggedInt value) {
    if (tagged_int_is_bigint(value)) {
        return tagged_int_get_bigint(value);
    }
    
    int64_t small_val = tagged_int_get_small(value);
    ViperBigInt* bigint = alloc_bigint_for_tagged();
    if (!bigint) {
        return NULL;
    }
    
    mpz_set_si(bigint->value, small_val);
    return bigint;
}

/* ============================================ */
/* Arithmetic Operations                        */
/* ============================================ */

/**
 * Free a temporary BigInt created by tagged_int_to_bigint
 */
static void free_temp_bigint(ViperBigInt* bigint) {
    if (bigint) {
        mpz_clear(bigint->value);
        free(bigint);
    }
}

TaggedInt tagged_int_add(TaggedInt a, TaggedInt b) {
    /* Case 1: Both small integers */
    if (tagged_int_is_small(a) && tagged_int_is_small(b)) {
        int64_t a_val = tagged_int_get_small(a);
        int64_t b_val = tagged_int_get_small(b);

        /* Check for overflow */
        if (would_overflow_add(a_val, b_val)) {
            /* Promote both to BigInt and add */
            ViperBigInt* a_big = tagged_int_to_bigint(a);
            ViperBigInt* b_big = tagged_int_to_bigint(b);
            ViperBigInt* result = alloc_bigint_for_tagged();

            if (result) {
                mpz_add(result->value, a_big->value, b_big->value);
                free_temp_bigint(a_big);
                free_temp_bigint(b_big);
                return tagged_int_from_bigint(result);
            }
            /* If allocation fails, return truncated result */
        }

        /* No overflow - return small int result */
        return tagged_int_from_i64(a_val + b_val);
    }

    /* Case 2: At least one BigInt */
    ViperBigInt* a_big = tagged_int_is_small(a) ? tagged_int_to_bigint(a) : tagged_int_get_bigint(a);
    ViperBigInt* b_big = tagged_int_is_small(b) ? tagged_int_to_bigint(b) : tagged_int_get_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();

    if (result) {
        mpz_add(result->value, a_big->value, b_big->value);
    }

    /* Free temporaries (only the ones we created, not the original BigInts) */
    if (tagged_int_is_small(a)) free_temp_bigint(a_big);
    if (tagged_int_is_small(b)) free_temp_bigint(b_big);

    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

TaggedInt tagged_int_sub(TaggedInt a, TaggedInt b) {
    /* Case 1: Both small integers */
    if (tagged_int_is_small(a) && tagged_int_is_small(b)) {
        int64_t a_val = tagged_int_get_small(a);
        int64_t b_val = tagged_int_get_small(b);
        
        /* Check for overflow */
        if (would_overflow_sub(a_val, b_val)) {
            /* Promote both to BigInt and subtract */
            ViperBigInt* a_big = tagged_int_to_bigint(a);
            ViperBigInt* b_big = tagged_int_to_bigint(b);
            ViperBigInt* result = alloc_bigint_for_tagged();
            
            if (result) {
                mpz_sub(result->value, a_big->value, b_big->value);
                vp_arc_release(a_big);
                vp_arc_release(b_big);
                return tagged_int_from_bigint(result);
            }
        }
        
        return tagged_int_from_i64(a_val - b_val);
    }
    
    /* Case 2: At least one BigInt */
    ViperBigInt* a_big = tagged_int_to_bigint(a);
    ViperBigInt* b_big = tagged_int_to_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();
    
    if (result) {
        mpz_sub(result->value, a_big->value, b_big->value);
    }
    
    vp_arc_release(a_big);
    vp_arc_release(b_big);
    
    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

TaggedInt tagged_int_mul(TaggedInt a, TaggedInt b) {
    /* Case 1: Both small integers */
    if (tagged_int_is_small(a) && tagged_int_is_small(b)) {
        int64_t a_val = tagged_int_get_small(a);
        int64_t b_val = tagged_int_get_small(b);
        
        /* Check for overflow */
        if (would_overflow_mul(a_val, b_val)) {
            /* Promote both to BigInt and multiply */
            ViperBigInt* a_big = tagged_int_to_bigint(a);
            ViperBigInt* b_big = tagged_int_to_bigint(b);
            ViperBigInt* result = alloc_bigint_for_tagged();
            
            if (result) {
                mpz_mul(result->value, a_big->value, b_big->value);
                vp_arc_release(a_big);
                vp_arc_release(b_big);
                return tagged_int_from_bigint(result);
            }
        }
        
        return tagged_int_from_i64(a_val * b_val);
    }
    
    /* Case 2: At least one BigInt */
    ViperBigInt* a_big = tagged_int_to_bigint(a);
    ViperBigInt* b_big = tagged_int_to_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();
    
    if (result) {
        mpz_mul(result->value, a_big->value, b_big->value);
    }
    
    vp_arc_release(a_big);
    vp_arc_release(b_big);
    
    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

TaggedInt tagged_int_div(TaggedInt a, TaggedInt b) {
    ViperBigInt* a_big = tagged_int_to_bigint(a);
    ViperBigInt* b_big = tagged_int_to_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();
    
    if (result) {
        if (mpz_sgn(b_big->value) == 0) {
            fprintf(stderr, "Error: Division by zero\n");
            mpz_set_ui(result->value, 0);
        } else {
            mpz_tdiv_q(result->value, a_big->value, b_big->value);
        }
    }
    
    vp_arc_release(a_big);
    vp_arc_release(b_big);
    
    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

TaggedInt tagged_int_mod(TaggedInt a, TaggedInt b) {
    ViperBigInt* a_big = tagged_int_get_bigint(a);
    ViperBigInt* b_big = tagged_int_to_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();

    if (result) {
        if (mpz_sgn(b_big->value) == 0) {
            fprintf(stderr, "Error: Modulo by zero\n");
            mpz_set_ui(result->value, 0);
        } else {
            mpz_tdiv_r(result->value, a_big->value, b_big->value);
        }
    }

    free_temp_bigint(b_big);

    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

/* ============================================ */
/* Comparison Operations                        */
/* ============================================ */

int tagged_int_cmp(TaggedInt a, TaggedInt b) {
    /* Case 1: Both small integers */
    if (tagged_int_is_small(a) && tagged_int_is_small(b)) {
        int64_t a_val = tagged_int_get_small(a);
        int64_t b_val = tagged_int_get_small(b);
        if (a_val < b_val) return -1;
        if (a_val > b_val) return 1;
        return 0;
    }

    /* Case 2: At least one BigInt */
    ViperBigInt* a_big = tagged_int_get_bigint(a);
    ViperBigInt* b_big = tagged_int_to_bigint(b);
    int result = mpz_cmp(a_big->value, b_big->value);
    free_temp_bigint(b_big);

    return result;
}

bool tagged_int_eq(TaggedInt a, TaggedInt b) {
    return tagged_int_cmp(a, b) == 0;
}

bool tagged_int_lt(TaggedInt a, TaggedInt b) {
    return tagged_int_cmp(a, b) < 0;
}

bool tagged_int_gt(TaggedInt a, TaggedInt b) {
    return tagged_int_cmp(a, b) > 0;
}

/* ============================================ */
/* Unary Operations                             */
/* ============================================ */

TaggedInt tagged_int_neg(TaggedInt a) {
    if (tagged_int_is_small(a)) {
        int64_t val = tagged_int_get_small(a);
        /* Check for negation overflow (INT_MIN) */
        if (val == TAGGED_INT_MIN_SMALL) {
            /* Promote to BigInt */
            ViperBigInt* bigint = alloc_bigint_for_tagged();
            if (bigint) {
                mpz_set_si(bigint->value, val);
                mpz_neg(bigint->value, bigint->value);
                return tagged_int_from_bigint(bigint);
            }
        }
        return tagged_int_from_i64(-val);
    }
    
    /* BigInt case */
    ViperBigInt* a_big = tagged_int_get_bigint(a);
    ViperBigInt* result = alloc_bigint_for_tagged();
    
    if (result) {
        mpz_neg(result->value, a_big->value);
    }
    
    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

/* ============================================ */
/* Utility Functions                            */
/* ============================================ */

char* tagged_int_to_str(TaggedInt value) {
    bool is_temp = false;
    ViperBigInt* bigint;
    
    if (tagged_int_is_bigint(value)) {
        bigint = tagged_int_get_bigint(value);
    } else {
        bigint = tagged_int_to_bigint(value);
        is_temp = true;
    }
    
    if (!bigint) return NULL;

    char* str = mpz_get_str(NULL, 10, bigint->value);

    /* Free the temporary bigint we created (if any) */
    if (is_temp) {
        mpz_clear(bigint->value);
        free(bigint);
    }

    return str;
}

void tagged_int_print(TaggedInt value) {
    char* str = tagged_int_to_str(value);
    if (str) {
        printf("%s", str);
        free(str);
    }
}

void tagged_int_free(TaggedInt value) {
    if (tagged_int_is_bigint(value)) {
        ViperBigInt* bigint = tagged_int_get_bigint(value);
        /* Sanity check: pointer should be in valid memory range */
        /* Valid heap pointers are typically in lower address space */
        if (bigint != NULL && ((uint64_t)bigint >> 48) == 0) {
            mpz_clear(bigint->value);
            free(bigint);
        }
    }
    /* Small integers don't need freeing */
}
