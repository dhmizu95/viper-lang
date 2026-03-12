# ARC (Automatic Reference Counting) Implementation Plan for Memoization Cache

## Overview

This plan outlines the migration from manual memory management to Automatic Reference Counting (ARC) for the Viper memoization cache system. ARC will provide safer memory management, prevent use-after-free bugs, and enable shared ownership of cached values.

---

## Current State Analysis

### Current Memory Model
- **Manual allocation**: `malloc`/`free` throughout
- **Ownership**: Cache takes ownership of keys on insert
- **Value storage**: `int64_t` stored directly (not pointers)
- **Deallocation**: Explicit in `destroy()` and `evict()` functions

### Problems with Current Approach
1. **Use-after-free risk**: Keys freed during eviction may still be referenced elsewhere
2. **No shared ownership**: Cannot safely share cached values across threads
3. **Memory leaks**: Easy to forget `free()` calls in error paths
4. **Double-free risk**: Same key could potentially be freed twice

---

## ARC Design

### Reference Counted Header

```c
typedef struct ARCHeader {
    volatile int64_t ref_count;
    volatile int64_t size;  // Optional: track allocation size
} ARCHeader;
```

### Reference Counted Key Structure

```c
typedef struct ARCKey {
    ARCHeader header;
    int64_t data[];  // Flexible array: [hash, value1, value2, ...]
} ARCKey;
```

### Updated Cache Node Structures

```c
typedef struct CacheNode {
    ARCKey* key;              // Reference-counted key
    int64_t value;            // Stored directly (still efficient)
    uint64_t key_hash;        // Pre-computed hash
    struct CacheNode* next;   // Collision chain
} CacheNode;

typedef struct LRUCacheNode {
    ARCKey* key;              // Reference-counted key
    int64_t value;            // Stored directly
    uint64_t key_hash;        // Pre-computed hash
    struct LRUCacheNode* prev;
    struct LRUCacheNode* next;
} LRUCacheNode;
```

---

## Core ARC Functions

### Reference Counting Primitives

```c
// Atomic increment - returns new count
static inline int64_t arc_retain(void* ptr) {
    if (!ptr) return 0;
    ARCHeader* header = (ARCHeader*)ptr - 1;
    return __atomic_add_fetch(&header->ref_count, 1, __ATOMIC_SEQ_CST);
}

// Atomic decrement - returns true if should free
static inline bool arc_release(void* ptr) {
    if (!ptr) return false;
    ARCHeader* header = (ARCHeader*)ptr - 1;
    int64_t new_count = __atomic_sub_fetch(&header->ref_count, 1, __ATOMIC_SEQ_CST);
    return new_count == 0;
}

// Safe release and free
static inline void arc_release_and_free(void* ptr) {
    if (arc_release(ptr)) {
        free((ARCHeader*)ptr - 1);
    }
}
```

### Key Creation Functions

```c
ARCKey* arc_key_create1(int64_t value) {
    size_t size = sizeof(ARCHeader) + 2 * sizeof(int64_t);
    ARCKey* key = (ARCKey*)malloc(size);
    if (!key) return NULL;
    
    key->header.ref_count = 1;
    key->header.size = size;
    key->data[0] = (int64_t)vp_hash_int(value);
    key->data[1] = value;
    return key;
}

ARCKey* arc_key_create2(int64_t value1, int64_t value2) {
    size_t size = sizeof(ARCHeader) + 3 * sizeof(int64_t);
    ARCKey* key = (ARCKey*)malloc(size);
    if (!key) return NULL;
    
    key->header.ref_count = 1;
    key->header.size = size;
    int64_t values[2] = {value1, value2};
    key->data[0] = (int64_t)vp_hash_tuple(values, 2);
    key->data[1] = value1;
    key->data[2] = value2;
    return key;
}
```

---

## Updated Cache Operations

### LRU Cache Get

```c
int64_t vp_lru_cache_get(LRUCache* cache, ARCKey* key, int* found) {
    if (!cache || !key) {
        if (found) *found = 0;
        return 0;
    }

    uint64_t hash = (uint64_t)key->data[0];

    CacheNode* base_node = hashmap_get(cache->map, hash, key);
    if (!base_node) {
        if (found) *found = 0;
        return 0;
    }

    // Move to head (LRU update)
    LRUCacheNode* lru_node = (LRUCacheNode*)base_node;
    lru_cache_move_to_head(cache, lru_node);

    if (found) *found = 1;
    return lru_node->value;
}
```

