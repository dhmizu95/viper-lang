# Arithmetic Performance Progress Tracker

**Date Started:** 2026-03-13
**Plan Reference:** `plans/arithmetic_performance_plan.md`

## Status Legend

- `INF` = identified / queued
- `WIP` = in progress
- `BLK` = blocked
- `DON` = done

## Current Overall Status

- Overall: `INF`
- Primary target: reduce tagged-int overhead in arithmetic-heavy code
- Current bottleneck confirmed in codegen and runtime dispatch

## Milestones

| Milestone | Status | Notes |
|----------|--------|-------|
| Baseline arithmetic benchmarks captured | `INF` | Re-run and save fresh numbers before changes |
| Small-int inline fast path designed | `INF` | Lower common tagged-int ops directly in LLVM IR |
| Runtime fallback preserved | `INF` | Keep BigInt and overflow handling in existing helpers |
| Regression tests added | `INF` | Cover fast path and overflow path |
| AOT benchmark validation completed | `INF` | Compare O1/O2/O3 after implementation |
| `i64` specialization tightened | `INF` | Make explicit fixed-width arithmetic stay native |

## File Targets

| Area | Status | Planned Files |
|------|--------|---------------|
| Binary op dispatch | `INF` | `src/codegen/expressions/operators/core.rs` |
| Arithmetic lowering | `INF` | `src/codegen/expressions/operators/arithmetic.rs` |
| Tagged-int runtime boundary | `INF` | `src/codegen/runtime/tagged_int.rs` |
| Runtime fallback semantics | `INF` | `runtime/src/tagged_int.c` |
| Bench validation | `INF` | `benchmarks/results/benchmark_report.md` |
| Regression tests | `INF` | `tests/integration/operators.rs` and related tests |

## Next Actions

1. `INF` Re-run arithmetic-focused benchmarks and capture a clean baseline.
2. `INF` Implement inline small-int codegen for `add`, `sub`, and `mul` first.
3. `INF` Extend the same structure to comparisons and bitwise operators.
4. `INF` Add overflow and BigInt fallback tests.
5. `INF` Re-benchmark and update this tracker.
