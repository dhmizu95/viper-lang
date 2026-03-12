# Memoization Cache: Complete Fix and ARC Integration Plan

**Date:** March 12, 2026  
**Status:** Ready for Implementation  
**Approach:** Fix all limitations using ARC (Automatic Reference Counting) infrastructure

---

## Executive Summary

This plan addresses **all identified limitations** in the Viper memoization cache system by integrating with the existing **ARC (Automatic Reference Counting)** infrastructure:

### Current Limitations Identified

| # | Limitation | Severity | Root Cause |
|---|------------|----------|------------|
| 1 | **BigInt Return Values Broken** | 🔴 CRITICAL | Cache stores pointers as i64, causing truncation |
| 2 | **Limited to 1-2 Parameters** | 🟠 HIGH | Missing tuple creation for 3+ parameters |
| 3 | **Performance Overhead** | 🟡 MEDIUM | malloc() on every call, expensive modulo operations |
| 4 | **Manual Memory Management** | 🟡 MEDIUM | Risk of leaks, use-after-free, double-free |

### Solution: ARC Integration

Leverage Viper's existing ARC system (`runtime/src/memory/arc.c`, `runtime/include/viper_arc.h`) to:

1. **✅ Fix BigInt Support** - ARC pointers handle BigInt return values correctly
2. **✅ Support 3-8 Parameters** - ARC-allocated tuple keys for multi-param functions
3. **✅ Improve Performance** - Pool allocation (2-3x faster than malloc), power-of-2 hash map
4. **✅ Safer Memory** - Automatic cleanup via reference counting, no leaks

---

## Current Architecture Analysis

### Existing ARC Infrastructure ✅

```
runtime/
├── include/
│   └── viper_arc.h          # ARC header, macros, function declarations
├── src/
│   └── memory/
│       ├── arc.c            # Core ARC implementation
│       ├── pool.c           # Object pool allocator
│       └── allocator.h      # Allocator interface
```

### ARC Object Layout

```
Memory Layout:
  [ViperHeader][Object Data...]
   ^           ^
   |           |
 returned     user gets
 from         this pointer
 malloc

ViperHeader (24 bytes):
  - ref_count_atomic (8 bytes) - for thread-safe objects
  - destructor (8 bytes)       - cleanup callback
  - flags (1 byte)             - shared/pooled/local flags
  - reserved (7 bytes)         - alignment padding
```

### Available ARC Functions

```c
// Allocation (with pool optimization)
void* vp_arc_alloc(size_t size);           // Thread-safe, atomic ref count
void* vp_arc_alloc_local(size_t size);     // Thread-local, non-atomic (fast path)

// Reference counting
void vp_arc_retain(void* ptr);             // Atomic increment
void vp_arc_retain_local(void* ptr);       // Non-atomic increment (fast)
void vp_arc_release(void* ptr);            // Atomic decrement + free if zero
void vp_arc_release_local(void* ptr);      // Non-atomic decrement + free
void vp_arc_release_batch(void** ptrs, size_t count);  // Batch release
void vp_arc_release_batch_local(void** ptrs, size_t count);

// Introspection
int64_t vp_arc_ref_count(void* ptr);       // Get current ref count
void vp_arc_set_destructor(void* ptr, void (*destructor)(void*));
bool vp_arc_is_shared(void* ptr);
```

### Current Cache Implementation Issues

```c
// PROBLEM 1: Manual malloc, no ref counting
typedef struct CacheNode {
    void* key;              // malloc'd tuple - ownership unclear
    CacheValue value;       // Union: i64 or BigInt pointer
    uint64_t key_hash;
    int64_t key_size;
    int is_bigint;          // Flag exists but not used correctly
    struct CacheNode* next;
} CacheNode;

// PROBLEM 2: Tuple creation uses raw malloc
void* vp_tuple_create1(int64_t value) {
    int64_t* tuple = (int64_t*)malloc(2 * sizeof(int64_t));
    // No ref counting, no pool allocation
}

// PROBLEM 3: Expensive modulo operation
static CacheNode* hashmap_get(HashMap* map, uint64_t hash, const void* key) {
    size_t index = hash % map->capacity;  // ~20-50 CPU cycles
}

// PROBLEM 4: BigInt pointer stored as i64
typedef union CacheValue {
    int64_t i64_value;    // For regular integers
    void* bigint_ptr;     // For BigInt - but stored in i64 field!
} CacheValue;
```

---

## New Architecture Design

### ARC-Integrated Cache Key

```c
/**
 * ARC-managed cache key
 * Allocated with vp_arc_alloc_local(), managed by reference counting
 * 
 * Memory layout:
 *   [ViperHeader][key_size][hash][data...]
 *    ^                                   
 *    |                                   
 *  returned from vp_arc_alloc_local()
 */
typedef struct ARCCacheKey {
    ViperHeader header;     // ARC header (24 bytes: ref_count, destructor, flags)
    int64_t key_size;       // Size of key data in bytes
    uint64_t hash;          // Pre-computed hash for fast lookup
    int64_t data[];         // Flexible array: [value1, value2, ...]
} ARCCacheKey;
```

### Updated Cache Node Structures

```c
/**
 * Hash map node - stores ARC-managed key and value
 */
typedef struct CacheNode {
    ARCCacheKey* key;       // ARC-managed (retain on insert, release on remove)
    int64_t value;          // Cached return value (i64 or BigInt pointer)
    int is_bigint;          // 1 if value is BigInt pointer, 0 if i64
    struct CacheNode* next; // Collision chain
} CacheNode;

/**
 * LRU Cache Node - extends CacheNode with doubly-linked list
 */
typedef struct LRUCacheNode {
    ARCCacheKey* key;       // ARC-managed key
    int64_t value;          // Cached return value
    int is_bigint;          // 1 if value is BigInt pointer
    struct LRUCacheNode* prev;  // Previous in LRU order (older)
    struct LRUCacheNode* next;  // Next in LRU order (newer)
} LRUCacheNode;
```

