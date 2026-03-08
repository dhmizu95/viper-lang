/**
 * Viper Tagged Integer Implementation
 *
 * Provides automatic promotion from small integers to BigInt on overflow.
 */

#include "tagged_int.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* Forward declare ViperString functions we need */
typedef struct ViperString ViperString;

/* ============================================ */
/* Internal Helper Functions                    */
/* ============================================ */

/**
 * Check if a value fits in i63 range
 */
static inline bool fits_in_i63(int64_t value) {
    return value >= TAGGED_INT_MIN_SMALL && value <= TAGGED_INT_MAX_SMALL;
}

/**
 * Allocate a new BigInt for TaggedInt promotion
 */
static ViperBigInt* alloc_bigint_for_tagged(void) {
    /* Use ARC allocation for proper memory management */
    ViperBigInt* bigint = (ViperBigInt*)malloc(sizeof(ViperBigInt));
    if (!bigint) return NULL;

    /* Initialize GMP value */
    mpz_init(bigint->value);
    
    /* Set destructor for ARC cleanup */
    
    return bigint;
}

/**
 * Free a BigInt allocated by alloc_bigint_for_tagged
 * This properly cleans up GMP resources and frees memory via ARC
 */
static void free_bigint_for_tagged(ViperBigInt* bigint) {
    if (bigint) {
        mpz_clear(bigint->value);
        free(bigint);
    }
}

/**
 * Try to demote a BigInt value back to SmallInt if it fits
 * Returns 0 if demotion is not possible, otherwise returns the SmallInt TaggedInt
 * 
 * This enables Python-like smooth integer behavior where values automatically
 * return to efficient SmallInt representation when they fit.
 */
static inline TaggedInt try_demote_bigint(mpz_t value) {
    /* Check if the value fits in a SmallInt (i63) */
    if (mpz_fits_slong_p(value)) {
        int64_t small = mpz_get_si(value);
        /* Double-check bounds to be safe */
        if (small >= TAGGED_INT_MIN_SMALL && small <= TAGGED_INT_MAX_SMALL) {
            return tagged_int_from_i64(small);
        }
    }
    return 0;  /* Cannot demote */
}

/* ============================================ */
/* TaggedInt Core Operations                    */
/* ============================================ */

/**
 * Create a TaggedInt from a string representation
 */
