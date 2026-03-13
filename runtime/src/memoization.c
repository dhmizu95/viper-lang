/**
 * Viper Memoization Runtime Implementation
 *
 * Implements LRU cache and unbounded cache for memoization decorators.
 * Uses ARC (Automatic Reference Counting) for memory management.
 */

#include "memoization.h"
#include "viper_arc.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ============================================================================
// Configuration
// ============================================================================

#define INITIAL_HASHMAP_CAPACITY 64
#define HASHMAP_LOAD_FACTOR 0.75

// Forward declarations
static int hashmap_resize(HashMap* map);
static LRUCacheNode* lru_hashmap_get(HashMap* map, uint64_t hash, const ARCCacheKey* key);
static void lru_hashmap_remove_no_free(HashMap* map, uint64_t hash, const ARCCacheKey* key);
static int lru_hashmap_resize(HashMap* map);
static void lru_hashmap_set(HashMap* map, uint64_t hash, LRUCacheNode* lru_node);

// ============================================================================
// ARC Key Creation Functions
// ============================================================================

ARCCacheKey* arc_key_create1(int64_t value) {
    size_t data_size = sizeof(int64_t);
    size_t total_size = sizeof(ARCCacheKey) + data_size;
    
    // Use thread-local allocation (fast path, non-atomic ref count)
    ARCCacheKey* key = (ARCCacheKey*)vp_arc_alloc_local(total_size);
    if (!key) return NULL;
    
    key->key_size = data_size;
    key->hash = vp_hash_int(value);
    key->data[0] = value;
    
    return key;
}

ARCCacheKey* arc_key_create2(int64_t value1, int64_t value2) {
    size_t data_size = 2 * sizeof(int64_t);
    size_t total_size = sizeof(ARCCacheKey) + data_size;

    ARCCacheKey* key = (ARCCacheKey*)vp_arc_alloc_local(total_size);
    if (!key) return NULL;

    int64_t values[2] = {value1, value2};
    key->key_size = data_size;
    key->hash = vp_hash_tuple(values, 2);
    key->data[0] = value1;
    key->data[1] = value2;

    return key;
}

ARCCacheKey* arc_key_create3(int64_t v1, int64_t v2, int64_t v3) {
    int64_t values[3] = {v1, v2, v3};
    return arc_key_create_n(values, 3);
}

ARCCacheKey* arc_key_create4(int64_t v1, int64_t v2, int64_t v3, int64_t v4) {
    int64_t values[4] = {v1, v2, v3, v4};
    return arc_key_create_n(values, 4);
}

ARCCacheKey* arc_key_create5(int64_t v1, int64_t v2, int64_t v3, int64_t v4, int64_t v5) {
    int64_t values[5] = {v1, v2, v3, v4, v5};
    return arc_key_create_n(values, 5);
}

ARCCacheKey* arc_key_create6(int64_t v1, int64_t v2, int64_t v3, int64_t v4, int64_t v5, int64_t v6) {
    int64_t values[6] = {v1, v2, v3, v4, v5, v6};
    return arc_key_create_n(values, 6);
}

ARCCacheKey* arc_key_create7(int64_t v1, int64_t v2, int64_t v3, int64_t v4, int64_t v5, int64_t v6, int64_t v7) {
    int64_t values[7] = {v1, v2, v3, v4, v5, v6, v7};
    return arc_key_create_n(values, 7);
}

ARCCacheKey* arc_key_create8(int64_t v1, int64_t v2, int64_t v3, int64_t v4, int64_t v5, int64_t v6, int64_t v7, int64_t v8) {
    int64_t values[8] = {v1, v2, v3, v4, v5, v6, v7, v8};
    return arc_key_create_n(values, 8);
}

ARCCacheKey* arc_key_create_n(const int64_t* values, size_t count) {
    if (count < 3 || count > 8) {
        fprintf(stderr, "arc_key_create_n: count must be 3-8, got %zu\n", count);
        return NULL;
    }
    
    size_t data_size = count * sizeof(int64_t);
    size_t total_size = sizeof(ARCCacheKey) + data_size;
    
    ARCCacheKey* key = (ARCCacheKey*)vp_arc_alloc_local(total_size);
    if (!key) return NULL;
    
    key->key_size = data_size;
    key->hash = vp_hash_tuple(values, count);
    
    memcpy(key->data, values, count * sizeof(int64_t));
    
    return key;
}

// Backward compatibility wrappers
void* vp_tuple_create1(int64_t value) {
    return arc_key_create1(value);
}

void* vp_tuple_create2(int64_t value1, int64_t value2) {
    return arc_key_create2(value1, value2);
}

