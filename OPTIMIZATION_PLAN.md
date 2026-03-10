# Viper Language Optimization Plan

**Date:** March 10, 2026  
**Version:** 0.5.0  
**Status:** Active Development  

---

## Executive Summary

This document outlines the comprehensive optimization strategy for the Viper Language compiler. The optimization plan is organized into phases, from immediate low-risk improvements to advanced long-term optimizations.

### Current State

| Optimization | Status | Implementation |
|--------------|--------|----------------|
| Dead Code Elimination (DCE) | ✅ Implemented | `src/codegen/dce.rs` |
| Escape Analysis | ✅ Implemented | `src/semantic/escape_analysis.rs` |
| Inline List Operations | ✅ Implemented | `src/codegen/inline_lists.rs` |
| ARC (Automatic Reference Counting) | ✅ Implemented | `src/codegen/state.rs` |
| LLVM JIT Optimization | ✅ Implemented | `src/driver/jit.rs` |
| LLVM AOT Optimization | ✅ Implemented | `src/driver/aot.rs` |
| Constant Folding | ❌ Not Implemented | Planned (Phase 1) |
| Function Inlining | ❌ Not Implemented | Planned (Phase 2) |
| Loop Optimizations | ❌ Not Implemented | Planned (Phase 2) |
| PGO (Profile-Guided Optimization) | ⚠️ Partial | Profile generation in `Cargo.toml` |

### Performance Baseline (from benchmarks/README.md)

| Benchmark | Viper JIT | Viper AOT -O2 | C -O3 | Gap to C |
|-----------|-----------|---------------|-------|----------|
| Fibonacci | 186ms | 111ms | 14ms | 7.9x |
| Prime Sieve | 26ms | 4ms | 1ms | 4x |
| Matrix Mul | 1ms | 8ms | 1ms | ~1x |

---

## Phase 1: Low-Risk Immediate Optimizations (Week 1-2)

### 1.1 Constant Folding and Constant Propagation

**Priority:** P0 - High  
**Risk:** Low  
**Estimated Impact:** 10-30% improvement on arithmetic-heavy code  

#### Description
Evaluate constant expressions at compile-time instead of runtime.

**Examples:**
```python
# Before optimization
x = 2 + 3 * 4      # Runtime: add, multiply
y = x + 10         # Runtime: add

# After constant folding
x = 14             # Compile-time evaluation
y = 24             # Compile-time evaluation
```

#### Implementation Plan

