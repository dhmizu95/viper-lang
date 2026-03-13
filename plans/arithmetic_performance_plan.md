# Arithmetic Performance Improvement Plan

**Date:** 2026-03-13
**Scope:** Reduce arithmetic overhead for `int`-heavy Viper programs while preserving arbitrary-precision semantics.

## Progress Legend

- `INF` = identified / ready to implement
- `WIP` = in progress
- `BLK` = blocked
- `DON` = done

## Problem Statement

**Status:** `INF`

Arithmetic-heavy workloads are slower than they should be because language-level `int` expressions currently lower to tagged-integer runtime calls instead of native LLVM integer instructions. This adds:

- per-operation function call overhead
- repeated small-int tag checks
- fallback allocation cost when promotion to BigInt occurs
- limited optimizer visibility across the runtime boundary

## Evidence

**Status:** `INF`

- `Type::Int` dispatches to tagged-int binary operations in `src/codegen/expressions/operators/core.rs`
- integer literals default to tagged-int-oriented `Type::Int` in `src/codegen/expressions/core.rs`
- tagged-int arithmetic is emitted as runtime calls in `src/codegen/runtime/tagged_int.rs`
- tagged-int runtime helpers perform branching and BigInt fallback in `runtime/src/tagged_int.c`
- AOT emits object code first and links `libviper.a` later, which prevents LLVM from optimizing through those helper bodies in `src/driver/aot.rs`

## Goals

**Status:** `INF`

1. Keep Python-style arbitrary-precision `int` semantics.
2. Reduce overhead for the common case where both operands are small integers.
3. Improve arithmetic-heavy benchmarks such as prime sieve and matrix multiplication.
4. Preserve correctness for overflow, division, modulo, comparisons, and mixed `int`/`BigInt` behavior.

## Non-Goals

**Status:** `INF`

- changing surface-language `int` semantics to fixed-width only
- removing BigInt support
- redesigning the entire numeric type system in the first pass

## Workstreams

### 1. Baseline and Benchmark Harness

**Status:** `INF`

Tasks:

1. Record a fresh baseline for `02_prime_sieve`, `03_matrix_mul`, and `05_matrix_mul`.
2. Add a focused arithmetic microbenchmark if existing benchmarks are too coarse.
3. Capture separate AOT and JIT numbers so improvements are measured in the right mode.

Success criteria:

- baseline numbers are saved before code changes
- arithmetic improvements can be validated on repeatable workloads

### 2. Inline Small-Int Fast Path in Codegen

**Status:** `WIP`

Tasks:

1. Update `src/codegen/expressions/operators/arithmetic.rs` to generate LLVM fast paths for tagged small ints.
2. Detect the common case where both operands are small ints and lower `add`, `sub`, `mul`, comparisons, shifts, and bitwise ops directly in LLVM IR.
3. Branch to the existing runtime helper only when an operand is BigInt or when overflow requires promotion.

Success criteria:

- common arithmetic ops no longer always emit `call @tagged_int_*`
- generated IR exposes the hot path directly to LLVM optimization passes

### 3. Overflow and BigInt Fallback Correctness

**Status:** `INF`

Tasks:

1. Preserve current overflow behavior for `int`.
2. Reuse existing runtime helpers for slow paths instead of duplicating BigInt logic initially.
3. Validate corner cases: negative values, divide by zero, modulo, large literals, and cross-boundary overflow.

Success criteria:

- all existing integer and BigInt behavior remains correct
- overflow transitions still promote to BigInt correctly

### 4. Type-Specialized `i64` Fast Path

**Status:** `INF`

Tasks:

1. Audit where hot code can remain in `Type::I64` instead of widening to `Type::Int`.
2. Tighten inference and codegen so explicitly typed `i64` loops and accumulators stay native.
3. Avoid accidental promotion of obviously fixed-width arithmetic back into tagged-int paths.

Success criteria:

- explicitly typed `i64` arithmetic stays on native LLVM integer ops
- hot loops can be written in a way that reliably bypasses tagged-int overhead

### 5. AOT Optimization Validation

**Status:** `INF`

Tasks:

1. Re-run AOT benchmarks at `-O1`, `-O2`, and `-O3`.
2. Compare generated performance before and after small-int fast-path lowering.
3. Check whether the custom `-O3` pipeline still behaves sensibly after codegen changes.

Success criteria:

- arithmetic-heavy AOT benchmarks improve materially
- no new optimization-level regressions are introduced

### 6. Regression Tests

**Status:** `WIP`

Tasks:

1. Add focused tests for small-int arithmetic correctness.
2. Add tests that force overflow and verify BigInt fallback.
3. Add mixed cases covering comparison, unary negation, and bitwise operations.

Success criteria:

- tests cover both fast and slow paths
- no semantic regressions slip through benchmark-only validation

## Implementation Order

**Status:** `INF`

1. Establish benchmark baseline.
2. Implement inline small-int fast path for arithmetic operators.
3. Preserve runtime fallback for overflow and BigInt cases.
4. Add focused regression tests.
5. Re-benchmark and tune optimization levels.
6. Tighten `i64` specialization where practical.

## Risks

**Status:** `INF`

- incorrect tagging or untagging can silently corrupt integer values
- signed overflow checks may differ by operator if lowered carelessly
- division and modulo semantics may diverge if fast-path lowering does not match runtime behavior
- mixed `int`/`BigInt` expressions may regress if dispatch shortcuts are too aggressive

## Exit Criteria

**Status:** `INF`

- arithmetic-heavy benchmarks show a measurable improvement
- integer and BigInt tests pass
- small-int operations are visibly lowered inline in generated IR
- the slow path remains correct for overflow and true BigInt operands