### Power-of-2 Hash Map

```c
/**
 * Hash map with power-of-2 capacity for fast modulo
 */
typedef struct HashMap {
    CacheNode** buckets;
    size_t capacity;
    size_t capacity_mask;   // NEW: capacity - 1 for fast bitwise AND
    size_t size;
} HashMap;

// Fast modulo replacement
static inline size_t hashmap_index(HashMap* map, uint64_t hash) {
    return hash & map->capacity_mask;  // ~1 cycle vs ~20-50 for modulo
}
```

---

## Implementation Plan

### Phase 1: ARC Key Infrastructure (2-3 hours) ⭐

#### Step 1.1: Define ARC Cache Key Type

**File:** `runtime/src/memoization.h`

```c
#ifndef VIPER_MEMOIZATION_H
#define VIPER_MEMOIZATION_H

#include <stddef.h>
#include <stdint.h>
#include "viper_arc.h"  // Include ARC header

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// ARC Cache Key
// ============================================================================

/**
 * ARC-managed cache key
 * Objects allocated with vp_arc_alloc_local(), managed by reference counting
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
// Updated Cache Node Structures
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
 * LRU Cache Node - extends CacheNode with LRU linked list
 */
typedef struct LRUCacheNode {
    ARCCacheKey* key;       // ARC-managed key
    int64_t value;          // Cached return value
    int is_bigint;          // 1 if value is BigInt pointer
    struct LRUCacheNode* prev;  // Previous in LRU order
    struct LRUCacheNode* next;  // Next in LRU order
} LRUCacheNode;

/**
 * Hash map - with power-of-2 capacity for fast modulo
 */
typedef struct HashMap {
    CacheNode** buckets;
    size_t capacity;
    size_t capacity_mask;   // capacity - 1 for fast bitwise AND
    size_t size;
} HashMap;

// ... rest of existing definitions (LRUCache, Cache, function declarations)
```

#### Step 1.2: Implement ARC Key Creation Functions

**File:** `runtime/src/memoization.c`

```c
#include "memoization.h"
#include "viper_arc.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// ============================================================================
// ARC Key Creation Functions
// ============================================================================

ARCCacheKey* arc_key_create1(int64_t value) {
    // Allocate: header + key_size + hash + 1 value = 2 int64_t extra
    size_t data_size = 2 * sizeof(int64_t);  // [hash, value]
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
    size_t data_size = 3 * sizeof(int64_t);  // [hash, v1, v2]
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

ARCCacheKey* arc_key_create_n(const int64_t* values, size_t count) {
    if (count < 3 || count > 8) {
        fprintf(stderr, "arc_key_create_n: count must be 3-8, got %zu\n", count);
        return NULL;
    }
    
    size_t data_size = (count + 1) * sizeof(int64_t);  // [hash, v1, v2, ...]
    size_t total_size = sizeof(ARCCacheKey) + data_size;
    
    ARCCacheKey* key = (ARCCacheKey*)vp_arc_alloc_local(total_size);
    if (!key) return NULL;
    
    key->key_size = data_size;
    key->hash = vp_hash_tuple(values, count);
    
    // Copy all values: data[0] = hash placeholder, data[1..count] = values
    memcpy(&key->data[1], values, count * sizeof(int64_t));
    
    return key;
}

// ============================================================================
// Backward Compatibility: Old Tuple Functions (deprecated but kept)
// ============================================================================

void* vp_tuple_create1(int64_t value) {
    // For backward compatibility - wraps arc_key_create1
    return arc_key_create1(value);
}

void* vp_tuple_create2(int64_t value1, int64_t value2) {
    // For backward compatibility - wraps arc_key_create2
    return arc_key_create2(value1, value2);
}
```

---

### Phase 2: Power-of-2 Hash Map Optimization (1-2 hours) ⭐

#### Step 2.1: Update HashMap Creation

**File:** `runtime/src/memoization.c`

```c
#define INITIAL_HASHMAP_CAPACITY 64
#define HASHMAP_LOAD_FACTOR 0.75

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
```

#### Step 2.2: Update HashMap Operations

```c
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

static void hashmap_remove(HashMap* map, uint64_t hash, const ARCCacheKey* key) {
    size_t index = hash & map->capacity_mask;
    CacheNode* node = map->buckets[index];
    CacheNode* prev = NULL;

    while (node) {
        if (node->key->hash == hash && 
            node->key->key_size == key->key_size &&
            memcmp(node->key->data, key->data, key->key_size) == 0) {
            if (prev) {
                prev->next = node->next;
            } else {
                map->buckets[index] = node->next;
            }

            // Release key reference (triggers ARC cleanup)
            arc_key_release(node->key);
            free(node);
            map->size--;
            return;
        }
        prev = node;
        node = node->next;
    }
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
```

---

### Phase 3: Update Cache Operations with ARC (3-4 hours) ⭐

#### Step 3.1: Update LRU Cache Set

```c
void vp_lru_cache_set(LRUCache* cache, ARCCacheKey* key, int64_t value, int is_bigint) {
    if (!cache || !key) return;

    uint64_t hash = key->hash;

    // Check if key exists
    CacheNode* base_node = hashmap_get(cache->map, hash, key);
    if (base_node) {
        LRUCacheNode* lru_node = (LRUCacheNode*)base_node;
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
    node->prev = NULL;
    node->next = cache->head;

    // Update linked list
    if (cache->head) cache->head->prev = node;
    cache->head = node;
    if (!cache->tail) cache->tail = node;

    // Add to hash map (transfers ownership)
    hashmap_set_lru(cache->map, hash, node);
    cache->currsize++;
}

static void hashmap_set_lru(HashMap* map, uint64_t hash, LRUCacheNode* lru_node) {
    if ((double)(map->size + 1) / map->capacity > HASHMAP_LOAD_FACTOR) {
        hashmap_resize(map);
    }

    size_t index = hash & map->capacity_mask;
    lru_node->key = lru_node->key;  // Already set
    ((CacheNode*)lru_node)->next = map->buckets[index];
    map->buckets[index] = (CacheNode*)lru_node;
    map->size++;
}
```

