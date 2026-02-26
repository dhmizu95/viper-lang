/**
 * Viper BigInt Library - Arbitrary Precision Integers
 * 
 * Implementation using sign-magnitude representation with base 2^32 limbs.
 */

#include "bigint.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <ctype.h>
#include <limits.h>

/* ============================================ */
/* Internal Helpers                             */
/* ============================================ */

#define BIGINT_MIN_CAP 16

static VpBigInt* bigint_new(void) {
    VpBigInt* n = (VpBigInt*)malloc(sizeof(VpBigInt));
    if (!n) return NULL;
    n->sign = 1;
    n->len = 0;
    n->cap = BIGINT_MIN_CAP;
    n->digits = (uint32_t*)malloc(sizeof(uint32_t) * BIGINT_MIN_CAP);
    if (!n->digits) {
        free(n);
        return NULL;
    }
    memset(n->digits, 0, sizeof(uint32_t) * BIGINT_MIN_CAP);
    return n;
}

static void bigint_normalize(VpBigInt* a) {
    while (a->len > 0 && a->digits[a->len - 1] == 0) {
        a->len--;
    }
    if (a->len == 0) {
        a->sign = 1; /* -0 should be +0 */
    }
}

static int bigint_ensure_cap(VpBigInt* a, size_t cap) {
    if (a->cap >= cap) return 0;
    size_t new_cap = a->cap * 2;
    if (new_cap < cap) new_cap = cap;
    uint32_t* new_digits = (uint32_t*)realloc(a->digits, sizeof(uint32_t) * new_cap);
    if (!new_digits) return -1;
    a->digits = new_digits;
    a->cap = new_cap;
    return 0;
}

/* Compare absolute values */
static int bigint_cmp_abs(VpBigInt* a, VpBigInt* b) {
    if (a->len != b->len) {
        return (a->len < b->len) ? -1 : 1;
    }
    for (size_t i = a->len; i > 0; i--) {
        if (a->digits[i - 1] != b->digits[i - 1]) {
            return (a->digits[i - 1] < b->digits[i - 1]) ? -1 : 1;
        }
    }
    return 0;
}

/* ============================================ */
/* Construction and Destruction                 */
/* ============================================ */

VpBigInt* vp_bigint_from_i64(int64_t v) {
    VpBigInt* n = bigint_new();
    if (!n) return NULL;
    
    if (v < 0) {
        n->sign = -1;
        v = -v;
    }
    
    while (v > 0) {
        if (bigint_ensure_cap(n, n->len + 1) < 0) {
            vp_bigint_free(n);
            return NULL;
        }
        n->digits[n->len++] = (uint32_t)(v & 0xFFFFFFFF);
        v >>= 32;
    }
    
    if (n->len == 0) n->len = 1; /* Zero */
    return n;
}

VpBigInt* vp_bigint_from_str(const char* s) {
    VpBigInt* n = bigint_new();
    if (!n) return NULL;
    
    const char* p = s;
    int sign = 1;
    
    /* Skip leading whitespace */
    while (isspace(*p)) p++;
    
    /* Handle sign */
    if (*p == '-') {
        sign = -1;
        p++;
    } else if (*p == '+') {
        p++;
    }
    
    /* Skip leading zeros */
    while (*p == '0') p++;
    
    /* Parse decimal digits */
    const char* start = p;
    while (isdigit(*p)) p++;
    
    if (p == start) {
        /* No digits found - return zero */
        vp_bigint_free(n);
        return vp_bigint_from_i64(0);
    }
    
    /* Parse digit by digit: n = n * 10 + digit */
    for (const char* q = start; q < p; q++) {
        int digit = *q - '0';
        
        /* Multiply by 10 */
        uint64_t carry = 0;
        for (size_t i = 0; i < n->len || carry; i++) {
            if (i >= n->len) {
                if (bigint_ensure_cap(n, n->len + 1) < 0) {
                    vp_bigint_free(n);
                    return NULL;
                }
                n->digits[n->len++] = 0;
            }
            uint64_t tmp = (uint64_t)n->digits[i] * 10 + carry;
            n->digits[i] = (uint32_t)(tmp & 0xFFFFFFFF);
            carry = tmp >> 32;
        }
        
        /* Add digit */
        if (n->len == 0) {
            if (bigint_ensure_cap(n, 1) < 0) {
                vp_bigint_free(n);
                return NULL;
            }
            n->len = 1;
        }
        uint64_t sum = (uint64_t)n->digits[0] + digit;
        n->digits[0] = (uint32_t)(sum & 0xFFFFFFFF);
        carry = sum >> 32;
        
        size_t i = 1;
        while (carry) {
            if (i >= n->len) {
                if (bigint_ensure_cap(n, n->len + 1) < 0) {
                    vp_bigint_free(n);
                    return NULL;
                }
                n->digits[n->len++] = 0;
            }
            sum = (uint64_t)n->digits[i] + carry;
            n->digits[i] = (uint32_t)(sum & 0xFFFFFFFF);
            carry = sum >> 32;
            i++;
        }
    }
    
    n->sign = sign;
    bigint_normalize(n);
    return n;
}

