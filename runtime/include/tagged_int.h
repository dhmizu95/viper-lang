/**
 * Viper Tagged Integer Support
 *
 * Provides automatic promotion from small integers to BigInt on overflow.
 * Uses tagged pointer representation:
 * - LSB = 0: Small integer (i63, stored directly in pointer bits)
 * - LSB = 1: BigInt pointer (heap-allocated ViperBigInt)
 *
 * This allows Python-like arbitrary precision integers with minimal overhead
 * for small integer operations.
 */

#ifndef VIPER_TAGGED_INT_H
#define VIPER_TAGGED_INT_H

#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>
#include "viper_arc.h"
#include "gmp_bridge.h"

/* Forward declaration - ViperString is defined in viper_types.h */
typedef struct ViperString ViperString;

/* ============================================ */
/* Branch Prediction Hints                      */
/* ============================================ */

/* Use GCC/Clang branch prediction hints when available */
#if defined(__GNUC__) || defined(__clang__)
    #define VIPER_LIKELY(x)   __builtin_expect(!!(x), 1)
    #define VIPER_UNLIKELY(x) __builtin_expect(!!(x), 0)
#else
    #define VIPER_LIKELY(x)   (x)
    #define VIPER_UNLIKELY(x) (x)
#endif

/* ============================================ */
/* Tagged Integer Representation                */
/* ============================================ */

/**
 * TaggedInt is represented as a 64-bit value:
 * - If LSB is 0: The value is a small integer (i63), stored as (value << 1)
 * - If LSB is 1: The value is a pointer to ViperBigInt (pointer | 1)
 * 
 * This gives us:
 * - Small int range: -2^62 to 2^62-1 (about ±4.6 quintillion)
 * - BigInt: Arbitrary precision when needed
 */
typedef uint64_t TaggedInt;

/* Tag bit constants */
#define TAGGED_INT_SMALL 0  /* LSB = 0: small integer */
#define TAGGED_INT_BIGINT  1  /* LSB = 1: BigInt pointer */

/* Mask for extracting small integer value */
#define TAGGED_INT_VALUE_MASK 0xFFFFFFFFFFFFFFFEULL

/* Maximum/minimum small integer values */
#define TAGGED_INT_MAX_SMALL ((1LL << 62) - 1)
#define TAGGED_INT_MIN_SMALL (-(1LL << 62))

/* ============================================ */
/* TaggedInt Operations                         */
/* ============================================ */

/**
 * Create a TaggedInt from a small integer
 */
static inline TaggedInt tagged_int_from_i64(int64_t value) {
    return ((uint64_t)value << 1) | TAGGED_INT_SMALL;
}

/* Exported version for LLVM codegen */
TaggedInt tagged_int_from_i64_export(int64_t value);

/**
 * Create a TaggedInt from a BigInt
 */
static inline TaggedInt tagged_int_from_bigint(ViperBigInt* bigint) {
    return ((uint64_t)bigint) | TAGGED_INT_BIGINT;
}

/**
 * Check if a TaggedInt is a small integer
 */
static inline bool tagged_int_is_small(TaggedInt value) {
    return (value & 1) == TAGGED_INT_SMALL;
}

/**
 * Check if a TaggedInt is a BigInt
 */
static inline bool tagged_int_is_bigint(TaggedInt value) {
    return (value & 1) == TAGGED_INT_BIGINT;
}

/**
 * Get the i64 value from a small TaggedInt (undefined if BigInt)
 */
static inline int64_t tagged_int_get_small(TaggedInt value) {
    return ((int64_t)value) >> 1;
}

/**
 * Get the BigInt pointer from a TaggedInt (undefined if small int)
 */
static inline ViperBigInt* tagged_int_get_bigint(TaggedInt value) {
    return (ViperBigInt*)(value & ~TAGGED_INT_BIGINT);
}

/**
 * Check if adding two small integers would overflow
 */
static inline bool would_overflow_add(int64_t a, int64_t b) {
    if (b > 0 && a > TAGGED_INT_MAX_SMALL - b) return true;
    if (b < 0 && a < TAGGED_INT_MIN_SMALL - b) return true;
    return false;
}

/**
 * Check if subtracting two small integers would overflow
 */
static inline bool would_overflow_sub(int64_t a, int64_t b) {
    if (b < 0 && a > TAGGED_INT_MAX_SMALL + b) return true;
    if (b > 0 && a < TAGGED_INT_MIN_SMALL + b) return true;
    return false;
}