### LRU Cache Set

```c
void vp_lru_cache_set(LRUCache* cache, ARCKey* key, int64_t value) {
    if (!cache || !key) return;

    uint64_t hash = (uint64_t)key->data[0];

    // Check if key exists
    CacheNode* existing = hashmap_get(cache->map, hash, key);
    if (existing) {
        LRUCacheNode* lru_node = (LRUCacheNode*)existing;
        lru_node->value = value;
        lru_cache_move_to_head(cache, lru_node);
        arc_release_and_free(key);  // Release caller's reference
        return;
    }

    // Evict if necessary
    if (cache->maxsize > 0 && cache->currsize >= cache->maxsize) {
        lru_cache_evict(cache);
    }

    // Create new LRU node
    LRUCacheNode* node = (LRUCacheNode*)malloc(sizeof(LRUCacheNode));
    if (!node) {
        arc_release_and_free(key);
        return;
    }

    node->key = key;        // Takes ownership
    node->value = value;
    node->key_hash = hash;
    node->prev = NULL;
    node->next = cache->head;

    // Update linked list
    if (cache->head) cache->head->prev = node;
    cache->head = node;
    if (!cache->tail) cache->tail = node;

    // Add to hash map
    hashmap_set_lru(cache->map, hash, key, node);
    cache->currsize++;
}
```

### LRU Cache Evict

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

    // Remove from hash map (just removes pointer, doesn't free)
    hashmap_remove_no_free(cache->map, node->key_hash, node->key);

    // Release key reference (may free if last reference)
    arc_release_and_free(node->key);

    // Free node
    free(node);

    cache->currsize--;
}
```

### Cache Destroy

```c
void vp_lru_cache_destroy(LRUCache* cache) {
    if (!cache) return;

    // Free all LRU nodes (keys freed via arc_release)
    LRUCacheNode* node = cache->head;
    while (node) {
        LRUCacheNode* next = node->next;
        arc_release_and_free(node->key);
        free(node);
        node = next;
    }

    // Free hash map structure
    free(cache->map->buckets);
    free(cache->map);
    free(cache);
}
```

---

## Codegen Changes

### Update Tuple Creation

```rust
// src/codegen/runtime/memoization.rs
pub fn declare_memoization_functions<'ctx>(...) -> MemoizationFunctions {
    // ...
    
    // Updated to return ARCKey* (i8*) instead of void*
    let tuple_create1_type = i8_ptr_type.fn_type(&[i64_type.into()], false);
    let tuple_create1 = module.add_function("vp_tuple_create1", tuple_create1_type, None);
    
    let tuple_create2_type = i8_ptr_type.fn_type(&[i64_type.into(), i64_type.into()], false);
    let tuple_create2 = module.add_function("vp_tuple_create2", tuple_create2_type, None);
    
    // ...
}
```

### Update Cache Function Signatures

```rust
// src/codegen/runtime/memoization.rs
let lru_cache_get_type = i64_type.fn_type(
    &[
        i8_ptr_type.into(),      // cache
        i8_ptr_type.into(),      // key (ARCKey*)
        context.i32_type().ptr_type(inkwell::AddressSpace::default()).into(),  // found
    ],
    false,
);