TaggedInt tagged_int_from_str(const char* str) {
    if (!str) return tagged_int_from_i64(0);

    ViperBigInt* bigint = alloc_bigint_for_tagged();
    if (!bigint) return tagged_int_from_i64(0);

    /* Parse string into BigInt */
    int base = 10;
    const char* actual_str = str;

    if (str[0] == '0' && str[1] != '\0') {
        if (str[1] == 'x' || str[1] == 'X') {
            base = 16;
            actual_str = str + 2;
        } else if (str[1] == 'o' || str[1] == 'O') {
            base = 8;
            actual_str = str + 2;
        } else if (str[1] == 'b' || str[1] == 'B') {
            base = 2;
            actual_str = str + 2;
        } else if (str[1] >= '0' && str[1] <= '7') {
            base = 8;
            actual_str = str + 1;
        }
    }

    if (mpz_set_str(bigint->value, actual_str, base) != 0) {
        mpz_clear(bigint->value);
        free(bigint);
        return tagged_int_from_i64(0);
    }

    /* Check if the value fits in SmallInt */
    if (mpz_fits_slong_p(bigint->value)) {
        int64_t small_val = mpz_get_si(bigint->value);
        if (small_val >= TAGGED_INT_MIN_SMALL && small_val <= TAGGED_INT_MAX_SMALL) {
            mpz_clear(bigint->value);
            free(bigint);
            return tagged_int_from_i64(small_val);
        }
    }

    return tagged_int_from_bigint(bigint);
}

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
                
                /* Try to demote result back to SmallInt */
                TaggedInt demoted = try_demote_bigint(result->value);
                if (demoted != 0) {
                    // mpz_clear handled by ARC destructor
                    free_bigint_for_tagged(result);
                    return demoted;
                }
                
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
        
        /* Try to demote result back to SmallInt */
        TaggedInt demoted = try_demote_bigint(result->value);
        if (demoted != 0) {
            // mpz_clear handled by ARC destructor
            free_bigint_for_tagged(result);
            /* Free temporaries */
            if (tagged_int_is_small(a)) free_temp_bigint(a_big);
            if (tagged_int_is_small(b)) free_temp_bigint(b_big);
            return demoted;
        }
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
                
                /* Try to demote result back to SmallInt */
                TaggedInt demoted = try_demote_bigint(result->value);
                if (demoted != 0) {
                    // mpz_clear handled by ARC destructor
                    free_bigint_for_tagged(result);
                    free_temp_bigint(a_big);
                    free_temp_bigint(b_big);
                    return demoted;
                }
                
                free_temp_bigint(a_big);
                free_temp_bigint(b_big);
                return tagged_int_from_bigint(result);
            }
        }

        return tagged_int_from_i64(a_val - b_val);
    }

    /* Case 2: At least one BigInt */
    ViperBigInt* a_big = tagged_int_is_small(a) ? tagged_int_to_bigint(a) : tagged_int_get_bigint(a);
    ViperBigInt* b_big = tagged_int_is_small(b) ? tagged_int_to_bigint(b) : tagged_int_get_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();

    if (result) {
        mpz_sub(result->value, a_big->value, b_big->value);
        
        /* Try to demote result back to SmallInt */
        TaggedInt demoted = try_demote_bigint(result->value);
        if (demoted != 0) {
            // mpz_clear handled by ARC destructor
            free_bigint_for_tagged(result);
            /* Free temporaries */
            if (tagged_int_is_small(a)) free_temp_bigint(a_big);
            if (tagged_int_is_small(b)) free_temp_bigint(b_big);
            return demoted;
        }
    }

    /* Free temporaries (only the ones we created, not the original BigInts) */
    if (tagged_int_is_small(a)) free_temp_bigint(a_big);
    if (tagged_int_is_small(b)) free_temp_bigint(b_big);

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

                /* Try to demote result back to SmallInt */
                TaggedInt demoted = try_demote_bigint(result->value);
                if (demoted != 0) {
                    // mpz_clear handled by ARC destructor
                    free_bigint_for_tagged(result);
                    free_temp_bigint(a_big);
                    free_temp_bigint(b_big);
                    return demoted;
                }

                free_temp_bigint(a_big);
                free_temp_bigint(b_big);
                return tagged_int_from_bigint(result);
            }
        }

        return tagged_int_from_i64(a_val * b_val);
    }

    /* Case 2: At least one BigInt */
    ViperBigInt* a_big = tagged_int_is_small(a) ? tagged_int_to_bigint(a) : tagged_int_get_bigint(a);
    ViperBigInt* b_big = tagged_int_is_small(b) ? tagged_int_to_bigint(b) : tagged_int_get_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();

    if (result) {
        mpz_mul(result->value, a_big->value, b_big->value);

        /* Try to demote result back to SmallInt */
        TaggedInt demoted = try_demote_bigint(result->value);
        if (demoted != 0) {
            // mpz_clear handled by ARC destructor
            free_bigint_for_tagged(result);
            /* Free temporaries */
            if (tagged_int_is_small(a)) free_temp_bigint(a_big);
            if (tagged_int_is_small(b)) free_temp_bigint(b_big);
            return demoted;
        }
    }

    /* Free temporaries (only the ones we created, not the original BigInts) */
    if (tagged_int_is_small(a)) free_temp_bigint(a_big);
    if (tagged_int_is_small(b)) free_temp_bigint(b_big);

    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

