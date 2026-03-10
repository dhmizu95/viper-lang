# Viper Language Benchmarks

Cross-language performance benchmarks comparing Viper against C, Rust, and Go.

## Benchmark Suite

| ID | Name | Description | Type |
|----|------|-------------|------|
| 01 | Fibonacci | Recursive Fibonacci (n=35) | CPU-bound |
| 02 | Prime Sieve | Prime counting, trial division (n=5000) | CPU-bound |
| 03 | Matrix Mul | Matrix multiplication (30x30, scalar) | CPU + Memory |
| 04 | QuickSort | Iterative quicksort (100 elements) | CPU + Memory |
| 05 | Matrix Mul Array | Matrix multiplication with arrays (50x50) | CPU + Memory |
| 06 | Prime Sieve Array | Sieve of Eratosthenes (n=10000) | Memory + CPU |
| 07 | String Operations | String manipulation & parsing | Memory + CPU |

## Directory Structure

```
benchmarks/
├── README.md           # This file
├── runner.sh           # JIT benchmark runner
├── test_aot.sh         # AOT compilation test
├── compare_aot.sh      # AOT cross-language comparison
├── results/            # Historical results
├── viper/              # Viper implementations
├── c/                  # C implementations
├── rust/               # Rust implementations
└── go/                 # Go implementations
```

## Running Benchmarks

### Prerequisites

- **Viper**: Built compiler (`cargo build --release`)
- **C**: GCC or Clang
- **Rust**: Rust compiler (`rustc`)
- **Go**: Go compiler (`go`)

### Quick Start

```bash
# Run all JIT benchmarks
cd benchmarks
./runner.sh

# Run specific benchmark
./runner.sh 01_fibonacci

# Test AOT compilation
./test_aot.sh

# Compare AOT performance across languages
./compare_aot.sh
```

### Using Makefile

```bash
# From project root
make bench-all          # Run all JIT benchmarks
make bench-fibonacci    # Run Fibonacci only
make bench-compare      # Run with 10 iterations
make bench-aot-test     # Test AOT compilation
```

## Compilation Flags

| Language | Command | Flags |
|----------|---------|-------|
| Viper JIT | `viper run -O3` | Level 3 optimization |
| Viper AOT | `viper build -O2` | Level 2 optimization |
| C | `gcc` | `-O3 -march=native -flto` |
| Rust | `rustc` | `-C opt-level=3 -C lto=fat -C target-cpu=native` |
| Go | `go build` | `-ldflags="-s -w"` |

## Results Summary

### JIT Mode Performance

| Benchmark | C | Rust | Go | Viper JIT |
|-----------|---|------|-----|-----------|
| Fibonacci | 14ms | 24ms | 44ms | 186ms |
| Prime Sieve | 1ms | 1ms | 2ms | 26ms |
| Matrix Mul | 2ms | 2ms | 2ms | 1ms |

### AOT Mode Performance

| Benchmark | C -O3 | Rust -O3 | Go | Viper AOT -O2 |
|-----------|-------|----------|-----|---------------|
| Fibonacci | 14ms | 24ms | 44ms | 111ms |
| Prime Sieve | 1ms | 1ms | 2ms | 4ms |
| Matrix Mul | 1ms | 1ms | 2ms | 8ms |

### JIT vs AOT for Viper

| Benchmark | JIT | AOT | Speedup |
|-----------|-----|-----|---------|
| Fibonacci | 186ms | 111ms | 1.67x |
| Prime Sieve | 26ms | 4ms | 6.5x |
| Matrix Mul | 1ms | 8ms | JIT faster* |

*Matrix Mul JIT appears faster due to measurement overhead for very fast operations

## Adding New Benchmarks

1. Create implementation in all 4 languages (viper/, c/, rust/, go/)
2. Ensure identical algorithm and input sizes
3. Add benchmark ID to `runner.sh` and `compare_aot.sh`
4. Document in this README

## Notes

- All benchmarks produce identical results across languages
- Results are averaged over multiple runs
- JIT compilation time is included for Viper
- AOT mode recommended for production use