#### Step 3.2: Update LRU Cache Get

```c
int64_t vp_lru_cache_get(LRUCache* cache, ARCCacheKey* key, int* found, int* is_bigint) {
    if (!cache || !key) {
        if (found) *found = 0;
        return 0;
    }

    uint64_t hash = key->hash;

    CacheNode* base_node = hashmap_get(cache->map, hash, key);
    if (!base_node) {
        if (found) *found = 0;
        return 0;
    }

    // Move to head (LRU update)
    LRUCacheNode* lru_node = (LRUCacheNode*)base_node;
    lru_cache_move_to_head(cache, lru_node);

    if (found) *found = 1;
    if (is_bigint) *is_bigint = lru_node->is_bigint;
    return lru_node->value;  // Return value directly (i64 or BigInt pointer)
}
```

#### Step 3.3: Update LRU Cache Evict

```c
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

    // Remove from hash map (just removes pointer, doesn't free key)
    hashmap_remove_no_free(cache->map, node->key->hash, node->key);

    // Release key reference (triggers ARC cleanup)
    arc_key_release(node->key);

    // Free node
    free(node);

    cache->currsize--;
}

static void hashmap_remove_no_free(HashMap* map, uint64_t hash, const ARCCacheKey* key) {
    size_t index = hash & map->capacity_mask;
    CacheNode* node = map->buckets[index];
    CacheNode* prev = NULL;

    while (node) {
        if (node->key->hash == hash && 
            node->key->key_size == key->key_size &&
            memcmp(node->key->data, key->data, key->key_size) == 0) {
            if (prev) {
                prev->next = node->next;
            } else {
                map->buckets[index] = node->next;
            }
            // Don't free key or node - caller will handle
            map->size--;
            return;
        }
        prev = node;
        node = node->next;
    }
}
```

#### Step 3.4: Update Cache Destroy

```c
void vp_lru_cache_destroy(LRUCache* cache) {
    if (!cache) return;

    // Free all LRU nodes (keys released via ARC)
    LRUCacheNode* node = cache->head;
    while (node) {
        LRUCacheNode* next = node->next;
        arc_key_release(node->key);  // Release key (triggers free)
        free(node);
        node = next;
    }

    // Free hash map structure
    free(cache->map->buckets);
    free(cache->map);
    free(cache);
}
```

#### Step 3.5: Update Unbounded Cache Functions

```c
// Similar updates for vp_cache_create, vp_cache_get, vp_cache_set, vp_cache_destroy
// Replace void* key with ARCCacheKey* key
// Use arc_key_retain/release for memory management

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
```

---

### Phase 4: Update Codegen for ARC and Multi-Param Support (3-4 hours) ⭐

#### Step 4.1: Update Runtime Function Declarations

**File:** `src/codegen/runtime/memoization.rs`

