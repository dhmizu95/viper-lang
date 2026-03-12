/**
 * Viper Memoization Runtime - LRU Cache and Unbounded Cache
 * 
 * Provides C implementations for memoization decorators:
 * - @lru_cache(maxsize=N) - LRU eviction policy
 * - @cache - Unbounded cache (maxsize=None)
 */

#ifndef VIPER_MEMOIZATION_H
#define VIPER_MEMOIZATION_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Hash Map Node for Cache
// ============================================================================

/**
 * Hash map node - stores key-value pairs
 */
typedef struct CacheNode {
    void* key;              // Cached argument tuple
    void* value;            // Cached return value
    uint64_t key_hash;      // Pre-computed hash for faster lookup
    struct CacheNode* next; // Collision chain
} CacheNode;

/**
 * Hash map - underlying data structure for cache
 */
typedef struct HashMap {
    CacheNode** buckets;    // Bucket array
    size_t capacity;        // Number of buckets
    size_t size;            // Number of entries
} HashMap;

// ============================================================================
// LRU Cache (with eviction)
// ============================================================================

/**
 * LRU Cache Node - extends CacheNode with doubly-linked list for LRU tracking
 */
typedef struct LRUCacheNode {
    void* key;                  // Cached argument tuple
    void* value;                // Cached return value
    uint64_t key_hash;          // Pre-computed hash
    struct LRUCacheNode* prev;  // Previous in LRU order (older)
    struct LRUCacheNode* next;  // Next in LRU order (newer)
} LRUCacheNode;

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
 * @param key The key (argument tuple)
 * @return Pointer to cached value, or NULL if not found
 */
void* vp_lru_cache_get(LRUCache* cache, void* key);

/**
 * Set a value in the LRU cache
 * @param cache The cache
 * @param key The key (argument tuple)
 * @param value The value to cache
 */
void vp_lru_cache_set(LRUCache* cache, void* key, void* value);

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
 * @param key The key (argument tuple)
 * @return Pointer to cached value, or NULL if not found
 */
void* vp_cache_get(Cache* cache, void* key);

/**
 * Set a value in the unbounded cache
 * @param cache The cache
 * @param key The key (argument tuple)
 * @param value The value to cache
 */
void vp_cache_set(Cache* cache, void* key, void* value);

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
 * Create a single-element tuple for cache key
 * @param value The integer value
 * @return Pointer to the tuple (allocated memory)
 */
void* vp_tuple_create1(int64_t value);

/**
 * Create a two-element tuple for cache key
 * @param value1 First integer value
 * @param value2 Second integer value
 * @return Pointer to the tuple (allocated memory)
 */
void* vp_tuple_create2(int64_t value1, int64_t value2);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_MEMOIZATION_H */
