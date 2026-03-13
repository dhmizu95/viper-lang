# Viper Language Benchmarks

Cross-language performance benchmarks for Viper, with the expanded suite integrated through `safe_runner.sh`.

## Benchmark Suite

| ID | Name | Description | Type |
|----|------|-------------|------|
| 01 | Fibonacci | Recursive Fibonacci (n=35) | CPU-bound |
| 02 | Prime Sieve | Prime counting, trial division (n=5000) | CPU-bound |
| 03 | Matrix Mul | Scalar matrix-style arithmetic (30x30) | CPU + Memory |
| 04 | QuickSort | Iterative quicksort (100 elements) | CPU + Memory |
| 05 | Matrix Mul Array | Real dense matrix multiplication (50x50) | CPU + Memory |
| 06 | Prime Sieve Array | Sieve of Eratosthenes (n=10000) | Memory + CPU |
| 07 | String Operations | String concatenation and character scanning | Memory + CPU |
| 08 | Int Hot Loop | Dynamic `int` arithmetic hot loop | CPU-bound |
| 09 | i64 Hot Loop | Fixed-width `i64` arithmetic hot loop | CPU-bound |
| 10 | Function Calls | Tiny helper call inside a hot loop | CPU-bound |
| 11 | String Concat Scan | `str(i)` concatenation and text scan | CPU + Memory |
| 12 | BigInt Overflow | Large-integer arithmetic on values beyond Viper small-int range | CPU + Memory |

## Directory Structure

- `viper/`, `c/`, `rust/`, `go/`, `python/`: per-language implementations
- `safe_runner.sh`: crash-protected runner and source of truth for the expanded suite

## Running Benchmarks

Prerequisites: built Viper compiler, GCC/Clang, Rust, Go, and optionally `python3` or `python`.

```bash
cd benchmarks
./safe_runner.sh all
./safe_runner.sh 08_int_hotloop
```

From the repo root:

```bash
make bench-safe
```

## Compilation Flags

| Language | Command | Flags |
|----------|---------|-------|
| Viper JIT | `viper run -O3` | JIT execution with optimization |
| Viper AOT | `viper build -O1/-O2/-O3` | Ahead-of-time compiled binary |
| C | `gcc` | `-O3 -march=native -flto` |
| Rust | `rustc` | `-C opt-level=3 -C lto=fat -C target-cpu=native` |
| Go | `go build` | `-ldflags="-s -w"` |
| Python | `python3` | direct interpreter execution via `safe_runner.sh` only |

## Results

Use `results/benchmark_report.md` as the source of truth for current numbers. This README intentionally avoids hard-coded results that can drift from the generated report.

## Adding New Benchmarks

1. Create matching implementations in `viper/`, `c/`, `rust/`, `go/`, and `python/`.
2. Keep algorithms, input sizes, and printed checksums identical.
3. Add the benchmark ID to `safe_runner.sh`.
4. Update this README.

## Notes

- JIT compilation time is included for Viper JIT runs.
- Python is included as an interpreted-language baseline in safe mode only.
- The suite now includes targeted arithmetic, call-overhead, string, and overflow cases for optimization work.
