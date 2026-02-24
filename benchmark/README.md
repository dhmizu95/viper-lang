# Viper Language Benchmark Suite

A comprehensive benchmark suite comparing **Viper**, **C**, **Go**, and **Rust** across 15 different computational problems.

## Overview

This benchmark suite tests various aspects of programming language performance:

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
...
├── build_all.sh         # Build all benchmarks
├── run_all.sh           # Run all benchmarks
├── build_c.sh           # Build C only
├── build_go.sh          # Build Go only
├── build_rust.sh        # Build Rust only
├── build_viper.sh       # Build Viper only
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

1. **No math functions** - `sqrt()`, `ln()`, `abs()` need builtins
2. **No modulo for floats** - Limited numeric operations
3. **No array slicing with step** - Limited list operations
4. **No dictionaries/maps** - Must use alternative data structures
5. **No string formatting** - Concatenation only
6. **No struct types** - Using parallel arrays instead
7. **No closures** - Limited functional programming
8. **No standard library** - Missing common functions

## Contributing

When adding new benchmarks:

1. Implement in all 4 languages
2. Keep algorithmic complexity similar
3. Include verification output
4. Document any scale differences
5. Add to this README

## License

Part of the Viper Language project. See main repository LICENSE.
