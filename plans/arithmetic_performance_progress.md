# Arithmetic Performance Progress Tracker

**Date Started:** 2026-03-13
**Plan Reference:** `plans/arithmetic_performance_plan.md`

## Status Legend

- `INF` = identified / queued
- `WIP` = in progress
- `BLK` = blocked
- `DON` = done

## Current Overall Status

- Overall: `WIP`
- Primary target: reduce tagged-int overhead in arithmetic-heavy code
- Current bottleneck confirmed in codegen and runtime dispatch
- Active slice: inline small-int fast path for tagged `add`/`sub`/`mul` in LLVM IR while preserving runtime fallback
- Current implementation now also covers tagged comparisons, bitwise ops, shifts, and unary negation fast paths
- Post-change benchmark spot checks captured for JIT `-O3` and AOT `-O1`/`-O2`; AOT `-O3` is currently blocked by the LLVM `opt` pipeline on this machine

## Milestones

| Milestone | Status | Notes |
|----------|--------|-------|
| Baseline arithmetic benchmarks captured | `BLK` | Clean pre-change baseline was not saved before implementation started |
| Small-int inline fast path designed | `DON` | Tagged `add`/`sub`/`mul`, comparisons, bitwise ops, shifts, and unary negation now lower inline for small ints |
| Runtime fallback preserved | `DON` | Slow path remains the existing tagged-int runtime helpers; mixed BigInt slow-path cleanup applied in runtime |
| Regression tests added | `DON` | Added focused overflow, negative-value, shift, bitwise, unary-negation, and mixed BigInt coverage |
| AOT benchmark validation completed | `WIP` | JIT `-O3` and AOT `-O1`/`-O2` spot checks ran; AOT `-O3` blocked by current LLVM `opt` pass pipeline |
| `i64` specialization tightened | `INF` | Make explicit fixed-width arithmetic stay native |

## File Targets

| Area | Status | Planned Files |
|------|--------|---------------|
| Binary op dispatch | `INF` | `src/codegen/expressions/operators/core.rs` |
| Arithmetic lowering | `DON` | `src/codegen/expressions/operators/arithmetic.rs` |
| Tagged-int runtime boundary | `INF` | `src/codegen/runtime/tagged_int.rs` |
| Runtime fallback semantics | `DON` | `runtime/src/tagged_int.c` |
| Bench validation | `WIP` | Spot numbers collected locally; `benchmarks/results/benchmark_report.md` not refreshed because clean pre-change baseline is missing |
| Regression tests | `DON` | `tests/integration/operators.rs` and related tests |

## Next Actions

1. `BLK` Recover or regenerate a comparable arithmetic baseline if historical numbers are needed for before/after reporting.
2. `INF` Fix the AOT `-O3` LLVM `opt` pipeline so `inline` is accepted again on the installed LLVM toolchain.
3. `INF` Refresh benchmark reporting once AOT `-O3` is unblocked.
4. `INF` Tighten `i64` specialization in hot paths that still widen to tagged `int`.