```rust
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicType;

/// Memoization runtime functions
pub struct MemoizationFunctions<'ctx> {
    // ARC key creation functions (replaces old tuple functions)
    pub arc_key_create1: FunctionValue<'ctx>,
    pub arc_key_create2: FunctionValue<'ctx>,
    pub arc_key_create3: FunctionValue<'ctx>,
    pub arc_key_create4: FunctionValue<'ctx>,
    pub arc_key_create5: FunctionValue<'ctx>,
    pub arc_key_create6: FunctionValue<'ctx>,
    pub arc_key_create7: FunctionValue<'ctx>,
    pub arc_key_create8: FunctionValue<'ctx>,
    
    // LRU Cache functions
    pub lru_cache_create: FunctionValue<'ctx>,
    pub lru_cache_get: FunctionValue<'ctx>,
    pub lru_cache_set: FunctionValue<'ctx>,
    pub lru_cache_destroy: FunctionValue<'ctx>,
    
    // Unbounded Cache functions
    pub cache_create: FunctionValue<'ctx>,
    pub cache_get: FunctionValue<'ctx>,
    pub cache_set: FunctionValue<'ctx>,
    pub cache_destroy: FunctionValue<'ctx>,
}

/// Declare memoization runtime functions
pub fn declare_memoization_functions<'ctx>(
    context: &'ctx Context,
    module: &mut Module<'ctx>,
) -> Result<MemoizationFunctions<'ctx>, String> {
    let i64_type = context.i64_type();
    let i32_type = context.i32_type();
    let void_type = context.void_type();
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let i32_ptr_type = i32_type.ptr_type(inkwell::AddressSpace::default());
    
    // ARC key creation functions (return ARCCacheKey*)
    let arc_key_create1_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let arc_key_create1 = module.add_function("arc_key_create1", arc_key_create1_type, None);
    
    let arc_key_create2_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let arc_key_create2 = module.add_function("arc_key_create2", arc_key_create2_type, None);
    
    let arc_key_create3_type = i8_ptr_type.fn_type(&[
        i64_type.into(), i64_type.into(), i64_type.into()
    ], false);
    let arc_key_create3 = module.add_function("arc_key_create3", arc_key_create3_type, None);
    
    let arc_key_create4_type = i8_ptr_type.fn_type(&[
        i64_type.into(), i64_type.into(), i64_type.into(), i64_type.into()
    ], false);
    let arc_key_create4 = module.add_function("arc_key_create4", arc_key_create4_type, None);
    
    // ... up to arc_key_create8 (similar pattern)
    
    // LRU Cache functions (updated signatures for ARCCacheKey*)
    let lru_cache_create_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let lru_cache_create = module.add_function("vp_lru_cache_create", lru_cache_create_type, None);
    
    let lru_cache_get_type = i64_type.fn_type(&[
        i8_ptr_type.into(),      // cache
        i8_ptr_type.into(),      // key (ARCCacheKey*)
        i32_ptr_type.into(),     // found
        i32_ptr_type.into(),     // is_bigint
    ], false);
    let lru_cache_get = module.add_function("vp_lru_cache_get", lru_cache_get_type, None);
    
    let lru_cache_set_type = void_type.fn_type(&[
        i8_ptr_type.into(),      // cache
        i8_ptr_type.into(),      // key (ARCCacheKey*)
        i64_type.into(),         // value
        i32_type.into(),         // is_bigint
    ], false);
    let lru_cache_set = module.add_function("vp_lru_cache_set", lru_cache_set_type, None);
    
    let lru_cache_destroy_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let lru_cache_destroy = module.add_function("vp_lru_cache_destroy", lru_cache_destroy_type, None);
    
    // Unbounded Cache functions (similar updates)
    let cache_create_type = i8_ptr_type.fn_type(&[], false);
    let cache_create = module.add_function("vp_cache_create", cache_create_type, None);
    
    let cache_get_type = i64_type.fn_type(&[
        i8_ptr_type.into(),
        i8_ptr_type.into(),
        i32_ptr_type.into(),
        i32_ptr_type.into(),
    ], false);
    let cache_get = module.add_function("vp_cache_get", cache_get_type, None);
    
    let cache_set_type = void_type.fn_type(&[
        i8_ptr_type.into(),
        i8_ptr_type.into(),
        i64_type.into(),
        i32_type.into(),
    ], false);
    let cache_set = module.add_function("vp_cache_set", cache_set_type, None);
    
    let cache_destroy_type = void_type.fn_type(&[i8_ptr_type.into()], false);
    let cache_destroy = module.add_function("vp_cache_destroy", cache_destroy_type, None);
    
    Ok(MemoizationFunctions {
        arc_key_create1,
        arc_key_create2,
        arc_key_create3,
        arc_key_create4,
        arc_key_create5,
        arc_key_create6,
        arc_key_create7,
        arc_key_create8,
        lru_cache_create,
        lru_cache_get,
        lru_cache_set,
        lru_cache_destroy,
        cache_create,
        cache_get,
        cache_set,
        cache_destroy,
    })
}

/// Create a global cache pointer for a memoized function
pub fn create_cache_global<'ctx>(
    context: &'ctx Context,
    module: &mut Module<'ctx>,
    func_name: &str,
    is_lru: bool,
) -> inkwell::values::PointerValue<'ctx> {
    let i8_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
    let global_name = format!("__memo_cache_{}", func_name);
    
    let global = module.add_global(i8_ptr_type, None, &global_name);
    global.set_initializer(&i8_ptr_type.const_null());
    global.set_internal_linkage();
    
    global
}
```

#### Step 4.2: Update Function Wrapper Generation

**File:** `src/codegen/core/functions.rs`

```rust
/// Define a memoized function (with @lru_cache or @cache decorator)
pub(crate) fn define_memoized_function(
    &mut self,
    mangled_name: &str,
    original_name: &str,
    params: &[crate::ast::Param],
    return_type: &Option<Type>,
    body: &[Stmt],
    nonlocal_vars_param: &[String],
    is_lru: bool,
    maxsize: i64,
    returns_bigint: bool,
) -> Result<(), String> {
    use crate::codegen::runtime::memoization;
    use inkwell::types::BasicType;
    use inkwell::values::BasicValue;

    // Declare memoization runtime functions
    let memo_funcs = memoization::declare_memoization_functions(self.context, &mut self.module)
        .map_err(|e| format!("Failed to declare memoization functions: {}", e))?;

    // Create global cache for this function
    let cache_global = memoization::create_cache_global(
        self.context, 
        &mut self.module, 
        original_name, 
        is_lru
    );

    // Store cache global for later use
    self.memoized_functions.insert(original_name.to_string(), cache_global);

    // ... [existing code to create body_func and generate original function body] ...

    // Now generate the wrapper function with cache logic
    let wrapper_entry = self.context.append_basic_block(func_value, "wrapper_entry");
    let init_cache_block = self.context.append_basic_block(func_value, "init_cache");
    let do_lookup_block = self.context.append_basic_block(func_value, "do_lookup");
    let cache_hit_block = self.context.append_basic_block(func_value, "cache_hit");
    let cache_miss_block = self.context.append_basic_block(func_value, "cache_miss");

    self.builder.position_at_end(wrapper_entry);

    // Build cache key from parameters using ARC
    let i64_type = self.context.i64_type();
    let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

    // Create cache key tuple based on number of parameters (supports 1-8)
    let key_value = match params.len() {
        1 => {
            let arg0 = func_value.get_nth_param(0).unwrap();
            let key_call = self.builder.build_call(
                memo_funcs.arc_key_create1,
                &[arg0.into()],
                "cache_key",
            ).expect("Failed to create cache key");
            match key_call.try_as_basic_value() {
                inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
                _ => return Err("Failed to create cache key".to_string()),
            }
        }
        2 => {
            let arg0 = func_value.get_nth_param(0).unwrap();
            let arg1 = func_value.get_nth_param(1).unwrap();
            let key_call = self.builder.build_call(
                memo_funcs.arc_key_create2,
                &[arg0.into(), arg1.into()],
                "cache_key",
            ).expect("Failed to create cache key");
            match key_call.try_as_basic_value() {
                inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
                _ => return Err("Failed to create cache key".to_string()),
            }
        }
        3 => {
            let arg0 = func_value.get_nth_param(0).unwrap();
            let arg1 = func_value.get_nth_param(1).unwrap();
            let arg2 = func_value.get_nth_param(2).unwrap();
            let key_call = self.builder.build_call(
                memo_funcs.arc_key_create3,
                &[arg0.into(), arg1.into(), arg2.into()],
                "cache_key",
            ).expect("Failed to create cache key");
            match key_call.try_as_basic_value() {
                inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
                _ => return Err("Failed to create cache key".to_string()),
            }
        }
        4 => {
            let args: Vec<_> = (0..4).map(|i| func_value.get_nth_param(i as u32).unwrap()).collect();
            let key_call = self.builder.build_call(
                memo_funcs.arc_key_create4,
                &[args[0].into(), args[1].into(), args[2].into(), args[3].into()],
                "cache_key",
            ).expect("Failed to create cache key");
            match key_call.try_as_basic_value() {
                inkwell::values::ValueKind::Basic(bv) => bv.into_pointer_value(),
                _ => return Err("Failed to create cache key".to_string()),
            }
        }
        // ... similar for 5-8 parameters
        n => {
            return Err(format!("Memoization supports up to 8 parameters, got {}", n));
        }
    };

    // ... [existing code for cache initialization and lookup] ...

    Ok(())
}
```

