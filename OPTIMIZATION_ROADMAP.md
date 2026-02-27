# Viper Optimization Roadmap
## Achieving C-Level Performance

Based on sieve benchmark (10M elements):

| Language | Time | Relative |
|----------|------|----------|
| Go | 0.071s | 1.0x |
| Rust | 0.081s | 1.14x |
| C | 0.092s | 1.30x |
| Viper AOT | 0.350s | 4.93x |
| Viper JIT | 0.380s | 5.35x |

---

## Priority 1: Typed Lists (4-5x expected improvement)

### Problem
All lists use `int64_t*` regardless of element type:
```c
struct ViperList {
    int64_t ref_count;
    int64_t length;
    int64_t capacity;
    int64_t* data;  // 8 bytes even for bool!
};
```

### Solution: Type-Specific List Structures

```c
// runtime/include/viper_types.h
typedef enum {
    VIPER_LIST_I64,
    VIPER_LIST_F64,
    VIPER_LIST_BOOL,
    VIPER_LIST_GENERIC,
} ViperListType;

// Type-specific storage
typedef struct {
    int64_t ref_count;
    int64_t length;
    int64_t capacity;
    ViperListType elem_type;
    union {
        int8_t*  data_bool;    // 1 byte per bool
        int64_t* data_i64;     // 8 bytes per i64
        double*  data_f64;     // 8 bytes per f64
        void**   data_generic; // For objects
    } data;
} ViperList;
```

### Type-Specific Operations

```c
// runtime/src/list_bool.c
void vp_list_bool_append(ViperList* list, bool value);
bool vp_list_bool_get(ViperList* list, int64_t index);
void vp_list_bool_set(ViperList* list, int64_t index, bool value);

// Codegen emits direct calls:
// is_prime[j] = 0  →  vp_list_bool_set(is_prime, j, 0)
```

**Expected Impact:**
- 8x memory reduction for bool lists
- 4x memory reduction for i32 lists
- Better cache locality → 2-3x speedup

---

## Priority 2: Stack Allocation via Escape Analysis (2-3x expected improvement)

### Problem
All local lists are heap-allocated with ARC overhead.

### Solution: Enhanced Escape Analysis

```rust
// semantic/escape_analysis.rs
pub fn analyze_escape(stmts: &[Stmt]) -> EscapeAnalysisResult {
    // Track if list escapes function scope
    // - Returned from function → escapes
    // - Stored in global → escapes
    // - Passed to unknown function → escapes
    // - Only used locally → NO ESCAPE
}
```

### Codegen Changes

```rust
// codegen/mod.rs
if escapes {
    // Heap allocate with ARC
    let list_ptr = builder.build_call(vp_list_create, ...);
} else {
    // Stack allocate - no ARC needed!
    let list_alloca = builder.build_alloca(list_struct_type, "stack_list");
    // Initialize inline
    builder.build_store(...);
}
```

**Expected Impact:**
- Zero malloc/free for local lists
- No ARC retain/release overhead
- Better CPU cache utilization

---

## Priority 3: Bounds Check Elimination (1.5-2x expected improvement)

### Problem
Every list access has bounds checking:
```llvm
; Current IR
%in_bounds = icmp slt i64 %idx, %len
call void @vp_panic_if(!%in_bounds)
%elem = getelementptr i64, i64* %data, i64 %idx
```

### Solution: Range Analysis

```rust
// semantic/range_analysis.rs
pub fn analyze_loop_bounds(stmts: &[Stmt]) -> RangeAnalysisResult {
    // Prove: for i in 0..len(list): list[i] is always in bounds
    // Prove: sieve loop j = i*i; j <= n; j += i is always valid
}
```

### Codegen Changes

```rust
// When bounds proven safe:
if proven_safe {
    // Skip bounds check
    let elem = unsafe_get_element(list, idx);
} else {
    // Keep bounds check
    let elem = safe_get_element(list, idx);
}
```

**Expected Impact:**
- Removes 2-3 instructions per list access
- Enables LLVM vectorization

---

## Priority 4: Bit-Packed Bool Lists (8x memory, 2-4x speedup)

### Ultimate Optimization for Bool Lists

```c
// runtime/include/viper_types.h
typedef struct {
    int64_t ref_count;
    int64_t length;
    int64_t capacity_bytes;
    uint8_t* bits;  // 1 bit per bool!
} ViperListBool;

// runtime/src/list_bool.c
static inline void vp_list_bool_set(ViperListBool* list, int64_t idx, bool val) {
    int64_t byte_idx = idx / 8;
    uint8_t bit_mask = 1 << (idx % 8);
    if (val) {
        list->bits[byte_idx] |= bit_mask;
    } else {
        list->bits[byte_idx] &= ~bit_mask;
    }
}

static inline bool vp_list_bool_get(ViperListBool* list, int64_t idx) {
    int64_t byte_idx = idx / 8;
    uint8_t bit_mask = 1 << (idx % 8);
    return (list->bits[byte_idx] & bit_mask) != 0;
}
```

**For 1B element sieve:**
- Current: ~8 GB (OOM)
- Bit-packed: ~125 MB (feasible!)

---

## Implementation Plan

### Phase 1: Typed Lists (Week 1-2)
1. Modify `ViperList` struct with type tag + union
2. Add `vp_list_bool_*` functions
3. Update codegen to emit type-specific calls
4. Add type inference for list element types

### Phase 2: Escape Analysis (Week 3-4)
1. Implement escape analysis pass
2. Add stack allocation codegen
3. Skip ARC for stack lists
4. Add deallocation at scope exit

### Phase 3: Bounds Check Elimination (Week 5)
1. Implement range analysis
2. Add safe/unsafe get/set variants
3. Update codegen to skip checks when proven safe

### Phase 4: Bit-Packed Bools (Week 6)
1. Add `ViperListBool` struct
2. Implement bit manipulation functions
3. Update codegen for bool list detection
4. Special-case bool list comprehensions

---

## Expected Final Performance

| Optimization | Cumulative Speedup |
|--------------|-------------------|
| Baseline | 1.0x (0.350s) |
| Typed Lists | 2.5x (0.140s) |
| + Stack Allocation | 4.0x (0.088s) |
| + Bounds Elimination | 5.0x (0.070s) |
| + Bit-Packed Bools | 6.0x (0.058s) |

**Target: Match or beat C (0.092s → 0.058s)**

---

## Additional Future Optimizations

- **SIMD Vectorization**: Auto-vectorize list operations
- **Parallel Sieve**: Multi-threaded marking
- **Cache-Optimized Layout**: Structure-of-arrays vs array-of-structures
- **Profile-Guided Optimization**: Already supported, needs tuning
- **Inline Functions**: Aggressive inlining for hot paths
- **Custom Allocator**: Arena allocation for short-lived lists
