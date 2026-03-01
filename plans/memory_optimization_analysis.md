# Memory Optimization Analysis - Viper Language

## Executive Summary

This document analyzes the current memory optimization strategies in the Viper language and identifies potential areas for improvement.

---

## Current Optimizations (Implemented)

### 1. Escape Analysis (`src/semantic/escape_analysis.rs`)

Determines whether variables can be stack-allocated vs heap-allocated.

```rust
pub enum EscapeState {
    None,       // Safe for stack allocation (thread-local, non-atomic ARC)
    Returned,   // Returns to parent (needs ARC but thread-local)
    MayEscape,  // Conservative estimate (atomic ARC)
    Shared,     // Global/concurrent access (atomic ARC)
}
```

**Benefit**: Enables thread-local (non-atomic) reference counting for ~80% of objects.

---

### 2. Automatic Reference Counting - ARC (`runtime/include/viper_arc.h`)

Dual-mode reference counting based on escape analysis:

| Function | Ref Count Type | Use Case |
|----------|----------------|----------|
| `vp_arc_retain_local()` | Non-atomic `int64_t` | Thread-local objects |
| `vp_arc_retain()` | Atomic `_Atomic int64_t` | Shared across threads |
| `vp_arc_release_batch_local()` | Batch non-atomic | Bulk deallocation |

**Header Structure**:
```c
typedef struct {
    union {
        _Atomic int64_t ref_count_atomic;  // For shared objects
        int64_t ref_count;                  // For local objects
    };
    void (*destructor)(void*);
    uint8_t flags;  // SHARED, POOL, LOCAL
    uint8_t reserved[7];
} ViperHeader;
```

**Benefit**: Non-atomic operations avoid expensive memory barriers for the majority of objects.

---

### 3. Tagged Integers (`runtime/include/tagged_int.h`, `src/codegen/runtime/tagged_int.rs`)

Pointer tagging for automatic small integer optimization:

```c
typedef uint64_t TaggedInt;

// LSB = 0: Small integer (i63) stored as (value << 1)
// LSB = 1: BigInt pointer (heap allocated)

#define TAGGED_INT_MAX_SMALL ((1LL << 62) - 1)   // ±4.6 quintillion
```

**Overflow Detection**:
```c
static inline bool would_overflow_add(int64_t a, int64_t b) {
    if (b > 0 && a > TAGGED_INT_MAX_SMALL - b) return true;
    if (b < 0 && a < TAGGED_INT_MIN_SMALL - b) return true;
    return false;
}
```

**Benefit**: Small integers use no heap allocation; BigInt (GMP) only allocated on overflow.

---

### 4. Inline List Operations (`src/codegen/inline_lists.rs`)

Instead of runtime function calls, generates direct LLVM IR:

```rust
pub fn inline_i64_list_get<'ctx>(...) -> Result<BasicValueEnum<'ctx>, String> {
    let data_ptr = get_list_data_ptr(state, list_val)?;
    let elem_ptr = builder.build_in_bounds_gep(i64_type, data_ptr, &[index_val], "...")?;
    let loaded = builder.build_load(i64_type, elem_ptr, "...")?;
    Ok(loaded)
}
```

**ViperList Layout**:
```c
struct ViperList {
    int64_t ref_count;      // offset 0
    int64_t length;         // offset 8
    int64_t capacity;       // offset 16
    ViperListType elem_type;// offset 24
    void* data;             // offset 32
};  // Total: 40 bytes
```

**Benefit**: 2-3x performance improvement for tight loops; enables LLVM vectorization.

---

### 5. Bit Vectors (`runtime/include/viper_stdlib.h`)

Specialized boolean storage using 1 bit per element:

```c
ViperList* vp_bitvec_create(void);
bool vp_bitvec_get(ViperList* vec, int64_t index);
void vp_bitvec_set(ViperList* vec, int64_t index, bool value);
```

**Benefit**: 8x memory savings compared to `bool[]` (1 byte per element).

---

### 6. Dead Code Elimination (`src/codegen/dce.rs`)

Removes unused variable declarations and dead stores using escape analysis:

