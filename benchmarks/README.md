# Viper Language Benchmark Suite

A comprehensive benchmark suite comparing **Viper**, **C**, **Go**, and **Rust** across 22 different computational problems.

## Overview

This benchmark suite tests various aspects of programming language performance:

### Original Benchmarks (01-15)

| # | Benchmark | Category | What it Tests |
|---|-----------|----------|---------------|
| 01 | Prime Sieve | Integer Arithmetic | Array operations, basic arithmetic, memory access |
| 02 | Fibonacci | Integer Arithmetic | Loop performance, variable assignment |
| 03 | Matrix Multiply | Linear Algebra | Nested loops, array access, FLOPs |
| 04 | QuickSort | Discrete Math | Recursion, array manipulation, comparisons |
| 05 | Mandelbrot | Floating Point | Complex arithmetic, nested loops |
| 06 | Ray Tracer | Simulation | 3D math, vector operations, recursion |
| 07 | N-Body | Simulation | Physics calculations, nested loops |
| 08 | Binary Trees | Data Structures | Tree traversal, recursion, memory |
| 09 | Fannkuch | Permutations | Array manipulation, permutations |
| 10 | Spectral Norm | Linear Algebra | Matrix-vector operations, iteration |
| 11 | K-Nucleotide | Bioinformatics | String manipulation, hash tables |
| 12 | Reverse Complement | Bioinformatics | String manipulation, character mapping |
| 13 | Regex DNA | Pattern Matching | Pattern matching, string scanning |
| 14 | Champernowne | Number Theory | String conversion, concatenation |
| 15 | Euler Sum | Numerical Analysis | Floating-point summation, precision |

### New Benchmarks (16-96)

| # | Benchmark | Category | What it Tests |
|---|-----------|----------|---------------|
| 16 | Factorial | Big Integer | Arbitrary precision arithmetic, memory |
| 17 | Fibonacci (Big) | Big Integer | Arbitrary precision, iterative computation |
| 26 | Monte Carlo Pi | Floating Point | FP performance, random numbers |
| 41 | Matrix 1000×1000 | Linear Algebra | Memory bandwidth, vectorization |
| 66 | BFS | Graph Theory | Data structures, memory access |
| 86 | FFT | Signal Processing | Recursion, FP math, arrays |
| 96 | Cellular Automata | Simulation | Array operations, parallel patterns |

## Directory Structure

```
benchmark/
├── 01_prime_sieve/
│   ├── sieve.c          # C implementation
│   ├── sieve.go         # Go implementation
│   ├── sieve.rs         # Rust implementation
│   └── sieve.vp         # Viper implementation
├── 02_fibonacci/
│   └── ...
├── 16_factorial/
│   ├── factorial.c      # C (requires GMP)
│   ├── factorial.go     # Go (big.Int)
│   ├── factorial.rs     # Rust (num-bigint)
│   └── factorial.vp     # Viper (i64 limited)
├── 17_fibonacci_big/
│   └── ... (same structure)
├── 26_monte_carlo_pi/
│   └── ...
├── 41_matrix_1000/
│   └── ...
├── 66_bfs/
│   └── ...
├── 86_fft/
│   └── ...
├── 96_cellular_automata/
│   └── ...
├── Cargo.toml           # Rust workspace for new benchmarks
├── build_all.sh         # Build all benchmarks
├── run_all.sh           # Run all benchmarks
└── README.md            # This file
```

## Prerequisites

### Required Compilers

- **GCC** (for C benchmarks)
- **Go 1.20+** (for Go benchmarks)
- **Rust 1.70+** (for Rust benchmarks)
- **Viper** (for Viper benchmarks) - Build from this repo

### Building Viper Compiler

```bash
cd /home/stl/viper-lang
cargo build --release
# Optional: Install to PATH
cp target/release/viper /usr/local/bin/
```

## Usage

### Build All Benchmarks

```bash
cd benchmark
./build_all.sh
```

### Build Individual Languages

```bash
./build_c.sh      # Build C benchmarks
./build_go.sh     # Build Go benchmarks
./build_rust.sh   # Build Rust benchmarks
./build_viper.sh  # Build Viper benchmarks
```

### Run All Benchmarks

```bash
./run_all.sh
```

This will run each benchmark 3 times and calculate average execution time.

### Run Individual Benchmark

```bash
cd 01_prime_sieve
./benchmark_c     # Run C version
./benchmark_go    # Run Go version
./benchmark_rs    # Run Rust version
./benchmark_vp    # Run Viper version
```

## Important Notes

### Scale Differences

**Viper benchmarks use reduced problem sizes** due to current language limitations:

| Benchmark | C/Go/Rust | Viper | Reason |
|-----------|-----------|-------|--------|
| Prime Sieve | 10M | 1M | Array size limits |
| Matrix Multiply | 512×512 | 100×100 | Memory/performance |
| Fibonacci | 10M iter | 1M iter | Loop performance |
| Mandelbrot | 1000×1000 | 200×200 | Nested loop perf |
| N-Body | 500 bodies | 50 bodies | O(n²) complexity |
| QuickSort | 100k | 10k | Recursion depth |
| Factorial | 1,000,000 | 20 | Needs BigInt support |
| Fibonacci (Big) | 1,000,000 | 50 | Needs BigInt support |
| Monte Carlo Pi | 1B samples | 1M samples | Loop performance |
| Matrix 1000×1000 | 1000×1000 | 200×200 | Memory/performance |
| BFS | 10M nodes | 10K nodes | Memory limits |
| FFT | 1M samples | 256 samples | Needs math functions |
| Cellular Automata | 4096×4096 | 100×100 | Nested loop perf |

When comparing results, consider these scale differences. **The goal is to identify performance characteristics and missing features, not raw speed comparison.**

## Expected Output

Each benchmark outputs:
- Problem size/configuration
- Verification data (to ensure correctness)
- Execution time

Example output:
```
=== 01_prime_sieve ===
  Running C...
  C: 0.1234s (average of 3 runs)
  Running Go...
  Go: 0.1456s (average of 3 runs)
  Running Rust...
  Rust: 0.1123s (average of 3 runs)
  Running Viper...
  Viper: 1.2345s (average of 3 runs)
```

## Analysis Goals

This benchmark suite helps identify:

1. **Performance Gaps** - Where is Viper slower and by how much?
2. **Missing Features** - What language features are needed?
3. **Optimization Opportunities** - Where can the compiler improve?
4. **Memory Patterns** - How does ARC compare to GC/manual memory?
5. **Numeric Performance** - How fast is floating-point math?

## Common Viper Limitations Found

Based on benchmark implementation:

1. **No BigInt support** - Limited to i64/f64 ranges
2. **No math functions** - `sqrt()`, `ln()`, `abs()`, `sin()`, `cos()` need builtins
3. **No modulo for floats** - Limited numeric operations
4. **No array slicing with step** - Limited list operations
5. **No dictionaries/maps** - Must use alternative data structures
6. **No string formatting** - Concatenation only
7. **No struct types** - Using parallel arrays instead
8. **No closures** - Limited functional programming
9. **No standard library** - Missing common functions
10. **No random number generation** - Must implement manually

## Contributing

When adding new benchmarks:

1. Implement in all 4 languages
2. Keep algorithmic complexity similar
3. Include verification output
4. Document any scale differences
5. Add to this README

## License

Part of the Viper Language project. See main repository LICENSE.