1. **AST-Level Constant Folding** (`src/semantic/constant_folding.rs`)
   - Create new module for constant folding
   - Implement fold for arithmetic operations (+, -, *, /, %, //, **)
   - Implement fold for comparison operations (==, !=, <, >, <=, >=)
   - Implement fold for logical operations (and, or, not)
   - Implement fold for bitwise operations (&, |, ^, <<, >>)

2. **Integration Points**
   - Run after type checking in `src/driver/aot.rs`
   - Run before DCE optimization
   - Apply at optimization levels -O1 and above

3. **Safety Measures**
   - Preserve overflow behavior for tagged integers
   - Handle BigInt constants correctly
   - Don't fold expressions with side effects

#### Files to Create/Modify
- `src/semantic/constant_folding.rs` (new)
- `src/semantic/mod.rs` (add constant_folding module)
- `src/driver/aot.rs` (integrate into compilation pipeline)
- `src/driver/jit.rs` (integrate into JIT pipeline)

#### Testing Strategy
- Unit tests for each operator type
- Integration tests with known constant expressions
- Verify no changes to runtime behavior
- Benchmark before/after on arithmetic-heavy code

---

### 1.2 Dead Store Elimination Enhancement

**Priority:** P0 - High  
**Risk:** Low  
**Estimated Impact:** 5-15% improvement  

#### Description
Current DCE removes unused variables but can be enhanced to detect more dead stores.

#### Implementation Plan

1. **Enhanced Dead Store Detection** (`src/codegen/dce.rs`)
   - Track write-write pairs without intervening reads
   - Consider control flow (stores in unreachable branches)
   - Use SSA-like analysis for better precision

2. **Integration with Escape Analysis**
   - Non-escaping variables with dead stores can be fully eliminated
   - Combine with ARC elision for maximum benefit

#### Files to Modify
- `src/codegen/dce.rs` (enhance existing implementation)

---

### 1.3 Peephole Optimization

**Priority:** P1 - Medium  
**Risk:** Low  
**Estimated Impact:** 5-10% improvement  

#### Description
Simple pattern-matching optimizations on generated LLVM IR.

**Examples:**
```llvm
; Before
%1 = add i64 %x, 0      ; Adding zero is no-op
%2 = mul i64 %x, 1      ; Multiplying by one is no-op
%3 = sub i64 %x, %x     ; Self-subtraction is zero

; After
%1 = %x
%2 = %x
%3 = 0
```

#### Implementation Plan

1. **IR-Level Peephole Pass** (`src/codegen/peephole.rs`)
   - Create new module for peephole optimizations
   - Pattern match common inefficiencies
   - Apply before LLVM optimization pipeline

2. **Patterns to Detect**
   - Identity operations (x + 0, x * 1, x - x, x / 1)
   - Double negation (~~x = x)
   - Redundant casts (int→int, ptr→ptr same type)
   - Strength reduction (x * 2 → x << 1)

#### Files to Create
- `src/codegen/peephole.rs` (new)

---

## Phase 2: Medium-Term Optimizations (Week 3-6)

### 2.1 Function Inlining

**Priority:** P0 - High  
**Risk:** Medium  
**Estimated Impact:** 20-50% improvement on function-heavy code  

#### Description
Inline small functions to eliminate call overhead and enable further optimization.

#### Implementation Plan

1. **Inline Heuristics** (`src/codegen/inline.rs`)
   - Size threshold (e.g., < 10 instructions)
   - Call frequency (hot functions prioritized)
   - Recursive function detection (avoid infinite inlining)
   - Cold function marking (don't inline cold code)

2. **Implementation Strategy**
   - AST-level inlining for simple functions
   - Let LLVM handle complex inlining at -O2 and above
   - Add `@inline` and `@noinline` decorators for user control

3. **Integration**
   - Run after constant folding
   - Run before DCE (inlining creates new DCE opportunities)

#### Files to Create
- `src/codegen/inline.rs` (new)
- Add `@inline` decorator support in parser

---

### 2.2 Loop Optimizations

**Priority:** P1 - Medium  
**Risk:** Medium  
**Estimated Impact:** 2-5x improvement on loop-heavy code  

#### Description
Optimize loop structures for better performance.

#### Optimizations

1. **Loop Invariant Code Motion (LICM)**
   - Move computations outside loop if operands don't change
   ```python
   # Before
   for i in range(n):
       x = a + b  # a, b don't change
       y = x * i
   ```

2. **Loop Unrolling**
   - Reduce loop overhead by processing multiple elements per iteration
   - Configurable unroll factor (2x, 4x, 8x)

3. **Strength Reduction**
   - Replace expensive operations with cheaper ones
   - `x * 2` → `x << 1`
   - `x / 2` → `x >> 1` (for positive integers)

4. **Loop Fusion**
   - Combine adjacent loops iterating over same range
   ```python
   # Before
   for i in range(n):
       a[i] = b[i] + 1
   for i in range(n):
       c[i] = a[i] * 2

   # After
   for i in range(n):
       a[i] = b[i] + 1
       c[i] = a[i] * 2
   ```

#### Implementation Plan

1. **Loop Analysis Framework** (`src/codegen/loop_analysis.rs`)
   - Identify loop headers, bodies, latches
   - Compute loop depth and nesting
   - Detect loop-carried dependencies

2. **LICM Implementation** (`src/codegen/licm.rs`)
   - Identify invariant expressions
   - Move to pre-header block
   - Update use-def chains

3. **Integration with LLVM**
   - Let LLVM handle complex LICM at -O2+
   - Implement simple AST-level LICM for -O1

#### Files to Create
- `src/codegen/loop_analysis.rs` (new)
- `src/codegen/licm.rs` (new)

---

### 2.3 Type Specialization

**Priority:** P1 - Medium  
**Risk:** Medium  
**Estimated Impact:** 10-30% improvement  

#### Description
Generate specialized code for monomorphic type instances.

#### Example
```python
# Generic (current approach)
def add(a, b):
    return a + b

# Specialized (optimized)
def add_int(a: int, b: int) -> int:
    return tagged_add_int(a, b)

def add_float(a: float, b: float) -> float:
    return fadd(a, b)
```

#### Implementation Plan

1. **Type Specialization Framework** (`src/codegen/specialize.rs`)
   - Track monomorphic instances
   - Generate specialized versions
   - Update call sites

2. **Integration with Type Checker**
   - Use type inference results
   - Specialize at call sites

---

## Phase 3: Advanced Optimizations (Week 7-12)

### 3.1 Profile-Guided Optimization (PGO)

**Priority:** P2 - Medium  
**Risk:** Low  
**Estimated Impact:** 10-30% improvement  

#### Description
Use runtime profiling data to guide optimization decisions.

#### Current State
- `Cargo.toml` has PGO profiles defined (`pgo-instrument`, `pgo`)
- No runtime profiling infrastructure

#### Implementation Plan

1. **Instrumentation Build** (`src/driver/pgo.rs`)
   - Build with coverage counters
   - Generate profiling runtime

2. **Profile Collection**
   - Run benchmark suite with instrumented binary
   - Collect `.profraw` files

3. **Profile Merge and Apply**
   - Merge profiles with `llvm-profdata`
   - Build with `-Cprofile-use`

4. **Automation**
   - Add `make pgo` target
   - Integrate with CI/CD

---

### 3.2 Interprocedural Analysis (IPA)

**Priority:** P2 - Medium  
**Risk:** High  
**Estimated Impact:** 20-50% improvement  

#### Description
Analyze across function boundaries for better optimization.

#### Optimizations Enabled
- Cross-function constant propagation
- Dead function elimination
- Argument specialization
- Return value optimization

#### Implementation Plan

1. **Call Graph Construction** (`src/semantic/call_graph.rs`)
   - Build whole-program call graph
   - Detect recursion cycles

2. **IPA Framework** (`src/semantic/ipa.rs`)
   - Bottom-up analysis (leaves to roots)
   - Top-down propagation (roots to leaves)

---

### 3.3 Parallelization

**Priority:** P3 - Low  
**Risk:** High  
**Estimated Impact:** 2-8x on parallelizable workloads  

#### Description
Automatically parallelize independent loop iterations.

#### Prerequisites
- Dependence analysis infrastructure
- Thread runtime support
- Work-stealing scheduler

#### Implementation Plan

1. **Dependence Analysis** (`src/codegen/dependence.rs`)
   - Detect loop-carried dependencies
   - Identify parallelizable loops

2. **Parallel Code Generation**
   - Generate thread-safe code
   - Insert synchronization where needed

---

## Phase 4: Platform-Specific Optimizations (Week 13+)

### 4.1 SIMD Vectorization

**Priority:** P3 - Low  
**Risk:** Medium  
**Estimated Impact:** 4-16x on vectorizable code  

#### Description
Use SIMD instructions for data-parallel operations.

#### Target Operations
- Vector arithmetic (list + list, list * scalar)
- Reductions (sum, product, min, max)
- Comparisons

---

### 4.2 Cache Optimization

**Priority:** P3 - Low  
**Risk:** Medium  
**Estimated Impact:** 2-5x on memory-bound code  

#### Optimizations
- Loop tiling/blocking
- Data structure padding
- Prefetching hints

---

## Optimization Pipeline

### Current Pipeline (AOT)

```
Source → Lexer → Parser → Type Check → [DCE] → CodeGen → LLVM IR → LLVM Opt → Binary
```

### Target Pipeline (-O1)

```
Source → Lexer → Parser → Type Check → Const Fold → Peephole → DCE → CodeGen → LLVM IR → LLVM Opt → Binary
```

### Target Pipeline (-O2)

```
Source → Lexer → Parser → Type Check → Const Fold → Peephole → Inline → DCE → LICM → CodeGen → LLVM IR → LLVM Opt → Binary
```

### Target Pipeline (-O3)

```
Source → Lexer → Parser → Type Check → Const Fold → Peephole → Inline → Type Spec → DCE → LICM → CodeGen → LLVM IR → LLVM Opt + PGO → Binary
```

---

## Testing Strategy

### Unit Tests
- Each optimization pass has dedicated test file
- Test transformation correctness
- Test edge cases

### Integration Tests
- End-to-end compilation tests
- Verify output correctness after optimization
- Compare optimized vs unoptimized output

### Performance Tests
- Benchmark suite in `benchmarks/`
- Track regression with each optimization
- Set performance budgets

### Safety Tests
- Fuzzing with optimized builds
- Memory safety (valgrind, ASan)
- Undefined behavior detection (UBSan)

---

## Success Metrics

### Performance Goals

| Benchmark | Current (AOT -O2) | Target (-O2) | Target (-O3) |
|-----------|-------------------|--------------|--------------|
| Fibonacci | 111ms | 50ms (2.2x) | 25ms (4.4x) |
| Prime Sieve | 4ms | 2ms (2x) | 1.5ms (2.7x) |
| Matrix Mul | 8ms | 4ms (2x) | 2ms (4x) |

### Code Quality Goals
- Zero regressions on existing tests (434+ tests)
- No new memory leaks
- Compile time overhead < 20% for -O1
- Compile time overhead < 50% for -O2

---

## Risk Mitigation

### Low-Risk Optimizations (Phase 1)
- Constant folding: Well-understood, easy to verify
- Peephole: Local transformations, minimal interaction
- Dead store enhancement: Builds on existing DCE

### Medium-Risk Optimizations (Phase 2)
- Inlining: Test thoroughly with recursive functions
- Loop optimizations: Verify loop semantics preserved
- Type specialization: Ensure type safety maintained

### High-Risk Optimizations (Phase 3+)
- IPA: Complex interactions, extensive testing needed
- Parallelization: Concurrency bugs possible
- SIMD: Platform-specific issues

### General Mitigation Strategies
1. **Incremental Deployment**: Enable one optimization at a time
2. **Fallback Path**: Keep unoptimized path for debugging
3. **Extensive Testing**: Unit + integration + performance tests
4. **Verification**: Compare optimized output against reference
5. **Documentation**: Document each optimization for maintainability

---

## Implementation Timeline

| Week | Phase | Tasks |
|------|-------|-------|
| 1-2 | Phase 1 | Constant folding, dead store enhancement, peephole |
| 3-4 | Phase 2 | Function inlining |
| 5-6 | Phase 2 | Loop optimizations (LICM, unrolling) |
| 7-8 | Phase 2 | Type specialization |
| 9-10 | Phase 3 | PGO infrastructure |
| 11-12 | Phase 3 | Interprocedural analysis |
| 13+ | Phase 4 | SIMD, cache optimization |

---

## Appendix: Optimization Attributes

### User-Controlled Optimization

```python
# Force inlining
@inline
def fast_add(a: int, b: int) -> int:
    return a + b

# Prevent inlining (for large cold functions)
@noinline
def large_cold_function(...):
    ...

# Optimization hints
@cold
def error_handler(...):
    ...

@hot
def main_loop(...):
    ...
```

### Compiler-Controlled Optimization

```python
# Optimization level pragmas
# pragma optimize: O0  # No optimization
# pragma optimize: O1  # Basic optimization
# pragma optimize: O2  # Aggressive optimization
# pragma optimize: O3  # Maximum optimization
```

---

## References

- LLVM Optimization Guide: https://llvm.org/docs/Passes.html
- Engineering a Compiler (Cooper & Torczon)
- Advanced Compiler Design (Muchnick)
- Viper Language Benchmarks: `benchmarks/README.md`
- Current Test Coverage: `TEST_COVERAGE_REPORT.md`

---

*Last Updated: March 10, 2026*  
*Version: 0.5.0*  
*Author: Viper Language Team*