// ============================================================================
// Hash Map Implementation
// ============================================================================

static HashMap* hashmap_create(size_t capacity) {
    HashMap* map = (HashMap*)malloc(sizeof(HashMap));
    if (!map) return NULL;

    // Round up to power of 2 for fast modulo
    size_t pow2_capacity = 1;
    while (pow2_capacity < capacity) {
        pow2_capacity <<= 1;
    }

    map->buckets = (CacheNode**)calloc(pow2_capacity, sizeof(CacheNode*));
    if (!map->buckets) {
        free(map);
        return NULL;
    }

    map->capacity = pow2_capacity;
    map->capacity_mask = pow2_capacity - 1;  // For fast bitwise AND
    map->size = 0;
    return map;
}

static void hashmap_destroy(HashMap* map) {
    if (!map) return;

    for (size_t i = 0; i < map->capacity; i++) {
        CacheNode* node = map->buckets[i];
        while (node) {
            CacheNode* next = node->next;
            arc_key_release(node->key);  // ARC release
            free(node);
            node = next;
        }
    }

    free(map->buckets);
    free(map);
}

static CacheNode* hashmap_get(HashMap* map, uint64_t hash, const ARCCacheKey* key) {
    size_t index = hash & map->capacity_mask;  // Fast! (~1 cycle vs ~20-50)
    CacheNode* node = map->buckets[index];

    while (node) {
        // Compare hash first, then compare key data
        if (node->key->hash == hash && 
            node->key->key_size == key->key_size &&
            memcmp(node->key->data, key->data, key->key_size) == 0) {
            return node;
        }
        node = node->next;
    }

    return NULL;
}

static void hashmap_set(HashMap* map, uint64_t hash, ARCCacheKey* key, 
                        int64_t value, int is_bigint) {
    // Check load factor and resize if needed
    if ((double)(map->size + 1) / map->capacity > HASHMAP_LOAD_FACTOR) {
        hashmap_resize(map);
    }

    size_t index = hash & map->capacity_mask;

    // Check if key exists
    CacheNode* existing = hashmap_get(map, hash, key);
    if (existing) {
        existing->value = value;
        existing->is_bigint = is_bigint;
        arc_key_release(key);  // Release caller's reference (key not needed)
        return;
    }

    // Create new node
    CacheNode* node = (CacheNode*)malloc(sizeof(CacheNode));
    if (!node) {
        arc_key_release(key);  // Clean up on failure
        return;
    }

    node->key = key;  // Takes ownership
    node->value = value;
    node->is_bigint = is_bigint;
    node->next = map->buckets[index];

    map->buckets[index] = node;
    map->size++;
}

static int hashmap_resize(HashMap* map) {
    size_t new_capacity = map->capacity << 1;  // Always power of 2
    CacheNode** new_buckets = (CacheNode**)calloc(new_capacity, sizeof(CacheNode*));
    if (!new_buckets) return -1;

    // Rehash all entries
    for (size_t i = 0; i < map->capacity; i++) {
        CacheNode* node = map->buckets[i];
        while (node) {
            CacheNode* next = node->next;
            size_t new_index = node->key->hash & (new_capacity - 1);  // Fast modulo
            node->next = new_buckets[new_index];
            new_buckets[new_index] = node;
            node = next;
        }
    }

    free(map->buckets);
    map->buckets = new_buckets;
    map->capacity = new_capacity;
    map->capacity_mask = new_capacity - 1;

    return 0;
}

static LRUCacheNode* lru_hashmap_get(HashMap* map, uint64_t hash, const ARCCacheKey* key) {
    size_t index = hash & map->capacity_mask;
    LRUCacheNode* node = (LRUCacheNode*)map->buckets[index];

    while (node) {
        if (node->key->hash == hash &&
            node->key->key_size == key->key_size &&
            memcmp(node->key->data, key->data, key->key_size) == 0) {
            return node;
        }
        node = node->hash_next;
    }

    return NULL;
}

static void lru_hashmap_remove_no_free(HashMap* map, uint64_t hash, const ARCCacheKey* key) {
    size_t index = hash & map->capacity_mask;
    LRUCacheNode* node = (LRUCacheNode*)map->buckets[index];
    LRUCacheNode* prev = NULL;

    while (node) {
        if (node->key->hash == hash &&
            node->key->key_size == key->key_size &&
            memcmp(node->key->data, key->data, key->key_size) == 0) {
            if (prev) {
                prev->hash_next = node->hash_next;
            } else {
                map->buckets[index] = (CacheNode*)node->hash_next;
            }
            map->size--;
            return;
        }
        prev = node;
        node = node->hash_next;
    }
}