TaggedInt tagged_int_div(TaggedInt a, TaggedInt b) {
    /* Case 1: Both small integers */
    if (tagged_int_is_small(a) && tagged_int_is_small(b)) {
        int64_t a_val = tagged_int_get_small(a);
        int64_t b_val = tagged_int_get_small(b);

        if (b_val != 0) {
            int64_t res = a_val / b_val;
            if (fits_in_i63(res)) {
                return tagged_int_from_i64(res);
            }
        } else {
            fprintf(stderr, "Error: Division by zero\n");
            return tagged_int_from_i64(0);
        }
    }

    /* Case 2: At least one BigInt */
    ViperBigInt* a_big = tagged_int_is_small(a) ? tagged_int_to_bigint(a) : tagged_int_get_bigint(a);
    ViperBigInt* b_big = tagged_int_is_small(b) ? tagged_int_to_bigint(b) : tagged_int_get_bigint(b);
    ViperBigInt* result = alloc_bigint_for_tagged();

    if (result) {
        if (mpz_sgn(b_big->value) == 0) {
            fprintf(stderr, "Error: Division by zero\n");
            mpz_set_ui(result->value, 0);
        } else {
            mpz_tdiv_q(result->value, a_big->value, b_big->value);
        }

        /* Try to demote result back to SmallInt */
        TaggedInt demoted = try_demote_bigint(result->value);
        if (demoted != 0) {
            // mpz_clear handled by ARC destructor
            free_bigint_for_tagged(result);
            /* Free temporaries */
            if (tagged_int_is_small(a)) free_temp_bigint(a_big);
            if (tagged_int_is_small(b)) free_temp_bigint(b_big);
            return demoted;
        }
    }

    /* Free temporaries (only the ones we created, not the original BigInts) */
    if (tagged_int_is_small(a)) free_temp_bigint(a_big);
    if (tagged_int_is_small(b)) free_temp_bigint(b_big);

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
        
        /* Try to demote result back to SmallInt */
        TaggedInt demoted = try_demote_bigint(result->value);
        if (demoted != 0) {
            // mpz_clear handled by ARC destructor
            free_bigint_for_tagged(result);
            free_temp_bigint(b_big);
            return demoted;
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
        
        /* Try to demote result back to SmallInt */
        TaggedInt demoted = try_demote_bigint(result->value);
        if (demoted != 0) {
            // mpz_clear handled by ARC destructor
            free_bigint_for_tagged(result);
            return demoted;
        }
    }

    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

/**
 * Power operation: base ^ exp
 * Both operands must be non-negative for BigInt case
 */
TaggedInt tagged_int_pow(TaggedInt base, TaggedInt exp) {
    /* Case 1: Both small integers */
    if (tagged_int_is_small(base) && tagged_int_is_small(exp)) {
        int64_t base_val = tagged_int_get_small(base);
        int64_t exp_val = tagged_int_get_small(exp);
        
        /* Check for negative base or exponent */
        if (base_val < 0 || exp_val < 0) {
            /* Promote to BigInt for negative values */
            goto bigint_case;
        }
        
        /* For small exponents, use repeated multiplication */
        if (exp_val == 0) {
            return tagged_int_from_i64(1);
        }
        if (exp_val == 1) {
            return base;
        }
        if (base_val == 0) {
            return tagged_int_from_i64(0);
        }
        if (base_val == 1) {
            return tagged_int_from_i64(1);
        }
        
        /* Check if result would overflow */
        /* Use logarithm to estimate: log(result) = exp * log(base) */
        /* For simplicity, just promote to BigInt for exp > 1 */
        /* This is conservative but safe */
    }

bigint_case:
    /* Case 2: At least one BigInt or overflow occurred */
    ViperBigInt* base_big = tagged_int_is_small(base) ? tagged_int_to_bigint(base) : tagged_int_get_bigint(base);
    ViperBigInt* exp_big = tagged_int_is_small(exp) ? tagged_int_to_bigint(exp) : tagged_int_get_bigint(exp);
    ViperBigInt* result = alloc_bigint_for_tagged();

    if (result) {
        /* Check for negative exponent */
        if (mpz_sgn(exp_big->value) < 0) {
            fprintf(stderr, "Error: Negative exponent in power operation\n");
            mpz_set_ui(result->value, 0);
        } else {
            /* Use repeated squaring for large exponents */
            if (mpz_fits_ulong_p(exp_big->value)) {
                mpz_pow_ui(result->value, base_big->value, mpz_get_ui(exp_big->value));
            } else {
                /* For very large exponents, use repeated squaring */
                mpz_t temp;
                mpz_init_set_ui(temp, 1);

                mp_bitcnt_t exp_bits = mpz_sizeinbase(exp_big->value, 2);
                for (mp_bitcnt_t i = exp_bits; i > 0; i--) {
                    mpz_mul(temp, temp, temp);
                    if (mpz_tstbit(exp_big->value, i - 1)) {
                        mpz_mul(temp, temp, base_big->value);
                    }
                }

                mpz_set(result->value, temp);
                mpz_clear(temp);
            }
        }

        /* Try to demote result back to SmallInt */
        TaggedInt demoted = try_demote_bigint(result->value);
        if (demoted != 0) {
            // mpz_clear handled by ARC destructor
            free_bigint_for_tagged(result);
            /* Free temporaries */
            if (tagged_int_is_small(base)) free_temp_bigint(base_big);
            if (tagged_int_is_small(exp)) free_temp_bigint(exp_big);
            return demoted;
        }
    }

    /* Free temporaries (only the ones we created, not the original BigInts) */
    if (tagged_int_is_small(base)) free_temp_bigint(base_big);
    if (tagged_int_is_small(exp)) free_temp_bigint(exp_big);

    return result ? tagged_int_from_bigint(result) : tagged_int_from_i64(0);
}

/* ============================================ */
/* Utility Functions                            */
/* ============================================ */

/**
 * Convert TaggedInt to C string (caller must free with free())
 * 
 * NOTE: This returns a raw char* which works for print() but NOT for str() builtin.
 * The str() builtin expects a ViperString* but this returns char*.
 * 
 * WORKAROUND: Use print(int_value) directly instead of print(str(int_value)).
 * 
 * TODO: To fix str() properly, we need to:
 * 1. Resolve header conflicts between viper_stdlib.h and viper_types.h
 * 2. Implement vp_str_create() that returns ViperString*
 * 3. Convert char* to ViperString* in str() builtin
 * 
 * For now, str() for tagged ints will segfault - this is a known limitation.
 */
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

    /* Get string from GMP - mpz_get_str returns malloc'd string */
    char* result = mpz_get_str(NULL, 10, bigint->value);

    /* Free the temporary bigint we created (if any) */
    if (is_temp) {
        mpz_clear(bigint->value);
        free(bigint);
    }

    return result;
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
        /* Free non-ARC allocated BigInt */
        if (bigint != NULL) {
            mpz_clear(bigint->value);
            free(bigint);
        }
    }
    /* Small integers don't need freeing */
}

