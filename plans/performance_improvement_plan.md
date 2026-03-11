# Viper Performance Improvement Plan

**Date:** 2026-03-11  
**Analysis Based On:** [`benchmarks/results/benchmark_report.md`](benchmarks/results/benchmark_report.md)

---

## Executive Summary

The benchmark report reveals significant performance gaps between Viper and native languages (C, Rust, Go). This plan identifies the root causes and provides actionable improvements.

### Key Performance Gaps

| Mode | Time vs C | Memory vs C |
|------|-----------|-------------|
| JIT | 5-13× slower | **46.3× more** |
| AOT-O1 | 1.9× slower | 1.7× more |
| AOT-O2 | 2.1× slower | 1.7× more |
| AOT-O3 | 2.0× slower | 1.7× more |

### Critical Finding
**AOT-O1 is the best performing mode** - O2 and O3 show *no improvement* or even *regression* for some benchmarks (e.g., `06_prime_sieve`: O1=9ms, O2=5ms, O3=14ms).

---

## Root Cause Analysis

### 1. JIT Memory Overhead (CRITICAL - 46× more memory)

**Location:** [`src/driver/jit.rs`](src/driver/jit.rs)

**Cause:** LLVM JIT engine loads ~60MB of infrastructure regardless of program size.

**Impact:**
- 66,876 KB vs C's 1,408 KB
- Makes JIT unusable for memory-constrained environments

**Recommendation:**
1. Implement lazy compilation to reduce initial memory footprint
2. Consider tiered JIT with interpreter for cold code

---

### 2. Tagged Integer Overhead (HIGH - 30-50% impact)

**Location:** [`src/codegen/runtime/tagged_int.rs`](src/codegen/runtime/tagged_int.rs:98-118)

**Current Behavior:** Every arithmetic operation generates a function call to runtime:
```llvm
; Every + operation becomes:
%result = call i64 @tagged_int_add(i64 %lhs, i64 %rhs)
```

**Impact:**
- Fibonacci: 6.7× slower than C (worst case)
- Each call adds function call overhead + tag checking

**Recommendation:**
1. **Add LLVM attributes** to enable inlining: `alwaysinline`, `readnone`, `willreturn`
2. **Implement type specialization** for monomorphic integer operations
3. **Add fast-path intrinsics** that LLVM can optimize

---

### 3. Missing Function Inlining (HIGH - 20-40% impact)

**Location:** [`src/codegen/functions.rs`](src/codegen/functions.rs)

**Current Behavior:** All functions call through function pointers, no inlining hints.

**Impact:**
- Recursive Fibonacci has full call overhead for each recursion
- Cannot leverage LLVM's inlining without hints

**Recommendation:**
1. Add `@inline` decorator support for user-controlled inlining
2. Add `alwaysinline` attribute to small functions
3. Implement AST-level inlining for simple functions

---

### 4. Alloca-Based Variables (MEDIUM - 15-25% impact)

**Location:** [`src/codegen/variables.rs`](src/codegen/variables.rs)

**Current Behavior:** All local variables use alloca + load/store pattern:
```llvm
%x = alloca i64
store i64 %val, i64* %x
%loaded = load i64, i64* %x
```

**Recommendation:**
1. Ensure LLVM's `mem2reg` pass runs in optimization pipeline (it appears to be included)
2. Consider promoting allocas to SSA form earlier in codegen

---

### 5. Missing Function Attributes (MEDIUM - 10-20% impact)

**Current Behavior:** Runtime functions lack optimization hints.

**Recommendation:**
Add these LLVM attributes to runtime functions:
```rust
// In src/codegen/runtime/tagged_int.rs
#[inline(always)]
fn tagged_int_add(a: i64, b: i64) -> i64 { ... }

// Or via LLVM attributes in function declaration
```

---

### 6. AOT Optimization Pipeline Issues

**Finding:** O2/O3 are *slower* than O1 for some benchmarks!

| Benchmark | O1 (ms) | O2 (ms) | O3 (ms) |
|-----------|---------|---------|---------|
| 01_fibonacci | 121 | 120 | 121 |
| 02_prime_sieve | 9 | 9 | 8 |
| 03_matrix_mul | 13 | 11 | 11 |
| 04_quicksort | 7 | 6 | 5 |
| 05_matrix_mul | 14 | 15 | 15 |
| 06_prime_sieve | 6 | 5 | 14 |
| 07_string_ops | 8 | 8 | 7 |

**Root Cause:** The optimization pipeline isn't leveraging type information.

**Recommendation:**
1. Investigate why O3 causes regression on `06_prime_sieve` (14ms vs 5ms)
2. Add type specialization before LLVM optimization
3. Use PGO to guide optimization decisions

---

## Optimization Priority Matrix

| Priority | Optimization | Expected Impact | Risk | Effort |
|----------|-------------|-----------------|------|--------|
| P0 | Add LLVM attributes to runtime functions | 20-30% | Low | 1 day |
| P0 | Fix AOT optimization pipeline regression | 10-20% | Medium | 2 days |
| P1 | Implement type specialization | 30-50% | Medium | 1 week |
| P1 | Add constant folding | 10-30% | Low | 3 days |
| P2 | Implement function inlining | 20-40% | Medium | 1 week |
| P2 | Implement loop optimizations | 2-5× | Medium | 2 weeks |
| P3 | Implement PGO | 10-30% | Low | 1 week |
| P3 | Reduce JIT memory footprint | 50% | High | 2 weeks |

