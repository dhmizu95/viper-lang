/**
 * Viper GMP Bridge Header
 * 
 * Provides arbitrary-precision integer operations using GMP (GNU Multiple Precision Arithmetic Library)
 * Integrated with Viper's ARC (Automatic Reference Counting) memory management
 * 
 * @file gmp_bridge.h
 */

#ifndef VIPER_GMP_BRIDGE_H
#define VIPER_GMP_BRIDGE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <gmp.h>

#include "viper_arc.h"

/* ============================================ */
/* ViperBigInt Structure                        */
/* ============================================ */

/**
 * ViperBigInt - Arbitrary precision integer backed by GMP mpz_t
 * 
 * This struct is managed by Viper's ARC system.
 * The header is prepended by vp_arc_alloc().
 */
typedef struct {
    mpz_t value;            /* GMP integer value */
} ViperBigInt;

/* ============================================ */
/* Core BigInt Operations                       */
/* ============================================ */

/**
 * Create a new BigInt from a string representation
 * @param str String representation (base 10, base 16 with "0x" prefix, etc.)
 * @return Pointer to new ViperBigInt, or NULL on failure
 */
ViperBigInt* vp_bigint_from_str(const char* str);

/**
 * Create a new BigInt from a 64-bit integer
 * @param value int64_t value to convert
 * @return Pointer to new ViperBigInt
 */
ViperBigInt* vp_bigint_from_i64(int64_t value);

/**
 * Create a new BigInt from a 64-bit integer for temporary operation results
 * This function is an alias for vp_bigint_from_i64 - the result has ref_count=1
 * The caller takes ownership - do NOT call retain when assigning the result
 * @param value int64_t value to convert
 * @return Pointer to new ViperBigInt with ref_count=1
 */
ViperBigInt* vp_bigint_from_i64_temp(int64_t value);

/**
 * Create a new BigInt from an unsigned 64-bit integer
 * @param value uint64_t value to convert
 * @return Pointer to new ViperBigInt
 */
ViperBigInt* vp_bigint_from_u64(uint64_t value);

/**
 * Destroy a BigInt and free GMP resources
 * @param bigint Pointer to ViperBigInt to destroy
 */
void vp_bigint_destroy(ViperBigInt* bigint);

/**
 * Convert BigInt to string representation
 * @param bigint Pointer to ViperBigInt
 * @param base Number base (10 for decimal, 16 for hex, etc.)
 * @return Newly allocated string (caller must free), or NULL on failure
 */
char* vp_bigint_to_str(ViperBigInt* bigint, int base);

/**
 * Convert BigInt to int64_t (may overflow for large values)
 * @param bigint Pointer to ViperBigInt
 * @return int64_t value (truncated if too large)
 */
int64_t vp_bigint_to_i64(ViperBigInt* bigint);

/**
 * Get the absolute value of a BigInt
 * @param result Result BigInt (must be initialized)
 * @param operand Operand BigInt
 */
void vp_bigint_abs(ViperBigInt* result, ViperBigInt* operand);

/**
 * Negate a BigInt
 * @param result Result BigInt (must be initialized)
 * @param operand Operand BigInt
 */
void vp_bigint_neg(ViperBigInt* result, ViperBigInt* operand);

/* ============================================ */
/* Arithmetic Operations                        */
/* ============================================ */

/**
 * Add two BigInts: result = a + b
 * @param result Result BigInt
 * @param a First operand
 * @param b Second operand
 */