/* ============================================ */
/* Minimal String Functions (for str() builtin) */
/* ============================================ */

/* Simple ViperString structure - matches viper_types.h */
typedef struct {
    union {
        struct {
            int64_t ref_count;
            int64_t length;
            char* heap_data;
        } heap;
        struct {
            int64_t _unused;
            int8_t sso_length;
            char sso_data[15];
        } sso;
    } data;
} MinimalViperString;

/* Create a ViperString from C string */
MinimalViperString* vp_str_create(const char* str) {
    if (!str) return NULL;
    
    int64_t len = (int64_t)strlen(str);
    MinimalViperString* s = (MinimalViperString*)malloc(sizeof(MinimalViperString));
    if (!s) return NULL;
    
    s->data.heap.ref_count = 1;
    s->data.heap.length = len;
    s->data.heap.heap_data = (char*)malloc(len + 1);
    if (!s->data.heap.heap_data) {
        free(s);
        return NULL;
    }
    memcpy(s->data.heap.heap_data, str, len + 1);
    return s;
}

/* Free a ViperString */
void vp_str_free(MinimalViperString* s) {
    if (!s) return;
    /* Check SSO flag (high bit of length) */
    if (!(s->data.heap.length & 0x80)) {
        /* Heap string - free the data */
        free(s->data.heap.heap_data);
    }
    free(s);
}

/* Get string data from ViperString */
const char* vp_str_data_inline(MinimalViperString* s) {
    if (!s) return "";
    /* Check SSO flag (high bit of length) */
    if (s->data.heap.length & 0x80) {
        /* SSO string - length is stored with high bit set */
        return s->data.sso.sso_data;
    } else {
        /* Heap string */
        return s->data.heap.heap_data;
    }
}

/* Print a C string (for use by print() builtin with str() result) */
void vp_print_str(const char* val) {
    if (!val) {
        printf("(null)");
        return;
    }
    printf("%s", val);
}

/* ============================================ */
/* Unified Runtime Dispatcher Aliases           */
/* ============================================ */
/* These aliases provide a consistent interface for runtime operations */

/**
 * Unified addition dispatcher
 * Alias for tagged_int_add - works with both SmallInt and BigInt
 */
TaggedInt vp_runtime_add(TaggedInt a, TaggedInt b) {
    return tagged_int_add(a, b);
}

/**
 * Unified subtraction dispatcher
 * Alias for tagged_int_sub - works with both SmallInt and BigInt
 */
TaggedInt vp_runtime_sub(TaggedInt a, TaggedInt b) {
    return tagged_int_sub(a, b);
}

/**
 * Unified multiplication dispatcher
 * Alias for tagged_int_mul - works with both SmallInt and BigInt
 */
TaggedInt vp_runtime_mul(TaggedInt a, TaggedInt b) {
    return tagged_int_mul(a, b);
}

/**
 * Unified division dispatcher
 * Alias for tagged_int_div - works with both SmallInt and BigInt
 */
TaggedInt vp_runtime_div(TaggedInt a, TaggedInt b) {
    return tagged_int_div(a, b);
}

/**
 * Unified modulo dispatcher
 * Alias for tagged_int_mod - works with both SmallInt and BigInt
 */
TaggedInt vp_runtime_mod(TaggedInt a, TaggedInt b) {
    return tagged_int_mod(a, b);
}

/**
 * Unified comparison dispatcher
 * Alias for tagged_int_cmp - works with both SmallInt and BigInt
 */
int vp_runtime_cmp(TaggedInt a, TaggedInt b) {
    return tagged_int_cmp(a, b);
}

/**
 * Unified equality check
 * Alias for tagged_int_eq - works with both SmallInt and BigInt
 */
bool vp_runtime_eq(TaggedInt a, TaggedInt b) {
    return tagged_int_eq(a, b);
}

/**
 * Unified less-than check
 * Alias for tagged_int_lt - works with both SmallInt and BigInt
 */
bool vp_runtime_lt(TaggedInt a, TaggedInt b) {
    return tagged_int_lt(a, b);
}

/**
 * Unified greater-than check
 * Alias for tagged_int_gt - works with both SmallInt and BigInt
 */
bool vp_runtime_gt(TaggedInt a, TaggedInt b) {
    return tagged_int_gt(a, b);
}

/**
 * Unified negation
 * Alias for tagged_int_neg - works with both SmallInt and BigInt
 */
TaggedInt vp_runtime_neg(TaggedInt a) {
    return tagged_int_neg(a);
}
