# Native Int Arithmetic Progress Tracker

**Date Started:** 2026-03-13
**Related Plan:** `plans/native_int_arithmetic_optimization_plan.md`

## Status Legend

- `INF` = identified / queued
- `WIP` = in progress
- `BLK` = blocked
- `DON` = done

## Overall Status

- Overall: `INF`
- Language priority: `DON` Python syntax compatibility is the top constraint
- Primary bottleneck confirmed: tagged-int runtime calls dominate ordinary `int` arithmetic hot paths
- Benchmark coverage added for arithmetic, `i64`, calls, strings, and large integers

## Milestones

| Milestone | Status | Notes |
|----------|--------|-------|
| Small-int fast-path design documented | `DON` | Plan created and scoped |
| Arithmetic benchmark coverage expanded | `DON` | Added benchmarks 08-12 |
| Main benchmark runner consolidated | `DON` | `benchmark_runner.sh` is now the benchmark entrypoint |
| Inline `Add/Sub/Mul` fast path implemented | `INF` | First codegen target |
| Inline comparisons fast path implemented | `INF` | Second codegen target |
| Inline division/modulo fast path implemented | `INF` | Needs semantic care |
| `i64` native path audit completed | `INF` | Prevent widening back to `Type::Int` |
| AOT pipeline retuned after IR changes | `INF` | Re-benchmark O1/O2/O3 |
| Regression tests added | `INF` | Fast path plus overflow path |

## File Targets

| Area | Status | Files |
|------|--------|-------|
| Tagged-int dispatch | `INF` | `src/codegen/expressions/operators/core.rs` |
| Arithmetic lowering | `INF` | `src/codegen/expressions/operators/arithmetic.rs` |
| Runtime fallback boundary | `INF` | `src/codegen/runtime/tagged_int.rs` |
| Tagged-int runtime semantics | `INF` | `runtime/src/tagged_int.c` |
| Type inference and native `i64` preservation | `INF` | `src/codegen/expressions/core.rs`, `src/codegen/functions.rs`, `src/semantic/type_checker/exprs.rs` |
| Benchmark reporting | `DON` | `benchmarks/benchmark_runner.sh` |

## Next Steps

1. `INF` Implement native small-int fast path for `Add`, `Sub`, and `Mul`.
2. `INF` Add direct comparison lowering for small-int tagged values.
3. `INF` Add correctness tests for overflow and BigInt fallback.
4. `INF` Re-benchmark `02`, `05`, `08`, `09`, `10`, and `12`.