void vp_bigint_add(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Subtract two BigInts: result = a - b
 * @param result Result BigInt
 * @param a First operand
 * @param b Second operand
 */
void vp_bigint_sub(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Multiply two BigInts: result = a * b
 * @param result Result BigInt
 * @param a First operand
 * @param b Second operand
 */
void vp_bigint_mul(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Divide two BigInts: result = a / b (integer division)
 * @param result Result BigInt (quotient)
 * @param a Dividend
 * @param b Divisor
 */
void vp_bigint_div(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Modulo operation: result = a % b
 * @param result Result BigInt (remainder)
 * @param a Dividend
 * @param b Divisor
 */
void vp_bigint_mod(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Divmod operation: quotient = a / b, remainder = a % b
 * @param quotient Quotient result
 * @param remainder Remainder result
 * @param a Dividend
 * @param b Divisor
 */
void vp_bigint_divmod(ViperBigInt* quotient, ViperBigInt* remainder, ViperBigInt* a, ViperBigInt* b);

/**
 * Power operation: result = base ^ exp
 * @param result Result BigInt
 * @param base Base
 * @param exp Exponent (must be non-negative)
 */
void vp_bigint_pow(ViperBigInt* result, ViperBigInt* base, ViperBigInt* exp);

/**
 * Square root: result = floor(sqrt(a))
 * @param result Result BigInt
 * @param a Operand (must be non-negative)
 */
void vp_bigint_sqrt(ViperBigInt* result, ViperBigInt* a);

/* ============================================ */
/* Mixed Arithmetic (BigInt + native types)     */
/* ============================================ */

/**
 * Add int64_t to BigInt: result = a + b
 * @param result Result BigInt
 * @param a BigInt operand
 * @param b int64_t operand
 */
void vp_bigint_add_i64(ViperBigInt* result, ViperBigInt* a, int64_t b);

/**
 * Subtract int64_t from BigInt: result = a - b
 * @param result Result BigInt
 * @param a BigInt operand
 * @param b int64_t operand
 */
void vp_bigint_sub_i64(ViperBigInt* result, ViperBigInt* a, int64_t b);

/**
 * Multiply BigInt by int64_t: result = a * b
 * @param result Result BigInt
 * @param a BigInt operand
 * @param b int64_t operand
 */
void vp_bigint_mul_i64(ViperBigInt* result, ViperBigInt* a, int64_t b);

/**
 * Divide BigInt by int64_t: result = a / b
 * @param result Result BigInt
 * @param a BigInt operand
 * @param b int64_t operand
 */
void vp_bigint_div_i64(ViperBigInt* result, ViperBigInt* a, int64_t b);

/* ============================================ */
/* Bitwise Operations                           */
/* ============================================ */

/**
 * Bitwise AND: result = a & b
 * @param result Result BigInt
 * @param a First operand
 * @param b Second operand
 */
void vp_bigint_and(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Bitwise OR: result = a | b
 * @param result Result BigInt
 * @param a First operand
 * @param b Second operand
 */
void vp_bigint_or(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Bitwise XOR: result = a ^ b
 * @param result Result BigInt
 * @param a First operand
 * @param b Second operand
 */
void vp_bigint_xor(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Left shift: result = a << b
 * @param result Result BigInt
 * @param a Operand
 * @param b Shift amount (must be non-negative)
 */
void vp_bigint_lshift(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Bitwise NOT / Inversion: result = ~a
 * @param result Result BigInt
 * @param a Operand
 */
void vp_bigint_invert(ViperBigInt* result, ViperBigInt* a);

/**
 * Right shift: result = a >> b
 * @param result Result BigInt
 * @param a Operand
 * @param b Shift amount (must be non-negative)
 */
void vp_bigint_rshift(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/* ============================================ */
/* Comparison Operations                        */
/* ============================================ */

/**
 * Compare two BigInts
 * @param a First operand
 * @param b Second operand
 * @return -1 if a < b, 0 if a == b, 1 if a > b
 */
int vp_bigint_cmp(ViperBigInt* a, ViperBigInt* b);

/**
 * Compare BigInt with int64_t
 * @param a BigInt operand
 * @param b int64_t operand
 * @return -1 if a < b, 0 if a == b, 1 if a > b
 */
int vp_bigint_cmp_i64(ViperBigInt* a, int64_t b);

/**
 * Check if two BigInts are equal
 * @param a First operand
 * @param b Second operand
 * @return true if equal, false otherwise
 */
bool vp_bigint_eq(ViperBigInt* a, ViperBigInt* b);

/**
 * Check if BigInt is less than another
 * @param a First operand
 * @param b Second operand
 * @return true if a < b, false otherwise
 */
bool vp_bigint_lt(ViperBigInt* a, ViperBigInt* b);

/**
 * Check if BigInt is greater than another
 * @param a First operand
 * @param b Second operand
 * @return true if a > b, false otherwise
 */
bool vp_bigint_gt(ViperBigInt* a, ViperBigInt* b);

/**
 * Check if BigInt is zero
 * @param a Operand
 * @return true if zero, false otherwise
 */
bool vp_bigint_is_zero(ViperBigInt* a);

/**
 * Check if BigInt is negative
 * @param a Operand
 * @return true if negative, false otherwise
 */
bool vp_bigint_is_negative(ViperBigInt* a);

/* ============================================ */
/* Utility Operations                           */
/* ============================================ */

/**
 * Get the number of bits required to represent the BigInt
 * @param a Operand
 * @return Number of bits
 */
size_t vp_bigint_bit_length(ViperBigInt* a);

/**
 * Get the sign of the BigInt
 * @param a Operand
 * @return -1 if negative, 0 if zero, 1 if positive
 */
int vp_bigint_sign(ViperBigInt* a);

/**
 * Copy a BigInt
 * @param dest Destination (must be initialized)
 * @param src Source
 */
void vp_bigint_copy(ViperBigInt* dest, ViperBigInt* src);

/**
 * Get a hash value for the BigInt (for use in dicts/sets)
 * @param a Operand
 * @return Hash value
 */
uint64_t vp_bigint_hash(ViperBigInt* a);

/**
 * Minimum of two BigInts: result = min(a, b)
 */
void vp_bigint_min(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/**
 * Maximum of two BigInts: result = max(a, b)
 */
void vp_bigint_max(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);

/* ============================================ */
/* JIT Stub Aliases (_c suffix for Rust FFI)    */
/* ============================================ */
ViperBigInt* vp_bigint_from_str_c(const char* s);
ViperBigInt* vp_bigint_from_i64_c(int64_t v);
const char* vp_bigint_to_str_c(ViperBigInt* bigint, int base);
void vp_bigint_add_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_sub_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_mul_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_div_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_mod_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_pow_c(ViperBigInt* result, ViperBigInt* base, ViperBigInt* exp);
void vp_bigint_sqrt_c(ViperBigInt* result, ViperBigInt* a);
void vp_bigint_abs_c(ViperBigInt* result, ViperBigInt* a);
void vp_bigint_neg_c(ViperBigInt* result, ViperBigInt* a);
void vp_bigint_invert_c(ViperBigInt* result, ViperBigInt* a);
void vp_bigint_and_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_or_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_xor_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_lshift_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
void vp_bigint_rshift_c(ViperBigInt* result, ViperBigInt* a, ViperBigInt* b);
bool vp_bigint_eq_c(ViperBigInt* a, ViperBigInt* b);
bool vp_bigint_lt_c(ViperBigInt* a, ViperBigInt* b);
bool vp_bigint_gt_c(ViperBigInt* a, ViperBigInt* b);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_GMP_BRIDGE_H */