static int lru_hashmap_resize(HashMap* map) {
    size_t new_capacity = map->capacity << 1;
    CacheNode** new_buckets = (CacheNode**)calloc(new_capacity, sizeof(CacheNode*));
    if (!new_buckets) return -1;

    for (size_t i = 0; i < map->capacity; i++) {
        LRUCacheNode* node = (LRUCacheNode*)map->buckets[i];
        while (node) {
            LRUCacheNode* next = node->hash_next;
            size_t new_index = node->key->hash & (new_capacity - 1);
            node->hash_next = (LRUCacheNode*)new_buckets[new_index];
            new_buckets[new_index] = (CacheNode*)node;
            node = next;
        }
    }

    free(map->buckets);
    map->buckets = new_buckets;
    map->capacity = new_capacity;
    map->capacity_mask = new_capacity - 1;
    return 0;
}

static void lru_hashmap_set(HashMap* map, uint64_t hash, LRUCacheNode* lru_node) {
    if ((double)(map->size + 1) / map->capacity > HASHMAP_LOAD_FACTOR) {
        lru_hashmap_resize(map);
    }

    size_t index = hash & map->capacity_mask;
    lru_node->hash_next = (LRUCacheNode*)map->buckets[index];
    map->buckets[index] = (CacheNode*)lru_node;
    map->size++;
}

// ============================================================================
// LRU Cache Implementation
// ============================================================================

LRUCache* vp_lru_cache_create(size_t maxsize) {
    LRUCache* cache = (LRUCache*)malloc(sizeof(LRUCache));
    if (!cache) return NULL;

    cache->map = hashmap_create(INITIAL_HASHMAP_CAPACITY);
    if (!cache->map) {
        free(cache);
        return NULL;
    }

    cache->maxsize = maxsize;
    cache->currsize = 0;
    cache->head = NULL;
    cache->tail = NULL;

    return cache;
}

static void lru_cache_move_to_head(LRUCache* cache, LRUCacheNode* node) {
    if (node == cache->head) return;  // Already at head

    // Remove from current position
    if (node->lru_prev) node->lru_prev->lru_next = node->lru_next;
    if (node->lru_next) node->lru_next->lru_prev = node->lru_prev;

    if (node == cache->tail) {
        cache->tail = node->lru_prev;
    }

    // Move to head
    node->lru_prev = NULL;
    node->lru_next = cache->head;
    if (cache->head) {
        cache->head->lru_prev = node;
    }
    cache->head = node;

    if (!cache->tail) {
        cache->tail = node;
    }
}

static void lru_cache_evict(LRUCache* cache) {
    if (!cache->tail) return;

    LRUCacheNode* node = cache->tail;

    // Remove from linked list
    if (node->lru_prev) {
        node->lru_prev->lru_next = NULL;
        cache->tail = node->lru_prev;
    } else {
        cache->head = NULL;
        cache->tail = NULL;
    }

    // Remove from hash map (just removes pointer, doesn't free key)
    lru_hashmap_remove_no_free(cache->map, node->key->hash, node->key);

    // Release key reference (triggers ARC cleanup)
    arc_key_release(node->key);

    // Free node
    free(node);

    cache->currsize--;
}

int64_t vp_lru_cache_get(LRUCache* cache, ARCCacheKey* key, int* found, int* is_bigint) {
    if (!cache || !key) {
        if (found) *found = 0;
        return 0;
    }

    uint64_t hash = key->hash;

    LRUCacheNode* lru_node = lru_hashmap_get(cache->map, hash, key);
    if (!lru_node) {
        if (found) *found = 0;
        return 0;
    }

    // Move to head (LRU update)
    lru_cache_move_to_head(cache, lru_node);

    if (found) *found = 1;
    if (is_bigint) *is_bigint = lru_node->is_bigint;
    return lru_node->value;  // Return value directly (i64 or BigInt pointer)
}

