/**
 * Viper Memoization Runtime - LRU Cache and Unbounded Cache
 *
 * Provides C implementations for memoization decorators:
 * - @lru_cache(maxsize=N) - LRU eviction policy
 * - @cache - Unbounded cache (maxsize=None)
 *
 * Uses ARC (Automatic Reference Counting) for memory management.
 */

#ifndef VIPER_MEMOIZATION_H
#define VIPER_MEMOIZATION_H

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include "viper_arc.h"

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// ARC Cache Key
// ============================================================================

/**
 * ARC-managed cache key
 * Objects allocated with vp_arc_alloc_local(), managed by reference counting
 * 
 * Memory layout:
 *   [ViperHeader][key_size][hash][data...]
 *    ^                                   
 *    |                                   
 *  returned from vp_arc_alloc_local()
 */
typedef struct ARCCacheKey {
    ViperHeader header;     // ARC header (ref count, destructor, flags)
    int64_t key_size;       // Size of key data in bytes
    uint64_t hash;          // Pre-computed hash for fast lookup
    int64_t data[];         // Flexible array: [value1, value2, ...]
} ARCCacheKey;

/**
 * Create an ARC cache key with 1 value
 * @param value The integer value
 * @return ARCCacheKey* with ref_count=1, or NULL on failure
 */
ARCCacheKey* arc_key_create1(int64_t value);

/**
 * Create an ARC cache key with 2 values
 * @param value1 First value
 * @param value2 Second value
 * @return ARCCacheKey* with ref_count=1, or NULL on failure
 */
ARCCacheKey* arc_key_create2(int64_t value1, int64_t value2);

/**
 * Create an ARC cache key with N values (3-8 parameters)
 * @param values Array of values
 * @param count Number of values (3-8)
 * @return ARCCacheKey* with ref_count=1, or NULL on failure
 */
ARCCacheKey* arc_key_create_n(const int64_t* values, size_t count);

/**
 * Convenience: retain a key (increment ref count)
 */
static inline void arc_key_retain(ARCCacheKey* key) {
    if (!key) return;
    vp_arc_retain_local(key);  // Cache keys are thread-local
}

/**
 * Convenience: release a key (decrement ref count, free if zero)
 */
static inline void arc_key_release(ARCCacheKey* key) {
    if (!key) return;
    vp_arc_release_local(key);
}

// ============================================================================
// Updated Cache Node Structures (with ARC keys)
// ============================================================================

/**
 * Hash map node - stores ARC-managed key and value
 */
typedef struct CacheNode {
    ARCCacheKey* key;       // ARC-managed key
    int64_t value;          // Cached return value (i64 or BigInt pointer)
    int is_bigint;          // 1 if value is BigInt pointer, 0 if i64
    struct CacheNode* next; // Collision chain
} CacheNode;

/**
 * LRU Cache Node - extends CacheNode with doubly-linked list for LRU tracking
 */
typedef struct LRUCacheNode {
    ARCCacheKey* key;       // ARC-managed key
    int64_t value;          // Cached return value (i64 or BigInt pointer)
    int is_bigint;          // 1 if value is BigInt pointer, 0 if i64
    struct LRUCacheNode* hash_next; // Collision chain in hash map bucket
    struct LRUCacheNode* lru_prev;  // Previous in LRU order (older)
    struct LRUCacheNode* lru_next;  // Next in LRU order (newer)
} LRUCacheNode;

/**
 * Hash map - underlying data structure for cache
 * Uses power-of-2 capacity for fast modulo via bitwise AND
 */
typedef struct HashMap {
    CacheNode** buckets;    // Bucket array
    size_t capacity;        // Number of buckets (always power of 2)
    size_t capacity_mask;   // capacity - 1 for fast bitwise AND
    size_t size;            // Number of entries
} HashMap;

// ============================================================================
// LRU Cache (with eviction)
// ============================================================================

/**
 * LRU Cache - cache with least-recently-used eviction policy
 */
typedef struct LRUCache {
    size_t maxsize;             // Maximum entries (0 = unlimited)
    size_t currsize;            // Current entry count
    LRUCacheNode* head;         // Most recently used
    LRUCacheNode* tail;         // Least recently used
    HashMap* map;               // Key -> node mapping
} LRUCache;

// ============================================================================
// Unbounded Cache (no eviction)
// ============================================================================