/**
 * Check if multiplying two small integers would overflow
 */
static inline bool would_overflow_mul(int64_t a, int64_t b) {
    if (a == 0 || b == 0) return false;
    if (a == 1) return false;
    if (b == 1) return false;
    if (a == -1) return b == TAGGED_INT_MIN_SMALL;
    if (b == -1) return a == TAGGED_INT_MIN_SMALL;
    
    /* Check using division to avoid overflow in multiplication */
    if (a > 0) {
        if (b > 0) {
            return a > TAGGED_INT_MAX_SMALL / b;
        } else {
            return b < TAGGED_INT_MIN_SMALL / a;
        }
    } else {
        if (b > 0) {
            return a < TAGGED_INT_MIN_SMALL / b;
        } else {
            return a != 0 && b < TAGGED_INT_MAX_SMALL / a;
        }
    }
}

/**
 * Promote a small TaggedInt to BigInt
 */
TaggedInt tagged_int_promote_to_bigint(TaggedInt value);

/**
 * Create a TaggedInt from a string representation
 */
TaggedInt tagged_int_from_str(const char* str);

/**
 * Convert a TaggedInt to BigInt (may allocate if small int)
 */
ViperBigInt* tagged_int_to_bigint(TaggedInt value);

/**
 * Add two TaggedInts with automatic overflow detection
 */
TaggedInt tagged_int_add(TaggedInt a, TaggedInt b);

/**
 * Subtract two TaggedInts with automatic overflow detection
 */
TaggedInt tagged_int_sub(TaggedInt a, TaggedInt b);

/**
 * Multiply two TaggedInts with automatic overflow detection
 */
TaggedInt tagged_int_mul(TaggedInt a, TaggedInt b);

/**
 * Divide two TaggedInts
 */
TaggedInt tagged_int_div(TaggedInt a, TaggedInt b);

/**
 * Modulo of two TaggedInts
 */
TaggedInt tagged_int_mod(TaggedInt a, TaggedInt b);

/**
 * Power operation: base ^ exp
 */
TaggedInt tagged_int_pow(TaggedInt base, TaggedInt exp);

/**
 * Compare two TaggedInts (returns -1, 0, or 1)
 */
int tagged_int_cmp(TaggedInt a, TaggedInt b);

/**
 * Check if two TaggedInts are equal
 */
bool tagged_int_eq(TaggedInt a, TaggedInt b);

/**
 * Check if TaggedInt a < b
 */
bool tagged_int_lt(TaggedInt a, TaggedInt b);

/**
 * Check if TaggedInt a > b
 */
bool tagged_int_gt(TaggedInt a, TaggedInt b);

/**
 * Negate a TaggedInt
 */
TaggedInt tagged_int_neg(TaggedInt a);

/**
 * Convert TaggedInt to ViperString (for str() builtin)
 * Returns a properly allocated ViperString*
 */
void* tagged_int_to_str(TaggedInt value);

/**
 * Print a TaggedInt
 */
void tagged_int_print(TaggedInt value);

/**
 * Increment reference count of a TaggedInt (if it's a BigInt)
 */
void tagged_int_retain(TaggedInt value);

/**
 * Decrement reference count of a TaggedInt (if it's a BigInt)
 */
void tagged_int_release(TaggedInt value);

/**
 * Free a TaggedInt if it's a BigInt (alias for tagged_int_release)
 */
void tagged_int_free(TaggedInt value);

/* ============================================ */
/* Unified Runtime Dispatcher Declarations      */
/* ============================================ */
/* These provide a consistent interface for runtime operations */

TaggedInt vp_runtime_add(TaggedInt a, TaggedInt b);
TaggedInt vp_runtime_sub(TaggedInt a, TaggedInt b);
TaggedInt vp_runtime_mul(TaggedInt a, TaggedInt b);
TaggedInt vp_runtime_div(TaggedInt a, TaggedInt b);
TaggedInt vp_runtime_mod(TaggedInt a, TaggedInt b);
int vp_runtime_cmp(TaggedInt a, TaggedInt b);
bool vp_runtime_eq(TaggedInt a, TaggedInt b);
bool vp_runtime_lt(TaggedInt a, TaggedInt b);
bool vp_runtime_gt(TaggedInt a, TaggedInt b);
TaggedInt vp_runtime_neg(TaggedInt a);

#endif /* VIPER_TAGGED_INT_H */
