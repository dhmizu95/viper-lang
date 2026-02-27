# Viper Optimization Status Report

## Executive Summary

This document summarizes the optimization work done on the Viper compiler to achieve C-like performance, focusing on the prime sieve benchmark.

## Completed Work

### 1. JIT Bool List Crash Fix ✅

**Problem**: JIT execution crashed with segmentation fault when using bool lists.

**Root Cause**: JIT stubs used incompatible struct layout (`Vec<bool>` directly vs `ViperList` struct).

**Solution**:
- Created compatible JIT stubs in `src/jit_stubs/lists_bool.rs`
- Added bool list tracking in codegen (`bool_list_vars`)
- Registered all bool list functions for JIT

**Result**: JIT now works correctly for bool list operations.

### 2. Bool List Type Support ✅

**Implementation**:
- Type-specific bool lists use 1 byte per element (vs 8 bytes for generic i64 lists)
- Runtime functions: `vp_list_bool_*` in `runtime/src/data_structures/list_bool.c`
- Codegen detects `[True] * n` pattern and uses bool-specific functions

**Memory Savings**: 8x reduction for boolean arrays

### 3. Type-Specific Codegen ✅

**Changes**:
- `generate_index()` uses `vp_list_bool_get` for bool lists
- Assignment uses `vp_list_bool_set` for bool values
- Bool list tracking in `CodeGenState`

## Current Performance (N=10,000,000)

| Language | Time | Relative | Memory |
|----------|------|----------|--------|
| **Rust** | **0.109s** | **1.0x** | 1.25 MB (bit vector) |
| **C** | **0.115s** | **1.1x** | 10 MB (byte array) |
| **Go** | **0.131s** | **1.2x** | 10 MB (byte array) |
| **Viper AOT** | **0.222s** | **2.0x** | 10 MB (byte array) |
| **Viper JIT** | **0.297s** | **2.7x** | 10 MB (byte array) |

**Note**: All implementations correctly find 664,579 primes.

## Performance Analysis

### Why Viper is Slower

1. **Function Call Overhead**: Every list access calls `vp_list_bool_get/set()`
   - C/Rust: Direct memory access via pointer arithmetic
   - Viper: Function call + bounds check + indirection

2. **Memory Layout**: 1 byte per bool (same as C/Go)
   - Rust uses `Vec<bool>`: 1 bit per bool (8x more efficient)

3. **LLVM Optimization**: Less aggressive inlining in JIT mode

### What Works Well

- **Bool lists**: 8x memory savings vs generic lists ✅
- **JIT execution**: No more crashes ✅
- **Correctness**: All tests pass ✅

## Pending Optimizations

### 1. Inline List Operations ⚠️ (Partially Implemented)

**Goal**: Generate direct LLVM IR instead of function calls.

**Status**: Implementation created in `src/codegen/inline_lists.rs` but disabled due to JIT/AOT struct layout differences.

**Challenge**: 
- AOT uses C `ViperList` struct from runtime
- JIT uses Rust `ViperList` struct from stubs
- Different memory layouts cause crashes

**Next Steps**:
1. Unify struct layout between AOT and JIT
2. Or: Use different codegen paths for AOT vs JIT
3. Or: Keep runtime functions but add LLVM `alwaysinline` attribute

**Expected Benefit**: 2-3x speedup for tight loops

### 2. Bit Vector Type 📋 (Not Started)

**Goal**: Match Rust's 1-bit-per-element efficiency.

**Design**:
```c
// New ViperBitSet struct
typedef struct {
    int64_t ref_count;
    int64_t length;      // Number of bits
    int64_t capacity;    // Number of bits
    uint8_t* bits;       // Packed bits (8x memory savings)
} ViperBitSet;
```

**Required Changes**:

1. **Runtime** (`runtime/src/data_structures/bitset.c`):
   ```c
   ViperBitSet* vp_bitset_create(int64_t n);
   bool vp_bitset_get(ViperBitSet* bs, int64_t i);
   void vp_bitset_set(ViperBitSet* bs, int64_t i, bool val);
   ```

2. **Codegen** (`src/codegen/bitset.rs`):
   - Detect `BitSet` type
   - Generate bit manipulation: `(bits[i/8] >> (i%8)) & 1`
   - LLVM IR: `lshr`, `and`, `or` operations

3. **Type System** (`src/ast/types.rs`):
   - Add `Type::BitSet`
   - Type checker support

**Expected Benefit**:
- 8x memory savings: 10 MB → 1.25 MB for N=10M
- Better cache utilization
- Potential 2x speedup from reduced memory bandwidth

**Estimated Effort**: 2-3 days

### 3. Stack Allocation for Non-Escaping Lists ⚠️ (Disabled)

**Goal**: Allocate local lists on stack instead of heap.

**Status**: Code exists but disabled for JIT compatibility.

**Challenge**: Stack allocation uses `alloca` which behaves differently in JIT vs AOT.

**Expected Benefit**: Eliminates malloc/free overhead for local lists

### 4. Bounds Check Elimination 📋 (Not Started)

**Goal**: Skip bounds checks in release mode.

**Implementation**:
```rust
// Add flag to CodeGenState
pub unsafe_mode: bool,

// In inline list operations:
if !state.unsafe_mode {
    // Generate bounds check
}
```

**Expected Benefit**: 10-20% speedup for tight loops

## Recommendations

### Immediate (P0)
1. **Fix inline operations**: Unify struct layout or use separate codegen paths
2. **Add `alwaysinline` to runtime functions**: Help LLVM optimize better

### Short-term (P1)
1. **Implement bit vectors**: Biggest performance win (8x memory + cache effects)
2. **Profile with perf**: Identify actual bottlenecks

### Long-term (P2)
1. **SIMD optimizations**: Vectorized bit operations
2. **Parallel sieve**: Multi-threaded implementation
3. **Region-based memory**: Arena allocation for local lists

## Files Modified

### Core Fixes
- `src/jit_stubs/lists_bool.rs` - NEW: JIT bool list stubs
- `src/jit_stubs/registry.rs` - Register bool list stubs
- `src/codegen/state.rs` - Bool list tracking
- `src/codegen/generator.rs` - Bool list state
- `src/codegen/expressions/collections.rs` - Type-specific indexing
- `src/codegen/statements/assignment.rs` - Type-specific assignment

### Inline Operations (Created but Disabled)
- `src/codegen/inline_lists.rs` - NEW: Inline list operations
- `src/codegen/mod.rs` - Module declaration

### Documentation
- `JIT_BOOL_LIST_FIX.md` - Detailed fix documentation
- `SIEVE_RESULTS_10M.md` - N=10M benchmark results
- `SIEVE_OPTIMIZATION_SUMMARY.md` - Complete summary
- `OPTIMIZATION_STATUS.md` - This document

## Conclusion

The Viper compiler now correctly executes bool list operations in both AOT and JIT modes. The 8x memory savings from type-specific bool lists is working correctly.

However, Viper is still 2-3x slower than C/Rust due to:
1. Function call overhead for list operations
2. Byte-per-bool storage (vs Rust's bit-per-bool)

The next big win is implementing bit vectors, which would match Rust's memory efficiency and potentially close the performance gap significantly.

## Build & Test Commands

```bash
# Build runtime
cd runtime && make

# Build compiler
cargo build --release

# Run benchmark
./benchmark/sieve_benchmark.sh

# Individual tests
cargo run --release -- build benchmark/sieve.vp -O 2 -o sieve_vp
./sieve_vp_bin

cargo run --release -- run benchmark/sieve.vp -O 2
```