```rust
pub fn optimize_with_escape_info(
    &mut self,
    module: &Module,
    escape_info: &HashMap<String, HashSet<String>>,
) -> Module
```

**Benefit**: Eliminates non-escaping unused variables and redundant assignments.

---

### 7. Monomorphization (`src/semantic/monomorphization.rs`)

Specializes generic functions for concrete types at compile time:

```rust
pub fn specialize_function(
    &mut self,
    func_name: &str,
    type_args: &[Type],
    original_symbol: &Symbol,
    original_body: &[Stmt],
) -> Result<String, String>
```

**Benefit**: Zero-cost generics; type-specific optimizations possible.

---

### 8. Memory Pools (`runtime/src/memory/pool.h`)

Object pool for fast fixed-size allocation:

```c
typedef struct VpObjectPool {
    size_t obj_size;
    int64_t capacity;
    void* free_list;
    void* allocated;
    int64_t total_allocated;
    int64_t total_freed;
} VpObjectPool;

VpObjectPool* vp_pool_create(size_t obj_size, int64_t capacity);
void* vp_pool_alloc(VpObjectPool* pool);
```

**Benefit**: O(1) allocation/deallocation without system malloc overhead.

---

## Potential Optimization Opportunities

### 1. String Interning (NOT IMPLEMENTED)

**Current Behavior**:
```c
char* s1 = vp_str_create("hello");  // Allocates
char* s2 = vp_str_create("hello");  // Allocates again!
```

**Proposed Solution**:
```c
char* s1 = vp_str_intern("hello");  // Allocates once
char* s2 = vp_str_intern("hello");  // Returns same pointer
```

**Benefits**:
- Deduplicate repeated string literals
- Fast equality comparison (pointer compare instead of `strcmp`)
- Reduced memory footprint for string-heavy workloads

**Implementation**: Hash table for interned strings, reference counted.

---

### 2. Copy-on-Write (CoW) for Lists/Dicts (NOT IMPLEMENTED)

**Current Behavior**:
```c
list2 = vp_list_copy(list1);  // O(n) deep copy
```

**Proposed Solution**:
```c
list2 = vp_list_cow(list1);   // O(1) - shares data, increments refcount
// On write to list2: trigger actual copy
```

**Benefits**:
- Zero-copy slicing
- Fast function argument passing
- Efficient "snapshot" semantics

**Implementation**: Add `is_shared` flag to ViperList; copy on first write if shared.

---

### 3. Small String Optimization (SSO) (NOT IMPLEMENTED)

**Current Behavior**:
All strings heap-allocated via `char*` (8 bytes pointer).

**Proposed Solution**:
```c
#define SSO_MAX 16
typedef struct {
    union {
        char* ptr;              // Heap string for large strings
        char inline[SSO_MAX];   // Small string (inline storage)
    } data;
    uint8_t len;    // Length, high bit = heap flag
} ViperString;
```

**Benefits**:
- No allocation for strings < 16 characters
- Better cache locality for small strings
- Reduced memory fragmentation

**Trade-off**: Slightly larger string struct (24 bytes vs 8 bytes).

---

### 4. Generational/Incremental GC (NOT IMPLEMENTED)

**Current Behavior**:
Only ARC (reference counting). Cyclic references leak memory.

**Proposed Solution**:
```c
// Hybrid ARC + Tracing GC
- Young generation: Copying collector (fast allocation, fast collection)
- Old generation: Mark-sweep (for long-lived objects)
- ARC handles acyclic objects (fast path)
- GC handles cycles (periodic or triggered)
```

**Benefits**:
- Handles circular references automatically
- Better cache locality (generational hypothesis)
- Incremental collection reduces pause times

**Trade-off**: Higher implementation complexity.

---

### 5. SIMD Vector Operations (NOT IMPLEMENTED)

**Current Behavior**:
Scalar loops for list operations.