VpBigInt* vp_bigint_copy(VpBigInt* a) {
    VpBigInt* n = bigint_new();
    if (!n) return NULL;
    
    if (bigint_ensure_cap(n, a->len) < 0) {
        vp_bigint_free(n);
        return NULL;
    }
    
    n->sign = a->sign;
    n->len = a->len;
    memcpy(n->digits, a->digits, sizeof(uint32_t) * a->len);
    return n;
}

void vp_bigint_free(VpBigInt* a) {
    if (a) {
        free(a->digits);
        free(a);
    }
}

/* ============================================ */
/* Addition and Subtraction                     */
/* ============================================ */

/* Add absolute values: |a| + |b| */
static VpBigInt* bigint_add_abs(VpBigInt* a, VpBigInt* b) {
    VpBigInt* result = bigint_new();
    if (!result) return NULL;
    
    size_t max_len = (a->len > b->len) ? a->len : b->len;
    if (bigint_ensure_cap(result, max_len + 1) < 0) {
        vp_bigint_free(result);
        return NULL;
    }
    
    uint64_t carry = 0;
    for (size_t i = 0; i < max_len || carry; i++) {
        uint64_t sum = carry;
        if (i < a->len) sum += a->digits[i];
        if (i < b->len) sum += b->digits[i];
        
        if (i >= result->len) {
            if (bigint_ensure_cap(result, i + 1) < 0) {
                vp_bigint_free(result);
                return NULL;
            }
            result->len = i + 1;
        }
        
        result->digits[i] = (uint32_t)(sum & 0xFFFFFFFF);
        carry = sum >> 32;
    }
    
    result->sign = 1;
    bigint_normalize(result);
    return result;
}

/* Subtract absolute values: |a| - |b|, assumes |a| >= |b| */
static VpBigInt* bigint_sub_abs(VpBigInt* a, VpBigInt* b) {
    VpBigInt* result = bigint_new();
    if (!result) return NULL;
    
    if (bigint_ensure_cap(result, a->len) < 0) {
        vp_bigint_free(result);
        return NULL;
    }
    
    int64_t borrow = 0;
    for (size_t i = 0; i < a->len; i++) {
        int64_t diff = (int64_t)a->digits[i] - borrow;
        if (i < b->len) diff -= b->digits[i];
        
        if (diff < 0) {
            diff += (1LL << 32);
            borrow = 1;
        } else {
            borrow = 0;
        }
        
        if (i >= result->len) {
            if (bigint_ensure_cap(result, i + 1) < 0) {
                vp_bigint_free(result);
                return NULL;
            }
            result->len = i + 1;
        }
        
        result->digits[i] = (uint32_t)diff;
    }
    
    result->sign = 1;
    bigint_normalize(result);
    return result;
}