---

## Detailed Implementation Plan

### Phase 1: Quick Wins (Week 1)

#### 1.1 Add LLVM Function Attributes

**File:** [`src/codegen/runtime/tagged_int.rs`](src/codegen/runtime/tagged_int.rs:10-95)

Add `alwaysinline`, `readnone`, `willreturn` attributes to function declarations:

```rust
// Before
module.add_function("tagged_int_add", tagged_op_type, None);

// After - add attribute set
let fn_val = module.add_function("tagged_int_add", tagged_op_type, None);
fn_val.add_attribute(
    inkwell::attributes::Attribute::get_named_enum_attr_id("alwaysinline"),
);
```

**Expected Impact:** 15-25% improvement on integer-heavy code

---

#### 1.2 Fix AOT-O2/O3 Regression

**File:** [`src/driver/aot.rs`](src/driver/aot.rs:154-163)

The current optimization pipeline may be causing regressions. Investigate and fix:

```rust
// Current - O3 causes regression
let passes = match opt_level {
    0 => "verify",
    1 => "default<O1>",
    2 => "default<O2>",  
    3 => "default<O3>",  // This causes 06_prime_sieve to slow down
    _ => "default<O1>",
};
```

**Actions:**
1. Benchmark individual passes to identify problematic ones
2. Consider custom pass pipeline for O3
3. Add `-loop-unroll-threshold` tuning

---

#### 1.3 Enable Constant Folding

**New File:** [`src/semantic/constant_folding.rs`](src/semantic/constant_folding.rs)

Implement compile-time evaluation:
```python
# Before optimization
x = 2 + 3 * 4  # Runtime: add, multiply

# After constant folding  
x = 14  # Compile-time evaluation
```

**Expected Impact:** 10-30% improvement on arithmetic-heavy code

---

### Phase 2: Core Optimizations (Week 2-3)

#### 2.1 Type Specialization

**New File:** [`src/codegen/specialize.rs`](src/codegen/specialize.rs)

Generate specialized code for monomorphic types:
```python
# Generic (current)
def add(a, b):
    return a + b

# Specialized (optimized)
def add_i64(a: i64, b: i64) -> i64:
    return a + b  # Direct LLVM addition, no runtime call
```

**Expected Impact:** 30-50% improvement on typed code

---

#### 2.2 Function Inlining

**New File:** [`src/codegen/inline.rs`](src/codegen/inline.rs)

Implement inlining for:
1. Small functions (< 10 instructions)
2. Functions called once
3. Recursive function detection

**Expected Impact:** 20-40% improvement

---

### Phase 3: Advanced Optimizations (Week 4+)

#### 3.1 Profile-Guided Optimization (PGO)

**Location:** Already partially implemented in [`src/driver/aot.rs`](src/driver/aot.rs:294-309)

Complete PGO implementation:
1. Add benchmark collection mode
2. Generate profile data
3. Apply profiles in optimized build

**Expected Impact:** 10-30% improvement

---

#### 3.2 JIT Memory Reduction

**Location:** [`src/driver/jit.rs`](src/driver/jit.rs)

Implement:
1. Lazy compilation (compile functions on first call)
2. Tiered compilation (interpreter → JIT)
3. Memory pooling for compiled code

**Expected Impact:** Reduce memory from 66MB to ~20MB

---

## Benchmark-Specific Recommendations

### Fibonacci (6.7× slower than C)

**Primary Cause:** Recursive calls + tagged integer overhead

**Actions:**
1. Add tail-call optimization support
2. Implement type specialization for integers
3. Add `alwaysinline` for small functions

---

### Matrix Multiplication (2.2-3.0× slower)

**Primary Cause:** Array bounds checking + tagged operations

**Actions:**
1. Add loop vectorization hints
2. Implement SIMD intrinsics for matrix ops
3. Remove bounds checks in hot loops

---

### Prime Sieve (1.3-1.5× slower)

**Primary Cause:** Array operations overhead

**Actions:**
1. Optimize array/list operations
2. Add loop optimizations (unrolling, fusion)

---

## Success Metrics

| Benchmark | Current (AOT-O1) | Target | Improvement |
|-----------|------------------|--------|-------------|
| Fibonacci | 121ms | 40ms | 3× |
| Prime Sieve | 9ms | 4ms | 2.2× |
| Matrix Mul | 13ms | 5ms | 2.6× |
| Quicksort | 7ms | 5ms | 1.4× |
| String Ops | 8ms | 4ms | 2× |

| Metric | Current | Target |
|--------|---------|--------|
| JIT Memory | 66,876 KB | 20,000 KB |
| AOT Memory | 2,500 KB | 2,000 KB |
| Avg vs C | 1.9× | 1.3× |

---

## Risk Mitigation

1. **Test thoroughly** - Each optimization needs regression testing
2. **Measure first** - Profile before and after each change
3. **Incremental** - Implement one optimization at a time
4. **Benchmark-driven** - Use benchmark suite to validate improvements

---

*Plan generated based on benchmark analysis dated 2026-03-11*