**Proposed Solution**:
```c
// In codegen: detect vectorizable loops
// Generate: vp_list_add_simd(list_a, list_b)

#include <immintrin.h>
void vp_list_add_simd(int64_t* result, int64_t* a, int64_t* b, int64_t n) {
    for (int i = 0; i < n; i += 4) {
        __m256i va = _mm256_loadu_si256((__m256i*)&a[i]);
        __m256i vb = _mm256_loadu_si256((__m256i*)&b[i]);
        __m256i vr = _mm256_add_epi64(va, vb);
        _mm256_storeu_si256((__m256i*)&result[i], vr);
    }
}
```

**Benefits**:
- 4-8x speedup for numeric array operations
- Automatic vectorization hints for LLVM

**Platforms**: AVX2 (x86), AVX-512 (server), NEON (ARM).

---

### 6. Arena/Bump Allocator (PARTIALLY IMPLEMENTED)

**Current Behavior**:
Object pools exist only for fixed-size objects.

**Proposed Solution**:
```c
VpArena* arena = vp_arena_create(1024 * 1024);  // 1MB arena
void* ptr = vp_arena_alloc(arena, size);        // Bump pointer (O(1))
// ... many allocations ...
vp_arena_destroy(arena);                        // Free all at once
```

**Use Cases**:
- Compiler AST nodes (short-lived, bulk freed)
- Request-scoped allocations in servers
- Temporary buffers in calculations

---

### 7. Hash Table Optimizations (NOT IMPLEMENTED)

**Current Behavior**:
Simple chaining hash table (`runtime/include/viper_types.h`):
```c
typedef struct DictEntry {
    char* key;              // 8 bytes
    ViperValue value;       // 24 bytes
    struct DictEntry* next; // 8 bytes
} DictEntry;  // 40 bytes per entry + malloc overhead
```

**Proposed Solution**:
```c
// Swiss Tables / F14 / Robin Hood Hashing
- Open addressing (better cache locality)
- SIMD-accelerated probing (find slot in 1-2 cache lines)
- Store hash code to avoid rehashing on resize
- Metadata byte per slot (empty/deleted/occupied + 7 bits of hash)
```

**Benefits**:
- 2-3x faster dict operations
- Less memory overhead (no pointer chasing)
- Better cache efficiency

---

### 8. Zero-Copy Serialization (NOT IMPLEMENTED)

**Current Behavior**:
JSON serialization allocates new strings/buffers.

**Proposed Solution**:
```c
// FlatBuffers-style zero-copy
- Write objects directly in serialized format
- Access without parsing (offset-based)
- Memory-mapped file compatible
```

**Benefits**:
- Near-zero cost serialization/deserialization
- Suitable for IPC and persistence

---

## Priority Recommendations

| Priority | Optimization | Effort | Impact | Notes |
|----------|--------------|--------|--------|-------|
| **High** | Small String Optimization | Low | High | Most strings are < 16 chars |
| **High** | String Interning | Low | Medium | Low effort, good ROI |
| **Medium** | Copy-on-Write | Medium | High | Benefits all collections |
| **Medium** | Swiss Table Dict | Medium | High | 2-3x dict performance |
| **Low** | Generational GC | High | Medium | Complex, solves edge case |
| **Low** | SIMD Operations | Medium | High | Numeric workloads only |
| **Low** | Arena Allocator | Low | Medium | Compiler-focused optimization |

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/semantic/escape_analysis.rs` | Escape analysis for stack allocation |
| `src/codegen/dce.rs` | Dead code elimination |
| `src/semantic/monomorphization.rs` | Generic specialization |
| `src/codegen/inline_lists.rs` | Inline list operations |
| `runtime/include/viper_arc.h` | ARC header definitions |
| `runtime/include/tagged_int.h` | Tagged integer implementation |
| `runtime/include/viper_types.h` | Core type definitions |
| `runtime/include/viper_stdlib.h` | Standard library functions |
| `runtime/src/memory/pool.h` | Object pool allocator |

---

## Additional Optimization Opportunities

### 9. Constant Folding & Propagation (PARTIALLY IMPLEMENTED)

**Current**: Basic DCE exists but constant folding is limited.

**Proposed**:
```rust
// Before:
const x = 10
const y = 20
const z = x + y  // Could be computed at compile time

