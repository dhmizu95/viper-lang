# Viper Compiler Optimization Plan

## Goal
Improve Viper AOT compilation performance to be within 1.5-2x of C/Rust for array-heavy and arithmetic-heavy workloads.

## Current Performance Baseline (String Operations Benchmark)

| Language | Time | vs Viper |
|----------|------|----------|
| C -O3 | 1ms | 3x faster |
| Rust -O3 | 1ms | 3x faster |
| Go | 2ms | 1.5x faster |
| **Viper AOT -O2** | **3ms** | baseline |

## Optimizations

### 1. Inline List Access (Priority: HIGH)
**File:** `src/codegen/expressions/collections/index.rs`
**Expected Gain:** 40-50%
**Effort:** Low

Replace runtime function calls with direct LLVM IR GEP + load/store operations.

### 2. Type-Specialized Arithmetic (Priority: HIGH)
**File:** `src/codegen/expressions/operators/arithmetic.rs`
**Expected Gain:** 30-40%
**Effort:** Medium

Generate native i64 operations for known integer types, skip tagging overhead.

### 3. Loop Unrolling (Priority: MEDIUM)
**File:** `src/codegen/control_flow/loops.rs`
**Expected Gain:** 20-30%
**Effort:** Medium

Detect counting loops and apply unrolling (factor=4).

### 4. Bounds Check Elision (Priority: MEDIUM)
**File:** `src/codegen/expressions/collections/index.rs`
**Expected Gain:** 15-25%
**Effort:** High

Skip bounds checks when index is provably in range.

### 5. SSA Promotion (Priority: LOW)
**File:** `src/codegen/variables.rs`
**Expected Gain:** 10-15%
**Effort:** High

Promote loop-local scalars to SSA form instead of stack allocas.

## Implementation Order

1. Inline List Access → Quick win, existing infrastructure
2. Type-Specialized Arithmetic → High impact, localized changes
3. Loop Unrolling → Medium complexity
4. Bounds Check Elision → Requires analysis infrastructure
5. SSA Promotion → Most complex, requires dataflow analysis

## Success Criteria

After all optimizations:
- String Operations: ≤1.5ms (currently 3ms) - 2x improvement
- Matrix Mul Array: ≤8ms (currently 12ms) - 1.5x improvement
- Prime Sieve Array: ≤7ms (currently 11ms) - 1.5x improvement
- Overall: Within 1.5-2x of C performance