void vp_lru_cache_set(LRUCache* cache, ARCCacheKey* key, int64_t value, int is_bigint) {
    if (!cache || !key) return;

    uint64_t hash = key->hash;

    // Check if key exists
    LRUCacheNode* lru_node = lru_hashmap_get(cache->map, hash, key);
    if (lru_node) {
        lru_node->value = value;
        lru_node->is_bigint = is_bigint;
        lru_cache_move_to_head(cache, lru_node);
        arc_key_release(key);  // Release caller's reference
        return;
    }

    // Evict if necessary
    if (cache->maxsize > 0 && cache->currsize >= cache->maxsize) {
        lru_cache_evict(cache);
    }

    // Create new LRU node
    LRUCacheNode* node = (LRUCacheNode*)malloc(sizeof(LRUCacheNode));
    if (!node) {
        arc_key_release(key);  // Clean up on failure
        return;
    }

    node->key = key;  // Takes ownership (ref count still 1)
    node->value = value;
    node->is_bigint = is_bigint;
    node->hash_next = NULL;
    node->lru_prev = NULL;
    node->lru_next = cache->head;

    // Update linked list
    if (cache->head) cache->head->lru_prev = node;
    cache->head = node;
    if (!cache->tail) cache->tail = node;

    // Add to hash map (transfers ownership)
    lru_hashmap_set(cache->map, hash, node);
    cache->currsize++;
}

void vp_lru_cache_destroy(LRUCache* cache) {
    if (!cache) return;

    // Free all LRU nodes (keys released via ARC)
    LRUCacheNode* node = cache->head;
    while (node) {
        LRUCacheNode* next = node->lru_next;
        arc_key_release(node->key);  // Release key (triggers free)
        free(node);
        node = next;
    }

    // Free hash map structure
    free(cache->map->buckets);
    free(cache->map);
    free(cache);
}

void vp_lru_cache_clear(LRUCache* cache) {
    if (!cache) return;

    // Free all LRU nodes (keys released via ARC)
    LRUCacheNode* node = cache->head;
    while (node) {
        LRUCacheNode* next = node->lru_next;
        arc_key_release(node->key);
        free(node);
        node = next;
    }

    // Clear hash map
    memset(cache->map->buckets, 0, cache->map->capacity * sizeof(CacheNode*));
    cache->map->size = 0;
    cache->currsize = 0;
    cache->head = NULL;
    cache->tail = NULL;
}

void vp_lru_cache_info(LRUCache* cache, size_t* hits, size_t* misses) {
    // Stub - would need to track hits/misses in cache struct
    (void)cache;  // Suppress unused warning
    if (hits) *hits = 0;
    if (misses) *misses = 0;
}

// ============================================================================
// Unbounded Cache Implementation
// ============================================================================

Cache* vp_cache_create(void) {
    Cache* cache = (Cache*)malloc(sizeof(Cache));
    if (!cache) return NULL;

    cache->map = hashmap_create(INITIAL_HASHMAP_CAPACITY);
    if (!cache->map) {
        free(cache);
        return NULL;
    }

    cache->currsize = 0;
    return cache;
}

int64_t vp_cache_get(Cache* cache, ARCCacheKey* key, int* found, int* is_bigint) {
    if (!cache || !key) {
        if (found) *found = 0;
        return 0;
    }

    uint64_t hash = key->hash;

    CacheNode* node = hashmap_get(cache->map, hash, key);
    if (!node) {
        if (found) *found = 0;
        return 0;
    }

    if (found) *found = 1;
    if (is_bigint) *is_bigint = node->is_bigint;
    return node->value;
}

void vp_cache_set(Cache* cache, ARCCacheKey* key, int64_t value, int is_bigint) {
    if (!cache || !key) return;

    uint64_t hash = key->hash;
    hashmap_set(cache->map, hash, key, value, is_bigint);
    cache->currsize++;
}

void vp_cache_destroy(Cache* cache) {
    if (!cache) return;

    hashmap_destroy(cache->map);  // Frees all keys via ARC
    free(cache);
}

void vp_cache_clear(Cache* cache) {
    if (!cache) return;

    // Free all keys via hashmap_destroy
    hashmap_destroy(cache->map);
    
    // Reinitialize hash map
    cache->map = hashmap_create(INITIAL_HASHMAP_CAPACITY);
    cache->currsize = 0;
}

// ============================================================================
// Utility Functions
// ============================================================================

uint64_t vp_hash_int(int64_t key) {
    // Simple but effective hash for integers
    uint64_t hash = (uint64_t)key;
    hash = (hash ^ (hash >> 30)) * 0xbf58476d1ce4e5b9ULL;
    hash = (hash ^ (hash >> 27)) * 0x94d049bb133111ebULL;
    hash = hash ^ (hash >> 31);
    return hash;
}

uint64_t vp_hash_tuple(const int64_t* values, size_t count) {
    // FNV-1a hash for tuples
    uint64_t hash = 14695981039346656037ULL;

    for (size_t i = 0; i < count; i++) {
        hash ^= (uint64_t)values[i];
        hash *= 1099511628211ULL;
    }

    return hash;
}