---

### Phase 5: BigInt Auto-Detection (2-3 hours) ⭐

#### Step 5.1: Analyze Function Return Type

**File:** `src/codegen/core/functions.rs`

```rust
impl<'ctx> CodeGen<'ctx> {
    /// Analyze function body to determine if it returns BigInt
    /// Returns true if BigInt return type is detected (explicit or inferred)
    fn analyze_returns_bigint(&self, body: &[Stmt], return_type: &Option<Type>) -> bool {
        // Check explicit type annotation first
        if let Some(Type::BigInt) = return_type {
            return true;
        }
        
        // For inferred types, check if body contains BigInt operations
        for stmt in body {
            if self.stmt_contains_bigint(stmt) {
                return true;
            }
        }
        
        false
    }
    
    /// Check if a statement contains BigInt operations
    fn stmt_contains_bigint(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return { value } => {
                value.map_or(false, |e| self.expr_contains_bigint(e))
            }
            Stmt::If { body, else_body, .. } => {
                body.iter().any(|s| self.stmt_contains_bigint(s)) ||
                else_body.iter().flatten().any(|s| self.stmt_contains_bigint(s))
            }
            Stmt::Assign { value, .. } => {
                self.expr_contains_bigint(value)
            }
            Stmt::Expr(expr) => {
                self.expr_contains_bigint(expr)
            }
            // ... check other statement types as needed
            _ => false,
        }
    }
    
    /// Check if an expression involves BigInt
    fn expr_contains_bigint(&self, expr: &Expr) -> bool {
        match expr {
            Expr::BigInt(_) => true,
            
            Expr::BinOp { left, right, op } => {
                // Operations that may produce BigInt
                matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow) &&
                (self.expr_contains_bigint(left) || self.expr_contains_bigint(right))
            }
            
            Expr::Call { func, .. } => {
                // Check if calling BigInt constructor
                if let Expr::Ident(name, _) = func.as_ref() {
                    name == "BigInt"
                } else {
                    false
                }
            }
            
            Expr::UnaryOp { operand, op } => {
                matches!(op, UnaryOp::Neg | UnaryOp::Pos) &&
                self.expr_contains_bigint(operand)
            }
            
            // Recursively check nested expressions
            Expr::Ternary { condition, then_expr, else_expr } => {
                self.expr_contains_bigint(then_expr) || self.expr_contains_bigint(else_expr)
            }
            
            // ... check other expression types as needed
            _ => false,
        }
    }
}
```

#### Step 5.2: Use Analysis in Memoization

**File:** `src/codegen/core/functions.rs`

```rust
// In function definition handling (around line 67-100)

let should_memoize = is_lru_cache || is_cache || (self.auto_memoize && is_recursive);

if should_memoize {
    // Get maxsize from decorator arguments or use default
    let maxsize = if is_lru_cache {
        decorators
            .iter()
            .find(|d| d.name == "lru_cache")
            .and_then(|d| d.args.first())
            .and_then(|arg| {
                if let Expr::Int(n) = arg {
                    Some(*n)
                } else {
                    None
                }
            })
            .unwrap_or(0)
    } else {
        0  // Unbounded for @cache or auto-memoize
    };
    
    // Determine if function returns BigInt (explicit annotation or inferred from body)
    let returns_bigint = self.analyze_returns_bigint(body, return_type);
    
    // Generate memoized function with correct BigInt handling
    self.define_memoized_function(
        &mangled_name, 
        func_name, 
        params, 
        return_type, 
        body,
        &nonlocal_vars, 
        is_lru_cache, 
        maxsize, 
        returns_bigint  // Pass analysis result for correct is_bigint flag
    )?;
}
```

---

### Phase 6: Update JIT Stubs (1-2 hours) ⭐

#### Step 6.1: Add ARC Key Stub Functions

**File:** `src/jit_stubs/memoization.rs`

