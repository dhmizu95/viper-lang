# Viper Language Benchmarks

Cross-language performance benchmarks comparing Viper against C, Rust, and Go.

## Benchmark Suite

| ID | Name | Description | Type |
|----|------|-------------|------|
| 01 | Fibonacci | Recursive Fibonacci calculation (n=35) | CPU-bound |
| 02 | Prime Sieve | Prime counting using trial division (n=5000) | CPU-bound |

## Directory Structure

```
benchmarks/
├── README.md           # This file
├── runner.sh           # JIT benchmark runner
├── test_aot.sh         # AOT compilation test
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

All languages are compiled with optimization flags for fair comparison:

| Language | Command | Flags |
|----------|---------|-------|
| Viper JIT | `viper run -O3` | Level 3 optimization |
| Viper AOT | `viper build -O2` | Level 2 optimization |
| C | `gcc` | `-O3 -march=native -flto` |
| Rust | `rustc` | `-C opt-level=3 -C lto=fat -C target-cpu=native` |
| Go | `go build` | `-ldflags="-s -w"` |

## Results

See `results/2026-03-10.md` for detailed benchmark results.

### Summary (JIT Mode)

| Benchmark | C | Rust | Go | Viper |
|-----------|---|------|-----|-------|
| Fibonacci | 22ms | 29ms | 44ms | 186ms |
| Prime Sieve | 2ms | 5ms | 5ms | 26ms |

### AOT Compilation

| Benchmark | -O0 | -O2 | -O3 |
|-----------|-----|-----|-----|
| Fibonacci | ✅ | ✅ | ✅ |
| Prime Sieve | ✅ | ✅ | ✅ |

## Adding New Benchmarks

1. Create implementation in all 4 languages under respective directories
2. Ensure identical algorithm and input sizes
3. Add to `runner.sh` benchmark list
4. Document in this README

## Notes

- All benchmarks run with warm-up iterations
- Results are averaged over multiple runs
- JIT compilation time is included for Viper
- AOT mode produces native binaries with comparable performance