/**
 * Unbounded Cache - simple cache without eviction
 */
typedef struct Cache {
    size_t currsize;            // Current entry count (for statistics)
    HashMap* map;               // Key -> value mapping
} Cache;

// ============================================================================
// LRU Cache Functions
// ============================================================================

/**
 * Create a new LRU cache
 * @param maxsize Maximum number of entries (0 = unlimited)
 * @return Pointer to the new cache, or NULL on failure
 */
LRUCache* vp_lru_cache_create(size_t maxsize);

/**
 * Get a value from the LRU cache
 * @param cache The cache
 * @param key The key (ARCCacheKey*)
 * @param found Output: pointer to int that will be set to 1 if found, 0 if not
 * @param is_bigint Output: pointer to int that will be set to 1 if value is BigInt, 0 if i64
 * @return Cached value as int64_t (for i64) or pointer (for BigInt). Check is_bigint to interpret.
 */
int64_t vp_lru_cache_get(LRUCache* cache, ARCCacheKey* key, int* found, int* is_bigint);

/**
 * Set a value in the LRU cache
 * @param cache The cache
 * @param key The key (ARCCacheKey*) - ownership transferred to cache
 * @param value The value to cache (i64 value or BigInt pointer)
 * @param is_bigint 1 if value is BigInt pointer, 0 if i64
 */
void vp_lru_cache_set(LRUCache* cache, ARCCacheKey* key, int64_t value, int is_bigint);

/**
 * Destroy an LRU cache and free all memory
 * @param cache The cache to destroy
 */
void vp_lru_cache_destroy(LRUCache* cache);

/**
 * Clear all entries from an LRU cache
 * @param cache The cache to clear
 */
void vp_lru_cache_clear(LRUCache* cache);

/**
 * Get cache statistics
 * @param cache The cache
 * @param hits Output: number of cache hits
 * @param misses Output: number of cache misses
 */
void vp_lru_cache_info(LRUCache* cache, size_t* hits, size_t* misses);

// ============================================================================
// Unbounded Cache Functions
// ============================================================================

/**
 * Create a new unbounded cache
 * @return Pointer to the new cache, or NULL on failure
 */
Cache* vp_cache_create(void);

/**
 * Get a value from the unbounded cache
 * @param cache The cache
 * @param key The key (ARCCacheKey*)
 * @param found Output: pointer to int that will be set to 1 if found, 0 if not
 * @param is_bigint Output: pointer to int that will be set to 1 if value is BigInt, 0 if i64
 * @return Cached value as int64_t (for i64) or pointer (for BigInt). Check is_bigint to interpret.
 */
int64_t vp_cache_get(Cache* cache, ARCCacheKey* key, int* found, int* is_bigint);

/**
 * Set a value in the unbounded cache
 * @param cache The cache
 * @param key The key (ARCCacheKey*) - ownership transferred to cache
 * @param value The value to cache (i64 value or BigInt pointer)
 * @param is_bigint 1 if value is BigInt pointer, 0 if i64
 */
void vp_cache_set(Cache* cache, ARCCacheKey* key, int64_t value, int is_bigint);

/**
 * Destroy an unbounded cache and free all memory
 * @param cache The cache to destroy
 */
void vp_cache_destroy(Cache* cache);

/**
 * Clear all entries from an unbounded cache
 * @param cache The cache to clear
 */
void vp_cache_clear(Cache* cache);

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Compute hash for a 64-bit integer key
 * @param key The integer key
 * @return Hash value
 */
uint64_t vp_hash_int(int64_t key);

/**
 * Compute hash for a tuple of integers
 * @param values Array of integer values
 * @param count Number of values
 * @return Hash value
 */
uint64_t vp_hash_tuple(const int64_t* values, size_t count);

/**
 * Create a single-element tuple for cache key (backward compatibility wrapper)
 * @param value The integer value
 * @return Pointer to ARCCacheKey (allocated with ARC)
 */
void* vp_tuple_create1(int64_t value);

/**
 * Create a two-element tuple for cache key (backward compatibility wrapper)
 * @param value1 First integer value
 * @param value2 Second integer value
 * @return Pointer to ARCCacheKey (allocated with ARC)
 */
void* vp_tuple_create2(int64_t value1, int64_t value2);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_MEMOIZATION_H */
