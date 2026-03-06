/**
 * Viper Optimization Macros
 * 
 * Branch prediction, inlining, and other performance hints
 */

#ifndef VIPER_OPTIMIZE_H
#define VIPER_OPTIMIZE_H

/* ============================================ */
/* Compiler Detection                           */
/* ============================================ */

#if defined(__GNUC__) || defined(__clang__)
    #define VIPER_GCC_COMPATIBLE 1
#else
    #define VIPER_GCC_COMPATIBLE 0
#endif

/* ============================================ */
/* Branch Prediction Hints                      */
/* ============================================ */

#if VIPER_GCC_COMPATIBLE
    /**
     * Hint that a condition is likely true
     * Helps CPU branch predictor optimize hot paths
     */
    #define VIPER_LIKELY(x)   __builtin_expect(!!(x), 1)
    
    /**
     * Hint that a condition is unlikely true
     * Helps CPU branch predictor optimize cold paths
     */
    #define VIPER_UNLIKELY(x) __builtin_expect(!!(x), 0)
#else
    #define VIPER_LIKELY(x)   (x)
    #define VIPER_UNLIKELY(x) (x)
#endif

/* ============================================ */
/* Function Inlining Hints                      */
/* ============================================ */

#if VIPER_GCC_COMPATIBLE
    /**
     * Force inlining of a function
     * Use for small, frequently-called functions
     */
    #define VIPER_ALWAYS_INLINE __attribute__((always_inline)) inline
    
    /**
     * Prevent inlining of a function
     * Use for large functions or to reduce code size
     */
    #define VIPER_NEVER_INLINE __attribute__((noinline))
    
    /**
     * Hint that a function is rarely called
     * Helps optimizer place cold code separately
     */
    #define VIPER_COLD __attribute__((cold))
    
    /**
     * Hint that a function is frequently called (hot path)
     */
    #define VIPER_HOT __attribute__((hot))
#else
    #define VIPER_ALWAYS_INLINE inline
    #define VIPER_NEVER_INLINE
    #define VIPER_COLD
    #define VIPER_HOT
#endif

/* ============================================ */
/* Memory Alignment Hints                       */
/* ============================================ */

#if VIPER_GCC_COMPATIBLE
    /**
     * Request minimum alignment for a variable/struct
     */
    #define VIPER_ALIGNED(n) __attribute__((aligned(n)))
    
    /**
     * Pack struct without padding
     */
    #define VIPER_PACKED __attribute__((packed))
#else
    #define VIPER_ALIGNED(n)
    #define VIPER_PACKED
#endif

/* ============================================ */
/* Cache Line Optimization                      */
/* ============================================ */

/* Typical cache line size on modern CPUs */
#define VIPER_CACHE_LINE_SIZE 64

/**
 * Pad struct to cache line boundary
 * Use to prevent false sharing in multi-threaded code
 */
#define VIPER_CACHE_PAD(size) char _pad[size]

/**
 * Align variable to cache line
 */
#define VIPER_CACHE_ALIGNED VIPER_ALIGNED(VIPER_CACHE_LINE_SIZE)

/* ============================================ */
/* Prefetch Hints                               */
/* ============================================ */

#if VIPER_GCC_COMPATIBLE
    /**
     * Prefetch data into cache
     * @param addr Address to prefetch
     * @param rw 0=read, 1=write
     * @param locality 0-3 (3=highest temporal locality)
     */
    #define VIPER_PREFETCH(addr, rw, locality) \
        __builtin_prefetch((addr), (rw), (locality))
#else
    #define VIPER_PREFETCH(addr, rw, locality) ((void)0)
#endif

/**
 * Prefetch for read (temporal locality = 3)
 */
#define VIPER_PREFETCH_READ(addr) VIPER_PREFETCH((addr), 0, 3)

/**
 * Prefetch for write (temporal locality = 3)
 */
#define VIPER_PREFETCH_WRITE(addr) VIPER_PREFETCH((addr), 1, 3)

/* ============================================ */
/* Loop Optimization Hints                      */
/* ============================================ */

#if VIPER_GCC_COMPATIBLE && (__GNUC__ > 9 || (__GNUC__ == 9 && __GNUC_MINOR__ >= 1))
    /**
     * Hint that a loop is unroll-friendly
     */
    #define VIPER_LOOP_UNROLL _Pragma("GCC unroll 4")
    
    /**
     * Hint that a loop should be vectorized
     */
    #define VIPER_LOOP_VECTORIZE _Pragma("GCC vectorize")
    
    /**
     * Hint that a loop should not be vectorized
     */
    #define VIPER_LOOP_NO_VECTORIZE _Pragma("GCC novector")
#else
    #define VIPER_LOOP_UNROLL
    #define VIPER_LOOP_VECTORIZE
    #define VIPER_LOOP_NO_VECTORIZE
#endif

/* ============================================ */
/* Fallthrough Attribute                        */
/* ============================================ */

#if VIPER_GCC_COMPATIBLE && __GNUC__ >= 7
    /**
     * Indicate intentional fallthrough in switch statements
     * Suppresses -Wimplicit-fallthrough warning
     */
    #define VIPER_FALLTHROUGH __attribute__((fallthrough))
#else
    #define VIPER_FALLTHROUGH ((void)0)
#endif

/* ============================================ */
/* Unused Variable Suppression                  */
/* ============================================ */

/**
 * Suppress unused variable warnings
 */
#define VIPER_UNUSED(x) ((void)(x))

/* ============================================ */
/* Fast Path / Slow Path Macros                 */
/* ============================================ */

/**
 * Mark code as fast path (likely executed)
 * Combines likely branch prediction with hot function attribute
 */
#define VIPER_FAST_PATH(cond) VIPER_LIKELY(cond)

/**
 * Mark code as slow path (unlikely executed)
 * Combines unlikely branch prediction with cold function attribute
 */
#define VIPER_SLOW_PATH(cond) VIPER_UNLIKELY(cond)

/* ============================================ */
/* Error Path Optimization                      */
/* ============================================ */

/**
 * Mark error handling code as unlikely
 * Use around error checks in hot paths
 * 
 * Example:
 *   if VIPER_ERROR_PATH(ptr == NULL) {
 *       return ERROR;
 *   }
 */
#define VIPER_ERROR_PATH(cond) VIPER_UNLIKELY(cond)

/**
 * Mark success path as likely
 * Use around success checks in hot paths
 */
#define VIPER_SUCCESS_PATH(cond) VIPER_LIKELY(cond)

/* ============================================ */
/* Bounds Check Optimization                    */
/* ============================================ */

/**
 * Mark bounds check as likely to pass
 * Use in hot loops where bounds are usually valid
 */
#define VIPER_BOUNDS_CHECK_LIKELY(idx, len) \
    VIPER_LIKELY((idx) >= 0 && (idx) < (len))

/**
 * Mark bounds check as unlikely to fail
 * Alternative formulation for bounds checking
 */
#define VIPER_BOUNDS_CHECK_UNLIKELY_FAIL(idx, len) \
    VIPER_UNLIKELY((idx) < 0 || (idx) >= (len))

#endif /* VIPER_OPTIMIZE_H */
