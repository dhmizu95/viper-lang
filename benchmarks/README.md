# Viper Language Benchmarks

Cross-language performance benchmarks comparing Viper against C, Rust, and Go.

## Benchmark Suite

| ID | Name | Description | Type |
|----|------|-------------|------|
| 01 | Fibonacci | Recursive Fibonacci calculation | CPU-bound |
| 02 | QuickSort | Array sorting algorithm | Memory + CPU |

## Directory Structure

```
benchmarks/
├── README.md           # This file
├── runner.sh           # Benchmark runner script
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
# Run all benchmarks
cd benchmarks
./runner.sh

# Run specific benchmark
./runner.sh 01_fibonacci

# Run with custom iterations
./runner.sh --iterations 10
```

### Using Makefile

```bash
# From project root
make bench-all          # Run all benchmarks
make bench-fibonacci    # Run Fibonacci only
make bench-compare      # Run and compare all languages
```

## Compilation Flags

All languages are compiled with optimization flags for fair comparison:

| Language | Command | Flags |
|----------|---------|-------|
| Viper | `viper run -O3` | Level 3 optimization |
| C | `gcc` | `-O3 -march=native -flto` |
| Rust | `rustc` | `-C opt-level=3 -C lto=fat -C target-cpu=native` |
| Go | `go build` | `-ldflags="-s -w"` |

## Results Format

Results are stored in `results/YYYY-MM-DD.md` with:

- Execution time (milliseconds)
- Relative performance (vs fastest)
- Memory usage (when available)

## Adding New Benchmarks

1. Create implementation in all 4 languages under respective directories
2. Ensure identical algorithm and input sizes
3. Add to `runner.sh` benchmark list
4. Document in this README

## Notes

- All benchmarks run with warm-up iterations
- Results are averaged over multiple runs
- JIT compilation time is excluded for Viper

## AOT Compilation Status

⚠️ **Known Issue**: AOT compilation has a linking issue with duplicate runtime symbols (`vp_print_str` is defined in both `runtime.o` and `libviper.a`).

### Workaround

Until this is fixed, use JIT mode for running benchmarks:

```bash
# JIT mode (works correctly)
viper run -O3 benchmarks/viper/01_fibonacci.vp

# AOT mode (has linking issue)
viper build -O0 benchmarks/viper/01_fibonacci.vp  # Will fail at linking stage
```

### AOT Test Script

To test AOT compilation status:

```bash
cd benchmarks
./test_aot.sh
```

