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

## Milestones

| Milestone | Status | Notes |
|----------|--------|-------|
| Baseline arithmetic benchmarks captured | `INF` | Re-run and save fresh numbers before changes |
| Small-int inline fast path designed | `WIP` | `add`/`sub`/`mul` lowering moved into implementation |
| Runtime fallback preserved | `WIP` | Slow path still targets existing tagged-int helpers |
| Regression tests added | `WIP` | Adding focused overflow and negative-value coverage for arithmetic |
| AOT benchmark validation completed | `INF` | Compare O1/O2/O3 after implementation |
| `i64` specialization tightened | `INF` | Make explicit fixed-width arithmetic stay native |

## File Targets

| Area | Status | Planned Files |
|------|--------|---------------|
| Binary op dispatch | `INF` | `src/codegen/expressions/operators/core.rs` |
| Arithmetic lowering | `WIP` | `src/codegen/expressions/operators/arithmetic.rs` |
| Tagged-int runtime boundary | `INF` | `src/codegen/runtime/tagged_int.rs` |
| Runtime fallback semantics | `INF` | `runtime/src/tagged_int.c` |
| Bench validation | `INF` | `benchmarks/results/benchmark_report.md` |
| Regression tests | `WIP` | `tests/integration/operators.rs` and related tests |

## Next Actions

1. `INF` Re-run arithmetic-focused benchmarks and capture a clean baseline.
2. `WIP` Implement inline small-int codegen for `add`, `sub`, and `mul` first.
3. `INF` Extend the same structure to comparisons and bitwise operators.
4. `WIP` Add overflow and BigInt fallback tests.
5. `INF` Re-benchmark and update this tracker.