```rust
// Memoization JIT stubs - C implementations linked at runtime

use std::os::raw::{c_int, c_void};

// Opaque ARC key type
#[repr(C)]
pub struct ARCCacheKey {
    _private: [u8; 0],
}

extern "C" {
    // ARC key creation functions
    fn arc_key_create1(value: i64) -> *mut ARCCacheKey;
    fn arc_key_create2(v1: i64, v2: i64) -> *mut ARCCacheKey;
    fn arc_key_create3(v1: i64, v2: i64, v3: i64) -> *mut ARCCacheKey;
    fn arc_key_create4(v1: i64, v2: i64, v3: i64, v4: i64) -> *mut ARCCacheKey;
    fn arc_key_create5(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64) -> *mut ARCCacheKey;
    fn arc_key_create6(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64) -> *mut ARCCacheKey;
    fn arc_key_create7(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64) -> *mut ARCCacheKey;
    fn arc_key_create8(v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64) -> *mut ARCCacheKey;
    
    // LRU Cache functions (updated signatures)
    fn vp_lru_cache_create(maxsize: u64) -> *mut c_void;
    fn vp_lru_cache_get(cache: *mut c_void, key: *mut ARCCacheKey, found: *mut c_int, is_bigint: *mut c_int) -> i64;
    fn vp_lru_cache_set(cache: *mut c_void, key: *mut ARCCacheKey, value: i64, is_bigint: c_int);
    fn vp_lru_cache_destroy(cache: *mut c_void);
    
    // Unbounded Cache functions (updated signatures)
    fn vp_cache_create() -> *mut c_void;
    fn vp_cache_get(cache: *mut c_void, key: *mut ARCCacheKey, found: *mut c_int, is_bigint: *mut c_int) -> i64;
    fn vp_cache_set(cache: *mut c_void, key: *mut ARCCacheKey, value: i64, is_bigint: c_int);
    fn vp_cache_destroy(cache: *mut c_void);
}

// ARC key creation stubs
#[no_mangle]
pub extern "C" fn arc_key_create1_stub(value: i64) -> *mut c_void {
    unsafe { arc_key_create1(value) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create2_stub(v1: i64, v2: i64) -> *mut c_void {
    unsafe { arc_key_create2(v1, v2) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create3_stub(v1: i64, v2: i64, v3: i64) -> *mut c_void {
    unsafe { arc_key_create3(v1, v2, v3) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn arc_key_create4_stub(v1: i64, v2: i64, v3: i64, v4: i64) -> *mut c_void {
    unsafe { arc_key_create4(v1, v2, v3, v4) as *mut c_void }
}

// ... similar for arc_key_create5_stub through arc_key_create8_stub

// Cache function stubs (updated signatures)
#[no_mangle]
pub extern "C" fn vp_lru_cache_create_stub(maxsize: u64) -> *mut c_void {
    unsafe { vp_lru_cache_create(maxsize) as *mut c_void }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_get_stub(cache: *mut c_void, key: *mut c_void, found: *mut c_int, is_bigint: *mut c_int) -> i64 {
    unsafe { vp_lru_cache_get(cache as *mut c_void, key as *mut ARCCacheKey, found, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_set_stub(cache: *mut c_void, key: *mut c_void, value: i64, is_bigint: c_int) {
    unsafe { vp_lru_cache_set(cache as *mut c_void, key as *mut ARCCacheKey, value, is_bigint) }
}

#[no_mangle]
pub extern "C" fn vp_lru_cache_destroy_stub(cache: *mut c_void) {
    unsafe { vp_lru_cache_destroy(cache as *mut c_void) }
}

// ... similar for unbounded cache stubs
```

#### Step 6.2: Register Stubs in Registry

**File:** `src/jit_stubs/registry/memoization.rs`

```rust
//! Memoization JIT stub registration

use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;

macro_rules! register_fn {
    ($ee:expr, $module:expr, $name:expr => $func:expr) => {
        unsafe {
            let func_ptr = $func as *const () as *mut ();
            $ee.add_global_mapping($module.get_function($name).unwrap(), func_ptr);
        }
    };
}

pub fn register_memoization_stubs(ee: &ExecutionEngine, module: &Module) {
    // Register ARC key creation stubs
    register_fn!(ee, module, "arc_key_create1" => super::super::memoization::arc_key_create1_stub);
    register_fn!(ee, module, "arc_key_create2" => super::super::memoization::arc_key_create2_stub);
    register_fn!(ee, module, "arc_key_create3" => super::super::memoization::arc_key_create3_stub);
    register_fn!(ee, module, "arc_key_create4" => super::super::memoization::arc_key_create4_stub);
    register_fn!(ee, module, "arc_key_create5" => super::super::memoization::arc_key_create5_stub);
    register_fn!(ee, module, "arc_key_create6" => super::super::memoization::arc_key_create6_stub);
    register_fn!(ee, module, "arc_key_create7" => super::super::memoization::arc_key_create7_stub);
    register_fn!(ee, module, "arc_key_create8" => super::super::memoization::arc_key_create8_stub);
    
    // Register LRU cache stubs
    register_fn!(ee, module, "vp_lru_cache_create" => super::super::memoization::vp_lru_cache_create_stub);
    register_fn!(ee, module, "vp_lru_cache_get" => super::super::memoization::vp_lru_cache_get_stub);
    register_fn!(ee, module, "vp_lru_cache_set" => super::super::memoization::vp_lru_cache_set_stub);
    register_fn!(ee, module, "vp_lru_cache_destroy" => super::super::memoization::vp_lru_cache_destroy_stub);
    
    // Register unbounded cache stubs
    register_fn!(ee, module, "vp_cache_create" => super::super::memoization::vp_cache_create_stub);
    register_fn!(ee, module, "vp_cache_get" => super::super::memoization::vp_cache_get_stub);
    register_fn!(ee, module, "vp_cache_set" => super::super::memoization::vp_cache_set_stub);
    register_fn!(ee, module, "vp_cache_destroy" => super::super::memoization::vp_cache_destroy_stub);
}
```

---

## Files to Modify Summary

| File | Changes | Effort |
|------|---------|--------|
| `runtime/src/memoization.h` | Add ARCCacheKey type, update struct definitions, add capacity_mask | 1h |
| `runtime/src/memoization.c` | Implement ARC key functions, update all cache operations, power-of-2 hash | 5h |
| `src/codegen/runtime/memoization.rs` | Update function declarations for ARC, add arc_key_create1-8 | 1.5h |
| `src/codegen/core/functions.rs` | Add BigInt detection, update wrapper generation for 1-8 params | 4h |
| `src/jit_stubs/memoization.rs` | Add ARC key stubs, update cache function signatures | 1.5h |
| `src/jit_stubs/registry/memoization.rs` | Register new stubs | 0.5h |

**Total Effort:** ~13-14 hours

---

## Testing Strategy