VpBigInt* vp_bigint_add(VpBigInt* a, VpBigInt* b) {
    /* Same sign: add absolute values, keep sign */
    if (a->sign == b->sign) {
        VpBigInt* result = bigint_add_abs(a, b);
        if (result) result->sign = a->sign;
        return result;
    }
    
    /* Different signs: subtract smaller from larger */
    int cmp = bigint_cmp_abs(a, b);
    if (cmp == 0) {
        return vp_bigint_from_i64(0);
    } else if (cmp > 0) {
        VpBigInt* result = bigint_sub_abs(a, b);
        if (result) result->sign = a->sign;
        return result;
    } else {
        VpBigInt* result = bigint_sub_abs(b, a);
        if (result) result->sign = b->sign;
        return result;
    }
}

VpBigInt* vp_bigint_sub(VpBigInt* a, VpBigInt* b) {
    /* a - b = a + (-b) */
    VpBigInt* neg_b = vp_bigint_copy(b);
    if (!neg_b) return NULL;
    neg_b->sign = -neg_b->sign;
    
    VpBigInt* result = vp_bigint_add(a, neg_b);
    vp_bigint_free(neg_b);
    return result;
}

/* ============================================ */
/* Multiplication                               */
/* ============================================ */

VpBigInt* vp_bigint_mul(VpBigInt* a, VpBigInt* b) {
    if (a->len == 0 || b->len == 0) {
        return vp_bigint_from_i64(0);
    }
    
    VpBigInt* result = bigint_new();
    if (!result) return NULL;
    
    if (bigint_ensure_cap(result, a->len + b->len) < 0) {
        vp_bigint_free(result);
        return NULL;
    }
    result->len = a->len + b->len;
    memset(result->digits, 0, sizeof(uint32_t) * result->len);
    
    /* Grade-school multiplication */
    for (size_t i = 0; i < a->len; i++) {
        uint64_t carry = 0;
        for (size_t j = 0; j < b->len || carry; j++) {
            uint64_t prod = result->digits[i + j] + carry;
            if (j < b->len) {
                prod += (uint64_t)a->digits[i] * b->digits[j];
            }
            result->digits[i + j] = (uint32_t)(prod & 0xFFFFFFFF);
            carry = prod >> 32;
        }
    }
    
    result->sign = (a->sign == b->sign) ? 1 : -1;
    bigint_normalize(result);
    return result;
}

/* ============================================ */
/* Division and Modulo                          */
/* ============================================ */

/* Simple division by single limb */
static void bigint_divmod_limb(VpBigInt* a, uint32_t divisor, VpBigInt** quot, uint32_t* rem) {
    if (divisor == 0) {
        if (quot) *quot = vp_bigint_from_i64(0);
        if (rem) *rem = 0;
        return;
    }
    
    VpBigInt* q = bigint_new();
    if (!q) {
        if (quot) *quot = NULL;
        if (rem) *rem = 0;
        return;
    }
    
    if (bigint_ensure_cap(q, a->len) < 0) {
        vp_bigint_free(q);
        if (quot) *quot = NULL;
        if (rem) *rem = 0;
        return;
    }
    
    uint64_t remainder = 0;
    for (size_t i = a->len; i > 0; i--) {
        uint64_t tmp = (remainder << 32) | a->digits[i - 1];
        q->digits[i - 1] = (uint32_t)(tmp / divisor);
        remainder = tmp % divisor;
    }
    
    q->len = a->len;
    q->sign = a->sign;
    bigint_normalize(q);
    
    if (quot) *quot = q;
    if (rem) *rem = (uint32_t)remainder;
}

