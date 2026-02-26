/**
 * Viper BigInt Library - Arbitrary Precision Integers
 * 
 * Uses sign-magnitude representation with base 2^32 limbs,
 * similar to CPython's implementation.
 */

#ifndef VIPER_BIGINT_H
#define VIPER_BIGINT_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/**
 * BigInt structure for arbitrary precision integers
 * 
 * Representation: value = sign * sum(digits[i] * (2^32)^i)
 * - sign: 1 for positive, -1 for negative
 * - len: number of used limbs
 * - cap: allocated capacity in limbs
 * - digits: array of 32-bit limbs (little-endian)
 */
typedef struct {
    int sign;           /* 1 or -1 */
    size_t len;         /* Number of used limbs */
    size_t cap;         /* Allocated capacity */
    uint32_t* digits;   /* Array of limbs (little-endian) */
} VpBigInt;

/* ============================================ */
/* Construction and Destruction                 */
/* ============================================ */

/**
 * Create a BigInt from a 64-bit integer
 */
VpBigInt* vp_bigint_from_i64(int64_t v);

/**
 * Create a BigInt from a decimal string
 * Supports optional leading '-' for negative numbers
 */
VpBigInt* vp_bigint_from_str(const char* s);

/**
 * Create a copy of a BigInt
 */
VpBigInt* vp_bigint_copy(VpBigInt* a);

/**
 * Free a BigInt and its resources
 */
void vp_bigint_free(VpBigInt* a);

/* ============================================ */
/* Arithmetic Operations                        */
/* ============================================ */

/**
 * Addition: a + b
 */
VpBigInt* vp_bigint_add(VpBigInt* a, VpBigInt* b);

/**
 * Subtraction: a - b
 */
VpBigInt* vp_bigint_sub(VpBigInt* a, VpBigInt* b);

/**
 * Multiplication: a * b
 * Uses grade-school O(n^2) algorithm
 */
VpBigInt* vp_bigint_mul(VpBigInt* a, VpBigInt* b);

/**
 * Division: a / b (floor division)
 * Returns floor(a / b)
 */
VpBigInt* vp_bigint_div(VpBigInt* a, VpBigInt* b);

/**
 * Modulo: a % b
 * Result has same sign as divisor (b)
 */
VpBigInt* vp_bigint_mod(VpBigInt* a, VpBigInt* b);

/**
 * Power: base^exp
 * Uses binary exponentiation
 * exp must be non-negative
 */
VpBigInt* vp_bigint_pow(VpBigInt* base, VpBigInt* exp);

/**
 * Negation: -a
 */
VpBigInt* vp_bigint_neg(VpBigInt* a);

/**
 * Absolute value: |a|
 */
VpBigInt* vp_bigint_abs(VpBigInt* a);

/* ============================================ */
/* Comparison Operations                        */
/* ============================================ */

/**
 * Compare two BigInts
 * Returns: -1 if a < b, 0 if a == b, 1 if a > b
 */
int vp_bigint_cmp(VpBigInt* a, VpBigInt* b);

/**
 * Check if two BigInts are equal
 */
bool vp_bigint_eq(VpBigInt* a, VpBigInt* b);

/**
 * Check if a is less than b
 */
bool vp_bigint_lt(VpBigInt* a, VpBigInt* b);

/**
 * Check if a is less than or equal to b
 */
bool vp_bigint_le(VpBigInt* a, VpBigInt* b);

/**
 * Check if a is greater than b
 */
bool vp_bigint_gt(VpBigInt* a, VpBigInt* b);

/**
 * Check if a is greater than or equal to b
 */
bool vp_bigint_ge(VpBigInt* a, VpBigInt* b);

/* ============================================ */
/* Conversion                                   */
/* ============================================ */

/**
 * Convert BigInt to decimal string
 * Caller must free the returned string
 */
char* vp_bigint_to_str(VpBigInt* a);

/**
 * Try to convert BigInt to i64
 * Returns 0 and sets *overflow to true if conversion would overflow
 */
int64_t vp_bigint_to_i64(VpBigInt* a, bool* overflow);

#endif /* VIPER_BIGINT_H */
