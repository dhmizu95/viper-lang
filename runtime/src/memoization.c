/**
 * Viper Memoization Runtime Implementation
 * 
 * Implements LRU cache and unbounded cache for memoization decorators.
 */

#include "memoization.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ============================================================================
// Configuration
// ============================================================================

#define INITIAL_HASHMAP_CAPACITY 64
#define HASHMAP_LOAD_FACTOR 0.75

// ============================================================================
// Hash Map Implementation
// ============================================================================

static HashMap* hashmap_create(size_t capacity) {
    HashMap* map = (HashMap*)malloc(sizeof(HashMap));
    if (!map) return NULL;
    
    map->buckets = (CacheNode**)calloc(capacity, sizeof(CacheNode*));
    if (!map->buckets) {
        free(map);
        return NULL;
    }
    
    map->capacity = capacity;
    map->size = 0;
    return map;
}

static void hashmap_destroy(HashMap* map) {
    if (!map) return;
    
    for (size_t i = 0; i < map->capacity; i++) {
        CacheNode* node = map->buckets[i];
        while (node) {
            CacheNode* next = node->next;
            free(node->key);  // Free the key
            free(node);       // Free the node (value is stored directly)
            node = next;
        }
    }
    
    free(map->buckets);
    free(map);
}

static CacheNode* hashmap_get(HashMap* map, uint64_t hash, const void* key) {
    size_t index = hash % map->capacity;
    CacheNode* node = map->buckets[index];
    
    while (node) {
        // Compare hash first, then compare full key data (hash + value = 2 int64_t)
        if (node->key_hash == hash && memcmp(node->key, key, 2 * sizeof(int64_t)) == 0) {
            return node;
        }
        node = node->next;
    }
    
    return NULL;
}

// Forward declaration
static int hashmap_resize(HashMap* map);

static void hashmap_set_lru(HashMap* map, uint64_t hash, void* key, LRUCacheNode* lru_node) {
    // Check load factor and resize if needed
    if ((double)(map->size + 1) / map->capacity > HASHMAP_LOAD_FACTOR) {
        hashmap_resize(map);
    }
    
    size_t index = hash % map->capacity;
    
    // Check if key already exists (using base CacheNode fields)
    CacheNode* existing = hashmap_get(map, hash, key);
    if (existing) {
        // Update by replacing the node in the bucket
        LRUCacheNode* existing_lru = (LRUCacheNode*)existing;
        // Copy value from new node
        existing_lru->value = lru_node->value;
        return;
    }
    
    // Use the lru_node directly (it starts with CacheNode-compatible fields)
    lru_node->key = key;
    lru_node->key_hash = hash;
    ((CacheNode*)lru_node)->next = map->buckets[index];
    
    map->buckets[index] = (CacheNode*)lru_node;
    map->size++;
}

static void hashmap_set(HashMap* map, uint64_t hash, void* key, int64_t value) {
    // Check load factor and resize if needed
    if ((double)(map->size + 1) / map->capacity > HASHMAP_LOAD_FACTOR) {
        hashmap_resize(map);
    }
    
    size_t index = hash % map->capacity;
    
    // Check if key already exists
    CacheNode* existing = hashmap_get(map, hash, key);
    if (existing) {
        existing->value = value;  // Update value directly
        return;
    }
    
    // Create new node
    CacheNode* node = (CacheNode*)malloc(sizeof(CacheNode));
    if (!node) return;
    
    node->key = key;
    node->value = value;  // Store value directly as int64_t
    node->key_hash = hash;
    node->next = map->buckets[index];
    
    map->buckets[index] = node;
    map->size++;
}

static void* hashmap_remove(HashMap* map, uint64_t hash, const void* key, void** out_value) {
    size_t index = hash % map->capacity;
    CacheNode* node = map->buckets[index];
    CacheNode* prev = NULL;
    
    while (node) {
        if (node->key_hash == hash && memcmp(node->key, key, sizeof(int64_t)) == 0) {
            if (prev) {
                prev->next = node->next;
            } else {
                map->buckets[index] = node->next;
            }
            
            // Note: value is stored directly as int64_t, not a pointer
            // if (out_value) *out_value = node->value;  // Not used currently
            void* removed_key = node->key;
            free(node);
            map->size--;
            return removed_key;
        }
        prev = node;
        node = node->next;
    }
    
    return NULL;
}