// After optimization:
const z = 30
```

**Benefit**: Eliminate runtime computations for constants.

---

### 10. Loop-Invariant Code Motion (NOT IMPLEMENTED)

**Current**: Loop bodies are generated as-is.

**Proposed**:
```rust
// Before:
for i in range(n):
    x = expensive_calculation()  // Same result every iteration
    result[i] = x + i

// After optimization:
x = expensive_calculation()      // Hoisted outside loop
for i in range(n):
    result[i] = x + i
```

**Benefit**: Move computations outside loops when possible.

---

### 11. Tail Call Optimization (NOT IMPLEMENTED)

**Current**: Recursive calls consume stack space.

**Proposed**:
```rust
// Recursive factorial - current implementation
fn factorial(n):
    if n <= 1: return 1
    return n * factorial(n - 1)  // Stack grows O(n)

// With tail call optimization:
fn factorial_tail(n, acc = 1):
    if n <= 1: return acc
    return factorial_tail(n - 1, n * acc)  // Reuse stack frame
```

**Benefit**: Recursive algorithms use O(1) stack space.

---

### 12. Memory Prefetching (NOT IMPLEMENTED)

**Current**: No prefetch hints for sequential access.

**Proposed**:
```c
// For sequential list/dict access
for (int i = 0; i < n; i++) {
    __builtin_prefetch(&data[i + 64], 0, 3);  // Prefetch ahead
    process(data[i]);
}
```

**Benefit**: Hide memory latency for predictable access patterns.

---

### 13. LLVM Pass Configuration Improvements (OPPORTUNITY EXISTS)

**Current**: Basic LLVM passes in `src/driver/aot.rs`:
```rust
let passes = match opt_level {
    1 => "default<O1>,mem2reg,instcombine,simplifycfg",
    2 => "default<O2>,mem2reg,instcombine,simplifycfg,loop-vectorize",
    3 => "default<O3>,mem2reg,instcombine,simplifycfg,loop-vectorize,aggressive-instcombine",
    _ => "default<O1>",
};
```

**Proposed Additions**:
- `-slp-vectorizer`: Vectorize straight-line code
- `-loop-unroll`: Unroll small loops
- `-licm`: Loop-invariant code motion
- `-inline`: Aggressive inlining
- `-coro-elide`: Coroutine elision for async
- `-pgo-memop-opt`: Profile-guided memory optimization

**Benefit**: Better LLVM optimization coverage.

---

### 14. Redundant Clone Elimination (OPPORTUNITY IN CODEGEN)

**Found in codebase**: Many `.clone()` calls in codegen:
```rust
// From src/codegen/dce.rs:185
self.used_vars.insert(name.clone());

// From src/codegen/dce.rs:275
let stores: Vec<_> = self.var_stores.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

// From src/codegen/oop/classes.rs:104
self.classes.insert(metadata.name.clone(), metadata);
```

**Proposed**: Use references or `Rc<str>` / `Arc<str>` for frequently cloned strings.

**Benefit**: Reduce memory copies during compilation.

---

### 15. Lazy Evaluation for Collections (NOT IMPLEMENTED)

**Current**: List comprehensions create intermediate collections.

**Proposed**:
```rust
// Before (creates intermediate list):
result = [x * 2 for x in items if x > 0]

// After (lazy iterator - no intermediate allocation):
result = items.iter().filter(|x| x > 0).map(|x| x * 2).collect()
```

**Benefit**: Chain operations without intermediate allocations.

---

### 16. Compressed References (NOT IMPLEMENTED)

**Current**: 64-bit pointers on all platforms.

**Proposed**: On 64-bit systems with <32GB heap, use 32-bit compressed pointers:
```c
typedef uint32_t CompressedPtr;  // 4 bytes instead of 8
void* decompress(CompressedPtr p) { return base_address + (p << 3); }
```

**Benefit**: 50% reduction in pointer memory usage.

---

### 17. Write Barriers for Generational GC (NOT IMPLEMENTED)

**Current**: ARC doesn't track old-to-young references.

**Proposed**: If generational GC is added:
```c
void vp_write_barrier(ViperObject* obj, ViperObject* ref) {
    if (is_old(obj) && is_young(ref)) {
        add_to_remembered_set(obj);
    }
}
```

**Benefit**: Efficient generational collection.

---

### 18. Stack Allocation for Short-Lived Collections (OPPORTUNITY)

**Current**: All collections heap-allocated.

**Proposed**: Small, non-escaping collections on stack:
```rust
fn sum_small_list():
    items = [1, 2, 3, 4, 5]  // Stack-allocated (known size, doesn't escape)
    return items.sum()