let lru_cache_set_type = void_type.fn_type(
    &[
        i8_ptr_type.into(),      // cache
        i8_ptr_type.into(),      // key (ARCKey*)
        i64_type.into(),         // value
    ],
    false,
);
```

---

## Implementation Phases

### Phase 1: Infrastructure (Low Risk)
- [ ] Add `ARCHeader` struct to `memoization.h`
- [ ] Implement `arc_retain()`, `arc_release()`, `arc_release_and_free()`
- [ ] Create `ARCKey` wrapper type
- [ ] Add `arc_key_create1()`, `arc_key_create2()` functions

### Phase 2: Key Management (Medium Risk)
- [ ] Update `vp_tuple_create1()` to return `ARCKey*`
- [ ] Update `vp_tuple_create2()` to return `ARCKey*`
- [ ] Update `CacheNode` and `LRUCacheNode` to use `ARCKey*`
- [ ] Update hash map functions to work with `ARCKey*`

### Phase 3: Cache Operations (High Risk)
- [ ] Update `vp_lru_cache_set()` with ARC semantics
- [ ] Update `vp_lru_cache_get()` with ARC semantics
- [ ] Update `lru_cache_evict()` to use `arc_release_and_free()`
- [ ] Update `vp_lru_cache_destroy()` with proper ARC cleanup
- [ ] Update unbounded cache functions similarly

### Phase 4: Codegen Updates (Medium Risk)
- [ ] Update memoization.rs function declarations
- [ ] Update functions.rs wrapper generation for ARC types
- [ ] Update JIT stubs for ARC compatibility

### Phase 5: Testing & Validation (Critical)
- [ ] Unit tests for ARC reference counting
- [ ] Stress tests for concurrent access
- [ ] Memory leak detection with valgrind
- [ ] Performance benchmarks (compare before/after)

---

## Thread Safety Considerations

### Atomic Operations
All reference count operations use `__atomic_*` builtins with `__ATOMIC_SEQ_CST` (sequential consistency) for maximum safety.

### Potential Race Conditions
1. **Simultaneous cache access**: May need mutex for cache structure itself
2. **Key hash computation**: Already computed before insertion (safe)
3. **LRU list updates**: May need mutex for linked list operations

### Recommended Approach
- Keep ARC operations lock-free (atomic ref counting)
- Add optional mutex for cache structure if thread-safe cache is needed
- Document that cache is not thread-safe by default

---

## Performance Impact Analysis

### Overhead
- **Memory**: +16 bytes per key (ARCHeader: 8 bytes ref_count + 8 bytes size)
- **CPU**: 2 atomic operations per cache insert (retain + release)
- **CPU**: 1 atomic operation per cache hit (for key comparison, no retain needed)

### Benefits
- **Safety**: Eliminates use-after-free bugs
- **Correctness**: Prevents double-free
- **Debugging**: Can add ref_count assertions for leak detection

### Optimization Opportunities
1. **Weak references**: For read-heavy workloads, avoid retain on get
2. **Batch operations**: Retain/release in batches for bulk inserts
3. **Thread-local caches**: Reduce contention for multi-threaded scenarios

---

## Migration Strategy

### Backward Compatibility
- Keep old function signatures where possible
- Add `ARC_` prefix to new functions
- Gradual migration: test each phase before proceeding

### Testing Checklist
- [ ] All existing tests pass
- [ ] No memory leaks (valgrind clean)
- [ ] No use-after-free (AddressSanitizer clean)
- [ ] Performance within 10% of current implementation

---

## Files to Modify

| File | Changes |
|------|---------|
| `runtime/src/memoization.h` | Add ARC types, update struct definitions |
| `runtime/src/memoization.c` | Implement ARC functions, update all cache operations |
| `runtime/src/tuple.c` | Update tuple creation to return ARCKey* |
| `src/codegen/runtime/memoization.rs` | Update function declarations |
| `src/codegen/core/functions.rs` | Update wrapper generation for ARC types |
| `src/jit_stubs/memoization.rs` | Update JIT stub signatures |

---

## Success Criteria

1. **Correctness**: All existing tests pass
2. **Safety**: No memory errors under valgrind/AddressSanitizer
3. **Performance**: <10% overhead for typical workloads
4. **Maintainability**: Clear ownership semantics documented

---

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Performance regression | Benchmark after each phase, optimize hot paths |
| Thread safety issues | Document single-threaded assumption, add mutex option |
| Migration complexity | Phase-by-phase approach with testing at each stage |
| Increased memory usage | Consider optional slim mode without ARC for performance-critical paths |

---

## Timeline Estimate

- **Phase 1**: 1-2 days
- **Phase 2**: 2-3 days
- **Phase 3**: 3-4 days
- **Phase 4**: 1-2 days
- **Phase 5**: 2-3 days

**Total**: 9-14 days for full implementation and validation
