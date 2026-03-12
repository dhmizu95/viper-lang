# Viper Compiler Performance Analysis

## Current Benchmark Results (O3)

| Benchmark | Viper JIT | Viper AOT-O3 | C -O3 | Rust -O3 | Go | Gap vs C |
|-----------|-----------|--------------|-------|----------|-----|----------|
| 01_fibonacci | 196ms | 71ms | 13ms | 19ms | 39ms | **5.5x** |
| 02_prime_sieve | 22ms | 4ms | 1ms | 1ms | 2ms | **4x** |
| 03_matrix_mul | 20ms | 6ms | 1ms | 1ms | 2ms | **6x** |
| 04_quicksort | 98ms | 1ms | 1ms | 1ms | 2ms | **1x** ✅ |
| 05_matrix_mul_array | 73ms | 12ms | 2ms | 1ms | 2ms | **6x** |
| 06_prime_sieve_array | 80ms | 18ms | 3ms | 1ms | 2ms | **6x** |
| 07_string_ops | 96ms | 2ms | 2ms | 1ms | 2ms | **1x** ✅ |

## Root Cause Analysis

### 1. Value Tagging Overhead (HIGH - 30-50% impact)
Viper uses tagged values for dynamic typing. Every arithmetic operation requires tag handling.

**Location:** `src/codegen/expressions/core.rs:249-267`

### 2. Missing Function Inlining (HIGH - 20-40% impact)
Recursive calls have full call overhead (stack frame, parameter passing, return handling).

**Location:** `src/codegen/core/functions.rs`

### 3. Alloca-Based Variables (MEDIUM - 15-25% impact)
All locals use alloca + load/store pattern instead of SSA form.

**Location:** `src/codegen/core/functions.rs:100-120`

### 4. Missing Function Attributes (MEDIUM - 10-20% impact)
LLVM can't optimize unknown functions aggressively without hints.

### 5. Runtime Call Overhead (LOW - 5-10% impact)
Simple operations call runtime functions instead of inline code.

## Implementation Priority

### Phase 1: Quick Wins (1-2 days)
1. Add `alwaysinline` for small functions
2. Add `readonly`/`willreturn` attributes to pure functions
3. Verify `mem2reg` runs in pass pipeline

**Expected:** 30-40% overall improvement

### Phase 2: Type Specialization (3-5 days)
4. Add monomorphic type detection
5. Implement untagged math mode for numeric functions

**Expected:** Additional 20-30% for numeric benchmarks

### Phase 3: Advanced Optimizations (1-2 weeks)
6. Constant folding for tagged values
7. Profile-guided optimization (PGO) support
8. Link-time optimization (LTO) improvements