/* Simple division algorithm using repeated subtraction for small divisors */
static void bigint_divmod_abs(VpBigInt* a, VpBigInt* b, VpBigInt** quot, VpBigInt** rem) {
    int cmp = bigint_cmp_abs(a, b);

    if (cmp < 0) {
        /* a < b: quotient = 0, remainder = a */
        if (quot) *quot = vp_bigint_from_i64(0);
        if (rem) *rem = vp_bigint_copy(a);
        return;
    }

    if (cmp == 0) {
        /* a == b: quotient = 1, remainder = 0 */
        if (quot) *quot = vp_bigint_from_i64(1);
        if (rem) *rem = vp_bigint_from_i64(0);
        return;
    }
    
    /* Special case: divisor is a single limb */
    if (b->len == 1) {
        VpBigInt* q;
        uint32_t r;
        bigint_divmod_limb(a, b->digits[0], &q, &r);
        if (quot) *quot = q;
        if (rem) {
            *rem = vp_bigint_from_i64(r);
            if (*rem) (*rem)->sign = a->sign;
        }
        return;
    }

    /* For multi-limb divisors, use a simpler algorithm */
    /* Repeated subtraction with bit shifting */
    
    VpBigInt* quotient = vp_bigint_from_i64(0);
    VpBigInt* remainder = vp_bigint_copy(a);
    
    if (!quotient || !remainder) {
        if (quotient) vp_bigint_free(quotient);
        if (remainder) vp_bigint_free(remainder);
        if (quot) *quot = NULL;
        if (rem) *rem = NULL;
        return;
    }
    
    /* Find the bit position of the highest set bit in divisor */
    size_t m = b->len;
    uint32_t high_digit = b->digits[m - 1];
    size_t shift_bits = 0;
    while (high_digit > 0) {
        high_digit >>= 1;
        shift_bits++;
    }
    size_t divisor_high_bit = (m - 1) * 32 + shift_bits;
    
    /* Find the bit position of the highest set bit in remainder */
    while (bigint_cmp_abs(remainder, b) >= 0) {
        size_t r_high_bit = 0;
        if (remainder->len > 0) {
            uint32_t r_high_digit = remainder->digits[remainder->len - 1];
            size_t r_shift = 0;
            while (r_high_digit > 0) {
                r_high_digit >>= 1;
                r_shift++;
            }
            r_high_bit = (remainder->len - 1) * 32 + r_shift;
        }
        
        /* Calculate how much to shift */
        size_t shift = (r_high_bit >= divisor_high_bit) ? (r_high_bit - divisor_high_bit) : 0;
        
        /* Create shifted divisor */
        VpBigInt* shifted_divisor = vp_bigint_copy(b);
        if (!shifted_divisor) break;
        
        /* Shift left by 'shift' bits */
        size_t limb_shift = shift / 32;
        size_t bit_shift = shift % 32;
        
        size_t new_len = shifted_divisor->len + limb_shift + (bit_shift > 0 ? 1 : 0);
        if (bigint_ensure_cap(shifted_divisor, new_len) < 0) {
            vp_bigint_free(shifted_divisor);
            break;
        }
        
        /* Shift the digits */
        for (size_t i = shifted_divisor->len; i > 0; i--) {
            uint64_t val = shifted_divisor->digits[i - 1];
            size_t new_idx = i - 1 + limb_shift;
            if (bit_shift > 0 && new_idx + 1 < shifted_divisor->cap) {
                shifted_divisor->digits[new_idx + 1] |= (uint32_t)(val >> (32 - bit_shift));
            }
            if (new_idx < shifted_divisor->cap) {
                shifted_divisor->digits[new_idx] = (uint32_t)(val << bit_shift);
            }
        }
        
        /* Zero out the lower limbs */
        for (size_t i = 0; i < limb_shift && i < shifted_divisor->cap; i++) {
            shifted_divisor->digits[i] = 0;
        }
        
        shifted_divisor->len = new_len;
        bigint_normalize(shifted_divisor);
        
        /* If shifted divisor is still <= remainder, subtract it */
        if (bigint_cmp_abs(remainder, shifted_divisor) >= 0) {
            VpBigInt* new_rem = bigint_sub_abs(remainder, shifted_divisor);
            if (new_rem) {
                vp_bigint_free(remainder);
                remainder = new_rem;
            }
            
            /* Add 2^shift to quotient */
            VpBigInt* power_of_2 = vp_bigint_from_i64(1);
            for (size_t i = 0; i < shift; i++) {
                VpBigInt* doubled = vp_bigint_add(power_of_2, power_of_2);
                vp_bigint_free(power_of_2);
                power_of_2 = doubled;
            }
            
            VpBigInt* new_quot = vp_bigint_add(quotient, power_of_2);
            vp_bigint_free(power_of_2);
            vp_bigint_free(quotient);
            quotient = new_quot;
        }
        
        vp_bigint_free(shifted_divisor);
    }
    
    if (quot) *quot = quotient;
    if (rem) *rem = remainder;
}

