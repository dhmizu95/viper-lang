# Native Int Arithmetic Optimization Plan

**Date:** 2026-03-13
**Goal:** Improve performance without compromising the higher-priority goal of preserving Python-like syntax and behavior. Performance work should stay behind Python syntax compatibility and Pythonic semantics.

## Progress Legend

- `INF` = identified / ready
- `WIP` = in progress
- `BLK` = blocked
- `DON` = done

## Current Status

**Overall:** `INF`

The dominant avoidable cost is that ordinary Viper `int` arithmetic takes the tagged-int runtime path even when both operands are small integers. This prevents LLVM from optimizing hot arithmetic loops effectively.

## Priority Order

**Status:** `DON`

1. Python syntax compatibility and Python-like language feel
2. Pythonic runtime semantics where intentionally supported
3. Performance improvements that do not distort the language model
4. C-near performance for hot paths where it can be achieved safely

## Guardrails

**Status:** `DON`

Performance changes must not:

- introduce syntax that pulls the language away from Python style
- force users to write non-Pythonic code just to get good performance
- break or weaken arbitrary-precision `int` semantics
- prefer benchmark wins over language consistency

Performance changes should:

- preserve current Python-like source syntax
- optimize under the hood in codegen/runtime
- keep the fast path invisible to end users

## Primary Objective

**Status:** `INF`

Compile the common case of `int` arithmetic as native LLVM integer operations with a runtime fallback only when:

- either operand is a BigInt
- the result overflows the small-int range
- the operation requires runtime-only behavior

## Phase 1: Native Small-Int Fast Path

### 1.1 Lower small-int detection in LLVM IR

**Status:** `INF`

Files:

- `src/codegen/expressions/operators/arithmetic.rs`
- `src/codegen/expressions/operators/core.rs`

Tasks:

1. Add IR to detect whether tagged `int` operands are small ints.
2. Create `fast_path`, `slow_path`, and `merge` basic blocks.
3. Preserve existing runtime helper behavior in the slow path.

### 1.2 Inline `Add`, `Sub`, `Mul`

**Status:** `INF`

Tasks:

1. Untag both small-int operands.
2. Emit native LLVM `add`, `sub`, and `mul`.
3. Check small-int range before retagging.
4. Fall back to `tagged_int_*` helper on overflow.

### 1.3 Inline comparisons

**Status:** `INF`

Tasks:

1. Emit native compare for `Eq`, `NotEq`, `Lt`, `Gt`, `LtEq`, `GtEq`.
2. Use runtime compare only when a BigInt is involved.

### 1.4 Extend to bitwise and shifts

**Status:** `INF`

Tasks:

1. Inline `BitAnd`, `BitOr`, `BitXor`, `LShift`, `RShift` for small ints.
2. Match current runtime semantics exactly.

### 1.5 Handle division/modulo carefully

**Status:** `INF`

Tasks:

1. Add fast paths for `Div`, `FloorDiv`, and `Mod`.
2. Preserve divide-by-zero behavior.
3. Verify sign and rounding behavior against current runtime semantics.

## Phase 2: Keep `i64` Truly Native

### 2.1 Prevent accidental widening back to `Type::Int`

**Status:** `INF`

Files:

- `src/codegen/expressions/core.rs`
- `src/codegen/functions.rs`
- `src/semantic/type_checker/exprs.rs`

Tasks:

1. Audit type inference where fixed-width arithmetic degrades back into tagged `int`.
2. Keep explicitly typed `i64` locals, loop counters, parameters, and return values on native integer ops.
3. Treat `i64` as an optimization tool for explicit low-level code, not as the default user-facing recommendation when Python-style `int` should work well.

### 2.2 Add benchmark pair for `int` vs `i64`

**Status:** `DON`

Reference benchmarks:

- `benchmarks/viper/08_int_hotloop.vp`
- `benchmarks/viper/09_i64_hotloop.vp`

## Phase 3: Eliminate Other Hot Runtime Boundaries

### 3.1 Inline list/array element access

**Status:** `INF`

Tasks:

1. Replace runtime get/set calls with direct GEP + load/store where layout is known.
2. Specialize for contiguous numeric arrays and lists.

### 3.2 Bounds-check reduction

**Status:** `INF`

Tasks:

1. Skip redundant bounds checks in proven-safe loops.
2. Keep safe fallback behavior when proof is not available.

### 3.3 Reduce alloca/load/store traffic

**Status:** `INF`

Tasks:

1. Keep short-lived scalars in SSA-friendly form.
2. Reduce unnecessary stack slots in hot loops.

### 3.4 Improve function call inlining

**Status:** `INF`

Tasks:

1. Inline tiny arithmetic helpers aggressively.
2. Add or refine function attributes to help LLVM inline effectively.

## Phase 4: Optimize the AOT Pipeline

### 4.1 Tune optimization levels for generated IR

**Status:** `INF`

Files:

- `src/driver/aot.rs`

Tasks:

1. Re-measure `-O1`, `-O2`, and `-O3` after arithmetic fast-path lowering.
2. Identify regressions caused by generic LLVM pipelines.
3. Keep the best pass pipeline for Viper-generated IR.

### 4.2 Add PGO after IR quality improves

**Status:** `INF`

Tasks:

1. Use representative arithmetic-heavy workloads.
2. Validate whether PGO helps dispatch-heavy or branch-heavy code.

## Phase 5: Validation and Regression Safety

### 5.1 Correctness tests

**Status:** `INF`

Tasks:

1. Add tests for small-int fast path.
2. Add overflow transition tests.
3. Add mixed small-int/BigInt tests.
4. Add division/modulo semantic tests.

### 5.2 Benchmark tracking

**Status:** `INF`

Primary benchmarks:

- `02_prime_sieve`
- `05_matrix_mul`
- `08_int_hotloop`
- `09_i64_hotloop`
- `10_function_calls`
- `12_bigint_overflow`

Tasks:

1. Record before/after timings.
2. Compare JIT vs AOT only when relevant.
3. Use `benchmarks/benchmark_runner.sh` as the reporting source of truth.

## All Possible High-Value Fixes

**Status:** `INF`

Ordered by expected impact:

1. Native small-int fast path for tagged `int`
2. Preserve native `i64` end-to-end
3. Inline list/array access
4. Reduce bounds-check overhead
5. Reduce stack traffic and improve SSA promotion
6. Improve function inlining
7. Tune AOT optimization pipeline
8. Apply PGO after codegen improvements
9. Add workload-specific specialization for numeric loops
10. Revisit runtime layout choices only after codegen wins are exhausted

## Language-First Success Criteria

**Status:** `INF`

- Python-like syntax remains the primary authoring experience
- Common Python-style `int` code gets faster without requiring source rewrites
- Users are not pushed toward non-Pythonic annotations just to recover baseline performance

## Exit Criteria

**Status:** `INF`

- Small-int arithmetic no longer always emits `call @tagged_int_*`
- Arithmetic-heavy AOT benchmarks improve measurably
- `i64` benchmarks stay clearly faster than tagged `int`
- Correctness is preserved for BigInt fallback and overflow transitions
- The optimization path is benchmarked and documented