### Test 1: ARC Key Operations

```c
// runtime/tests/test_arc_key.c

#include <assert.h>
#include "memoization.h"
#include "viper_arc.h"

void test_arc_key_create1() {
    ARCCacheKey* key = arc_key_create1(42);
    assert(key != NULL);
    assert(key->hash == vp_hash_int(42));
    assert(key->data[0] == 42);
    assert(key->key_size == 2 * sizeof(int64_t));
    arc_key_release(key);  // Should free
}

void test_arc_key_retain_release() {
    ARCCacheKey* key = arc_key_create1(100);
    int64_t initial_ref = vp_arc_ref_count(key);
    
    arc_key_retain(key);  // ref_count++
    assert(vp_arc_ref_count(key) == initial_ref + 1);
    
    arc_key_release(key); // ref_count--
    assert(vp_arc_ref_count(key) == initial_ref);
    
    arc_key_release(key); // ref_count = 0, freed
}

void test_arc_key_create_n() {
    int64_t values[] = {1, 2, 3, 4, 5};
    ARCCacheKey* key = arc_key_create_n(values, 5);
    assert(key != NULL);
    assert(key->key_size == 6 * sizeof(int64_t));
    assert(key->data[1] == 1);
    assert(key->data[5] == 5);
    arc_key_release(key);
}
```

### Test 2: Cache with ARC Keys

```c
// runtime/tests/test_cache_arc.c

#include <assert.h>
#include "memoization.h"

void test_lru_cache_arc() {
    LRUCache* cache = vp_lru_cache_create(128);
    assert(cache != NULL);
    
    // Insert
    ARCCacheKey* key1 = arc_key_create1(10);
    vp_lru_cache_set(cache, key1, 100, 0);
    
    // Lookup
    int found, is_bigint;
    ARCCacheKey* lookup_key = arc_key_create1(10);
    int64_t value = vp_lru_cache_get(cache, lookup_key, &found, &is_bigint);
    assert(found == 1);
    assert(value == 100);
    assert(is_bigint == 0);
    arc_key_release(lookup_key);
    
    // Cache miss
    ARCCacheKey* miss_key = arc_key_create1(999);
    value = vp_lru_cache_get(cache, miss_key, &found, &is_bigint);
    assert(found == 0);
    arc_key_release(miss_key);
    
    vp_lru_cache_destroy(cache);
}

void test_lru_cache_eviction() {
    LRUCache* cache = vp_lru_cache_create(3);  // Small cache
    
    // Insert 4 items (should evict oldest)
    for (int i = 0; i < 4; i++) {
        ARCCacheKey* key = arc_key_create1(i);
        vp_lru_cache_set(cache, key, i * 100, 0);
    }
    
    // First item should be evicted
    int found;
    ARCCacheKey* key0 = arc_key_create1(0);
    vp_lru_cache_get(cache, key0, &found, NULL);
    assert(found == 0);  // Evicted
    arc_key_release(key0);
    
    // Other items should exist
    for (int i = 1; i < 4; i++) {
        ARCCacheKey* key = arc_key_create1(i);
        int64_t value = vp_lru_cache_get(cache, key, &found, NULL);
        assert(found == 1);
        assert(value == i * 100);
        arc_key_release(key);
    }
    
    vp_lru_cache_destroy(cache);
}
```

### Test 3: BigInt Return Values

```python
# tests/test_memoization_bigint.py

def test_fib_bigint():
    """Test that BigInt return values are cached correctly"""
    code = """
@lru_cache(maxsize=None)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    # First call - cache miss
    result1 = fib(75)
    
    # Second call - cache hit (should be instant)
    result2 = fib(75)
    
    print("fib(75) =", result1)
    assert result1 == result2, "Cache hit returned different value"
    assert result1 == 2111485077978050, f"Wrong result: {result1}"
"""
    result = run_viper_code(code)
    assert result.returncode == 0
```

### Test 4: Multi-Parameter Functions

```python
# tests/test_memoization_params.py

def test_knapsack_3_params():
    """Test memoization with 3 parameters"""
    code = """
@lru_cache(maxsize=256)
def knapsack(i, weight, value):
    if i < 0 or weight <= 0:
        return 0
    # Include current item
    include = value + knapsack(i - 1, weight - 1, value) if weight >= 1 else 0
    # Exclude current item
    exclude = knapsack(i - 1, weight, value)
    return max(include, exclude)

def main():
    result = knapsack(5, 10, 20)
    print("knapsack(5, 10, 20) =", result)
"""
    result = run_viper_code(code)
    assert result.returncode == 0

def test_edit_distance_4_params():
    """Test memoization with 4 parameters"""
    code = """
@lru_cache(maxsize=None)
def edit_distance(i, j, s1_len, s2_len):
    if i < 0: return j + 1
    if j < 0: return i + 1
    # Simplified version for testing
    return min(
        edit_distance(i-1, j, s1_len, s2_len) + 1,
        edit_distance(i, j-1, s1_len, s2_len) + 1,
        edit_distance(i-1, j-1, s1_len, s2_len)
    )

def main():
    result = edit_distance(3, 3, 4, 4)
    print("edit_distance(3, 3, 4, 4) =", result)
"""
    result = run_viper_code(code)
    assert result.returncode == 0
```

### Test 5: Performance Benchmark