VpBigInt* vp_bigint_div(VpBigInt* a, VpBigInt* b) {
    if (b->len == 0 || (b->len == 1 && b->digits[0] == 0)) {
        return vp_bigint_from_i64(0); /* Division by zero */
    }
    
    VpBigInt* quot = NULL;
    VpBigInt* rem = NULL;
    bigint_divmod_abs(a, b, &quot, &rem);
    
    if (quot) {
        /* Floor division: result sign is XOR of operand signs */
        quot->sign = (a->sign == b->sign) ? 1 : -1;
        
        /* Adjust for floor semantics (round toward negative infinity) */
        if (rem && rem->len > 0 && quot->sign < 0) {
            /* If there's a remainder and result is negative, subtract 1 */
            VpBigInt* one = vp_bigint_from_i64(1);
            VpBigInt* adjusted = vp_bigint_sub(quot, one);
            vp_bigint_free(one);
            vp_bigint_free(quot);
            quot = adjusted;
        }
    }
    
    if (rem) vp_bigint_free(rem);
    return quot;
}

VpBigInt* vp_bigint_mod(VpBigInt* a, VpBigInt* b) {
    if (b->len == 0 || (b->len == 1 && b->digits[0] == 0)) {
        return vp_bigint_from_i64(0); /* Modulo by zero */
    }
    
    VpBigInt* quot = NULL;
    VpBigInt* rem = NULL;
    bigint_divmod_abs(a, b, &quot, &rem);
    
    if (quot) vp_bigint_free(quot);
    
    if (rem) {
        /* Modulo result has same sign as divisor */
        rem->sign = b->sign;
    }
    
    return rem;
}

/* ============================================ */
/* Power                                        */
/* ============================================ */

VpBigInt* vp_bigint_pow(VpBigInt* base, VpBigInt* exp) {
    /* Handle special cases */
    if (exp->len == 0 || (exp->len == 1 && exp->digits[0] == 0)) {
        return vp_bigint_from_i64(1); /* x^0 = 1 */
    }
    
    if (base->len == 0 || (base->len == 1 && base->digits[0] == 0)) {
        return vp_bigint_from_i64(0); /* 0^x = 0 (for x > 0) */
    }
    
    /* Binary exponentiation */
    VpBigInt* result = vp_bigint_from_i64(1);
    VpBigInt* b = vp_bigint_copy(base);
    
    if (!result || !b) {
        if (result) vp_bigint_free(result);
        if (b) vp_bigint_free(b);
        return NULL;
    }
    
    VpBigInt* e = vp_bigint_copy(exp);
    if (!e) {
        vp_bigint_free(result);
        vp_bigint_free(b);
        return NULL;
    }
    
    while (e->len > 0) {
        /* If odd, multiply result by base */
        if (e->digits[0] & 1) {
            VpBigInt* tmp = vp_bigint_mul(result, b);
            vp_bigint_free(result);
            result = tmp;
            if (!result) break;
        }
        
        /* Square base */
        VpBigInt* tmp = vp_bigint_mul(b, b);
        vp_bigint_free(b);
        b = tmp;
        if (!b) break;
        
        /* Divide exponent by 2 */
        uint32_t carry = 0;
        for (size_t i = e->len; i > 0; i--) {
            uint32_t new_digit = (e->digits[i - 1] >> 1) | (carry ? (1U << 31) : 0);
            carry = e->digits[i - 1] & 1;
            e->digits[i - 1] = new_digit;
        }
        bigint_normalize(e);
    }
    
    vp_bigint_free(b);
    vp_bigint_free(e);
    
    return result;
}

VpBigInt* vp_bigint_neg(VpBigInt* a) {
    VpBigInt* result = vp_bigint_copy(a);
    if (result) {
        result->sign = -result->sign;
    }
    return result;
}

VpBigInt* vp_bigint_abs(VpBigInt* a) {
    VpBigInt* result = vp_bigint_copy(a);
    if (result) {
        result->sign = 1;
    }
    return result;
}

/* ============================================ */
/* Comparison                                   */
/* ============================================ */