static int hashmap_resize(HashMap* map) {
    size_t new_capacity = map->capacity * 2;
    CacheNode** new_buckets = (CacheNode**)calloc(new_capacity, sizeof(CacheNode*));
    if (!new_buckets) return -1;
    
    // Rehash all entries
    for (size_t i = 0; i < map->capacity; i++) {
        CacheNode* node = map->buckets[i];
        while (node) {
            CacheNode* next = node->next;
            size_t new_index = node->key_hash % new_capacity;
            node->next = new_buckets[new_index];
            new_buckets[new_index] = node;
            node = next;
        }
    }
    
    free(map->buckets);
    map->buckets = new_buckets;
    map->capacity = new_capacity;
    
    return 0;
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
    if (node->prev) node->prev->next = node->next;
    if (node->next) node->next->prev = node->prev;
    
    if (node == cache->tail) {
        cache->tail = node->prev;
    }
    
    // Move to head
    node->prev = NULL;
    node->next = cache->head;
    if (cache->head) {
        cache->head->prev = node;
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
    if (node->prev) {
        node->prev->next = NULL;
        cache->tail = node->prev;
    } else {
        cache->head = NULL;
        cache->tail = NULL;
    }
    
    // Remove from hash map
    void* old_value;
    hashmap_remove(cache->map, node->key_hash, node->key, &old_value);
    
    // Free node and key
    free(node->key);
    free(node);
    
    cache->currsize--;
}

int64_t vp_lru_cache_get(LRUCache* cache, void* key, int* found) {
    if (!cache || !key) {
        if (found) *found = 0;
        return 0;
    }
    
    // Compute hash from key (key is a tuple with first element as hash)
    int64_t* key_data = (int64_t*)key;
    uint64_t hash = (uint64_t)key_data[0];
    
    CacheNode* base_node = hashmap_get(cache->map, hash, key);
    if (!base_node) {
        if (found) *found = 0;
        return 0;
    }
    
    // Cast to LRU node and move to head
    LRUCacheNode* lru_node = (LRUCacheNode*)base_node;
    lru_cache_move_to_head(cache, lru_node);
    
    if (found) *found = 1;
    return lru_node->value;  // Return value directly as int64_t
}

void vp_lru_cache_set(LRUCache* cache, void* key, int64_t value) {
    if (!cache || !key) return;
    
    // Compute hash from key
    int64_t* key_data = (int64_t*)key;
    uint64_t hash = (uint64_t)key_data[0];
    
    // Check if key already exists
    CacheNode* existing = hashmap_get(cache->map, hash, key);
    if (existing) {
        // Update value and move to head
        LRUCacheNode* lru_node = (LRUCacheNode*)existing;
        lru_node->value = value;  // Store value directly
        lru_cache_move_to_head(cache, lru_node);
        return;
    }
    
    // Evict if necessary
    if (cache->maxsize > 0 && cache->currsize >= cache->maxsize) {
        lru_cache_evict(cache);
    }
    
    // Create new LRU node (with proper size for prev/next pointers)
    LRUCacheNode* node = (LRUCacheNode*)malloc(sizeof(LRUCacheNode));
    if (!node) return;
    
    node->key = key;
    node->value = value;  // Store value directly as int64_t
    node->key_hash = hash;
    node->prev = NULL;
    node->next = cache->head;
    
    if (cache->head) {
        cache->head->prev = node;
    }
    cache->head = node;
    
    if (!cache->tail) {
        cache->tail = node;
    }
    
    // Add to hash map using LRU-specific function
    hashmap_set_lru(cache->map, hash, key, node);
    cache->currsize++;
}

void vp_lru_cache_destroy(LRUCache* cache) {
    if (!cache) return;
    
    // Free all LRU nodes
    LRUCacheNode* node = cache->head;
    while (node) {
        LRUCacheNode* next = node->next;
        free(node->key);
        free(node);
        node = next;
    }
    
    // Free hash map structure (not the values, they're the same as nodes)
    free(cache->map->buckets);
    free(cache->map);
    free(cache);
}

void vp_lru_cache_clear(LRUCache* cache) {
    if (!cache) return;
    
    // Free all LRU nodes
    LRUCacheNode* node = cache->head;
    while (node) {
        LRUCacheNode* next = node->next;
        free(node->key);
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

int64_t vp_cache_get(Cache* cache, void* key, int* found) {
    if (!cache || !key) {
        if (found) *found = 0;
        return 0;
    }
    
    // Compute hash from key
    int64_t* key_data = (int64_t*)key;
    uint64_t hash = (uint64_t)key_data[0];
    
    fprintf(stderr, "[CACHE_GET] key=[%ld,%ld] hash=%lu cache=%p\n", 
            key_data[0], key_data[1], hash, (void*)cache);
    fflush(stderr);
    
    CacheNode* node = hashmap_get(cache->map, hash, key);
    if (!node) {
        fprintf(stderr, "[CACHE_GET] MISS\n");
        fflush(stderr);
        if (found) *found = 0;
        return 0;
    }
    
    fprintf(stderr, "[CACHE_GET] HIT value=%ld\n", node->value);
    fflush(stderr);
    if (found) *found = 1;
    return node->value;
}

void vp_cache_set(Cache* cache, void* key, int64_t value) {
    if (!cache || !key) {
        fprintf(stderr, "[CACHE_SET] NULL cache=%p key=%p\n", (void*)cache, (void*)key);
        fflush(stderr);
        return;
    }
    
    // Compute hash from key
    int64_t* key_data = (int64_t*)key;
    uint64_t hash = (uint64_t)key_data[0];
    
    fprintf(stderr, "[CACHE_SET] key=[%ld,%ld] hash=%lu value=%ld cache=%p\n", 
            key_data[0], key_data[1], hash, value, (void*)cache);
    fflush(stderr);
    
    // Set in hash map
    hashmap_set(cache->map, hash, key, value);
    cache->currsize++;
    
    fprintf(stderr, "[CACHE_SET] DONE size=%zu\n", cache->currsize);
    fflush(stderr);
}

void vp_cache_destroy(Cache* cache) {
    if (!cache) return;
    
    hashmap_destroy(cache->map);  // Free keys and nodes
    free(cache);
}

void vp_cache_clear(Cache* cache) {
    if (!cache) return;
    
    // Free all keys
    for (size_t i = 0; i < cache->map->capacity; i++) {
        CacheNode* node = cache->map->buckets[i];
        while (node) {
            CacheNode* next = node->next;
            free(node->key);
            free(node);
            node = next;
        }
    }
    
    memset(cache->map->buckets, 0, cache->map->capacity * sizeof(CacheNode*));
    cache->map->size = 0;
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

void* vp_tuple_create1(int64_t value) {
    // Allocate tuple: [hash, value]
    int64_t* tuple = (int64_t*)malloc(2 * sizeof(int64_t));
    if (!tuple) return NULL;
    
    tuple[0] = (int64_t)vp_hash_int(value);  // Hash
    tuple[1] = value;                         // Value
    
    return tuple;
}

void* vp_tuple_create2(int64_t value1, int64_t value2) {
    // Allocate tuple: [hash, value1, value2]
    int64_t* tuple = (int64_t*)malloc(3 * sizeof(int64_t));
    if (!tuple) return NULL;
    
    int64_t values[2] = {value1, value2};
    tuple[0] = (int64_t)vp_hash_tuple(values, 2);  // Hash
    tuple[1] = value1;                              // Value 1
    tuple[2] = value2;                              // Value 2
    
    return tuple;
}