```python
# tests/test_memoization_performance.py

import time

def test_cache_hit_performance():
    """Test that cache hits are fast (<100ns)"""
    code = """
@lru_cache(maxsize=None)
def double(n):
    return n * 2

def main():
    import time
    
    # Warm up
    for i in range(100):
        double(i)
    
    # Measure cache hit performance
    start = time.time()
    iterations = 100000
    for i in range(iterations):
        double(i % 100)  # All hits
    end = time.time()
    
    avg_ns = ((end - start) * 1_000_000_000) / iterations
    print(f"Average cache hit time: {avg_ns:.1f} ns")
    
    # Should be < 100ns per hit
    assert avg_ns < 100, f"Cache hit too slow: {avg_ns} ns"
"""
    result = run_viper_code(code)
    assert result.returncode == 0

def test_fib_performance():
    """Test that fib(90) with caching completes quickly"""
    code = """
@lru_cache(maxsize=None)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    import time
    
    start = time.time()
    result = fib(90)
    end = time.time()
    
    elapsed_ms = (end - start) * 1000
    print(f"fib(90) = {result}, took {elapsed_ms:.2f} ms")
    
    # Should complete in < 10ms with caching
    assert elapsed_ms < 10, f"fib(90) too slow: {elapsed_ms} ms"
    assert result == 2880067194370816120
"""
    result = run_viper_code(code)
    assert result.returncode == 0
```

### Test 6: Memory Safety

```bash
# Run with valgrind to check for memory leaks
valgrind --leak-check=full --show-leak-kinds=all \
         --track-origins=yes --verbose \
         viper run test_memoization.vp

# Run with AddressSanitizer
export ASAN_OPTIONS=detect_leaks=1
viper run test_memoization.vp
```

---

## Expected Performance Improvements

| Metric | Current | With ARC + Optimizations | Improvement |
|--------|---------|-------------------------|-------------|
| **Cache hit latency** | ~200ns | ~50ns | **4x faster** |
| **Cache miss latency** | ~300ns | ~100ns | **3x faster** |
| **Key allocation** | malloc() (~100ns) | Pool alloc (~30ns) | **3x faster** |
| **Hash map lookup** | Modulo (~20-50 cycles) | Bitwise AND (~1 cycle) | **20-50x faster** |
| **Memory overhead** | 8 bytes/key | 24 bytes/key (ARC header) | +16 bytes |
| **BigInt support** | ❌ Broken | ✅ Working | **Fixed** |
| **Max parameters** | 2 | 8 | **4x more** |
| **Memory safety** | Manual (error-prone) | ARC (automatic) | **No leaks** |

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| ARC overhead slows cache | Medium | Low | Use `vp_arc_alloc_local()` for non-atomic fast path |
| Memory leak if ref count wrong | High | Medium | Add ref count assertions, test with valgrind |
| Breaking existing code | Medium | Low | Keep old tuple functions as wrappers, add deprecation warnings |
| Thread safety issues | Low | Low | Document single-threaded assumption, use atomic ARC if needed |
| Pool exhaustion | Low | Low | Pool falls back to malloc, test with large workloads |

---

## Success Criteria

- [ ] All existing tests pass
- [ ] `fib(75)` returns correct value (2111485077978050) and is fast (<10ms)
- [ ] Functions with 3-8 parameters can use `@lru_cache`
- [ ] No memory leaks (valgrind clean)
- [ ] No use-after-free (AddressSanitizer clean)
- [ ] Cache hit latency <100ns (target: ~50ns)
- [ ] BigInt caching works correctly (no pointer truncation)
- [ ] Power-of-2 hash map implemented (fast modulo)

---

## Implementation Order and Timeline

| Phase | Description | Effort | Priority |
|-------|-------------|--------|----------|
| **Phase 1** | ARC Key Infrastructure | 2-3h | P0 |
| **Phase 2** | Power-of-2 Hash Map | 1-2h | P0 |
| **Phase 3** | Update Cache Operations | 3-4h | P0 |
| **Phase 4** | Update Codegen | 3-4h | P1 |
| **Phase 5** | BigInt Auto-Detection | 2-3h | P1 |
| **Phase 6** | Update JIT Stubs | 1-2h | P1 |

**Total:** 12-18 hours

**Recommended Schedule:**
- **Day 1:** Phases 1-3 (ARC infrastructure, hash optimization, cache ops) - 6-9 hours
- **Day 2:** Phases 4-6 (Codegen, BigInt detection, JIT stubs) - 6-9 hours
- **Day 3:** Testing and validation

---

## Comparison: Before vs After

### Before (Current State)

```python
# ❌ Broken: BigInt return values
@lru_cache(maxsize=None)
def fib(n):
    if n <= 1: return n
    return fib(n-1) + fib(n-2)

fib(75)  # Returns wrong value (pointer truncation)

# ❌ Limited: Only 1-2 parameters
@lru_cache(maxsize=256)
def knapsack(i, weight, value):  # Error: 3 parameters not supported
    pass

# ⚠️ Slow: ~200ns per cache hit
# ⚠️ Risk: Manual memory management (leaks, use-after-free)
```

### After (With This Plan)

```python
# ✅ Fixed: BigInt return values work correctly
@lru_cache(maxsize=None)
def fib(n):
    if n <= 1: return n
    return fib(n-1) + fib(n-2)

fib(75)  # Returns 2111485077978050 instantly (cached)

# ✅ Extended: Up to 8 parameters
@lru_cache(maxsize=256)
def knapsack(i, weight, value):
    # Works!
    pass

@lru_cache(maxsize=None)
def edit_distance(i, j, k, l):
    # 4 parameters - works!
    pass

# ✅ Fast: ~50ns per cache hit (4x faster)
# ✅ Safe: ARC automatic memory management (no leaks)
```

---

## Conclusion

This plan provides a **comprehensive solution** to all identified memoization limitations by:

1. **Leveraging existing ARC infrastructure** - No need to reinvent memory management
2. **Fixing critical BigInt bug** - ARC pointers handle BigInt correctly
3. **Extending to 8 parameters** - ARC key creation for 1-8 params
4. **Improving performance** - Pool allocation + power-of-2 hash map
5. **Ensuring safety** - Automatic cleanup via reference counting

**Total effort:** 12-18 hours  
**Expected ROI:** 4x performance improvement, fixed BigInt support, 4x more parameters, zero memory leaks

---

*Last Updated: March 12, 2026*  
*Author: Viper Language Team*