int vp_bigint_cmp(VpBigInt* a, VpBigInt* b) {
    /* Different signs */
    if (a->sign != b->sign) {
        return a->sign;
    }
    
    /* Same sign: compare absolute values */
    int abs_cmp = bigint_cmp_abs(a, b);
    
    if (a->sign > 0) {
        return abs_cmp;
    } else {
        return -abs_cmp;
    }
}

bool vp_bigint_eq(VpBigInt* a, VpBigInt* b) {
    return vp_bigint_cmp(a, b) == 0;
}

bool vp_bigint_lt(VpBigInt* a, VpBigInt* b) {
    return vp_bigint_cmp(a, b) < 0;
}

bool vp_bigint_le(VpBigInt* a, VpBigInt* b) {
    return vp_bigint_cmp(a, b) <= 0;
}

bool vp_bigint_gt(VpBigInt* a, VpBigInt* b) {
    return vp_bigint_cmp(a, b) > 0;
}

bool vp_bigint_ge(VpBigInt* a, VpBigInt* b) {
    return vp_bigint_cmp(a, b) >= 0;
}

/* ============================================ */
/* Conversion                                   */
/* ============================================ */

char* vp_bigint_to_str(VpBigInt* a) {
    if (!a) {
        return strdup("(null)");
    }
    if (a->len == 0) {
        char* s = (char*)malloc(2);
        if (s) strcpy(s, "0");
        return s;
    }

    /* For numbers that fit in i64, use snprintf */
    bool overflow;
    int64_t small_val = vp_bigint_to_i64(a, &overflow);
    if (!overflow) {
        char* s = (char*)malloc(32);
        if (s) {
            snprintf(s, 32, "%lld", (long long)small_val);
        }
        return s;
    }

    /* For large numbers, use repeated division by 10 */
    VpBigInt* temp = vp_bigint_copy(a);
    if (!temp) return NULL;

    /* Allocate buffer - each limb is ~10 decimal digits */
    size_t max_digits = a->len * 10 + 20;
    char* buf = (char*)malloc(max_digits);
    if (!buf) {
        vp_bigint_free(temp);
        return NULL;
    }

    char* p = buf + max_digits - 1;
    *p = '\0';

    /* Repeatedly divide by 10 and collect remainders */
    while (temp->len > 0) {
        /* Check if temp is zero */
        int is_zero = 1;
        for (size_t i = 0; i < temp->len; i++) {
            if (temp->digits[i] != 0) {
                is_zero = 0;
                break;
            }
        }
        if (is_zero) break;

        /* Divide by 10 using single-limb division */
        VpBigInt* quot = NULL;
        uint32_t rem = 0;
        bigint_divmod_limb(temp, 10, &quot, &rem);

        vp_bigint_free(temp);
        temp = quot;

        p--;
        *p = '0' + rem;
    }

    vp_bigint_free(temp);

    /* Add negative sign if needed */
    if (a->sign < 0 && p > buf) {
        p--;
        *p = '-';
    }

    /* Move result to beginning of buffer */
    size_t len = strlen(p);
    memmove(buf, p, len + 1);

    return buf;
}

int64_t vp_bigint_to_i64(VpBigInt* a, bool* overflow) {
    *overflow = false;
    
    if (a->len == 0) return 0;
    
    /* Check if value fits in i64 */
    if (a->len > 2) {
        *overflow = true;
        return 0;
    }
    
    if (a->len == 2) {
        /* High limb must be 0 for positive, or all 1s for negative */
        if (a->sign > 0 && a->digits[1] > 0) {
            *overflow = true;
            return 0;
        }
        if (a->sign < 0) {
            /* Check for INT64_MIN */
            if (a->digits[1] > 0x7FFFFFFF || 
                (a->digits[1] == 0x7FFFFFFF && a->digits[0] != 0)) {
                *overflow = true;
                return 0;
            }
        }
    }
    
    int64_t result = a->digits[0];
    if (a->len > 1) {
        result |= ((int64_t)a->digits[1] << 32);
    }
    
    if (a->sign < 0) {
        result = -result;
    }
    
    return result;
}