```

**Benefit**: Zero allocation for small temporary collections.

---

### 19. Branch Prediction Hints (PARTIALLY IMPLEMENTED)

**Current**: Uses `VIPER_LIKELY`/`VIPER_UNLIKELY` in some places.

**Opportunity**: More consistent usage:
```c
// Before:
if (list->length == 0) return NULL;

// After:
if (VIPER_UNLIKELY(list->length == 0)) return NULL;
```

**Benefit**: Better CPU branch prediction.

---

### 20. PGO (Profile-Guided Optimization) Enhancement (PARTIALLY IMPLEMENTED)

**Current**: Basic PGO support exists in `src/driver/aot.rs`.

**Proposed Extensions**:
- Hot/cold function splitting
- Indirect call promotion
- Value profiling for memcmp

**Benefit**: Binary optimized for actual runtime behavior.

---

## Summary of All Optimization Opportunities

| # | Optimization | Status | Effort | Impact |
|---|--------------|--------|--------|--------|
| 1 | String Interning | Not Implemented | Low | Medium |
| 2 | Copy-on-Write | Not Implemented | Medium | High |
| 3 | Small String Optimization | Not Implemented | Low | High |
| 4 | Generational GC | Not Implemented | High | Medium |
| 5 | SIMD Operations | Not Implemented | Medium | High |
| 6 | Arena Allocator | Partially Implemented | Low | Medium |
| 7 | Swiss Table Dict | Not Implemented | Medium | High |
| 8 | Zero-Copy Serialization | Not Implemented | Medium | Medium |
| 9 | Constant Folding | Partially Implemented | Low | Medium |
| 10 | Loop-Invariant Code Motion | Not Implemented | Medium | High |
| 11 | Tail Call Optimization | Not Implemented | Medium | Medium |
| 12 | Memory Prefetching | Not Implemented | Low | Medium |
| 13 | LLVM Pass Improvements | Opportunity Exists | Low | High |
| 14 | Clone Elimination | Opportunity Exists | Medium | Medium |
| 15 | Lazy Evaluation | Not Implemented | High | High |
| 16 | Compressed References | Not Implemented | High | Medium |
| 17 | Write Barriers | Not Implemented | Medium | Medium |
| 18 | Stack Allocation | Not Implemented | Medium | High |
| 19 | Branch Prediction | Partially Implemented | Low | Low |
| 20 | PGO Enhancement | Partially Implemented | Medium | High |

---

## Recommended Implementation Priority

### Phase 1: Quick Wins (Low Effort, High Impact)
1. Small String Optimization
2. String Interning
3. LLVM Pass Improvements
4. Memory Prefetching

### Phase 2: Medium Investment (Medium Effort, High Impact)
1. Copy-on-Write for Collections
2. Swiss Table Dict
3. Loop-Invariant Code Motion
4. Stack Allocation for Small Collections

### Phase 3: Long-term Investments
1. Generational GC
2. Lazy Evaluation
3. SIMD Operations
4. Compressed References

---

## Conclusion

The Viper language already implements sophisticated memory optimizations including escape analysis, dual-mode ARC, tagged integers, and inline list operations. The highest-impact future optimizations would be **Small String Optimization** and **String Interning** due to their low implementation complexity and significant performance benefits for typical workloads.

The codebase shows good optimization foundations but has opportunities for improvement in:
- String handling (interning, SSO)
- Collection operations (CoW, Swiss tables)
- LLVM pass configuration
- Compiler performance (reducing clones)
