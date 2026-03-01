# New Benchmarks Summary (16-96)

This document summarizes the new benchmarks added to the Viper benchmark suite, extending from 15 to 22 total benchmarks.

## Overview

The new benchmarks cover additional categories from the 100 Mathematical Benchmark Problems list:
- **Big Integer** (16-17): Tests arbitrary precision arithmetic
- **Floating Point** (26): Monte Carlo simulation
- **Linear Algebra** (41): Large matrix operations
- **Graph Theory** (66): Graph traversal algorithms
- **Signal Processing** (86): Fast Fourier Transform
- **Simulation** (96): Cellular automata

## Benchmark Details

### 16. Factorial

**Problem:** Compute factorial(1,000,000)

**Category:** Big Integer

**What it tests:**
- Arbitrary precision arithmetic
- Memory management for large numbers
- Multiplication performance

**Scale differences:**
- C/Go/Rust: 1,000,000
- Viper: 20 (limited by i64)

**Viper limitations identified:**
- No BigInt support in standard library
- Need to implement arbitrary precision integers

**Files:**
- `16_factorial/factorial.c` (requires GMP)
- `16_factorial/factorial.go` (uses big.Int)
- `16_factorial/factorial.rs` (uses num-bigint)
- `16_factorial/factorial.vp` (i64 limited)

---

### 17. Fibonacci (Big)

**Problem:** Compute fibonacci(1,000,000)

**Category:** Big Integer

**What it tests:**
- Arbitrary precision addition
- Iterative computation with large numbers
- Memory allocation patterns

**Scale differences:**
- C/Go/Rust: 1,000,000
- Viper: 50 (limited by i64)

**Viper limitations identified:**
- Same as Factorial - needs BigInt

**Files:**
- `17_fibonacci_big/fibonacci.c` (requires GMP)
- `17_fibonacci_big/fibonacci.go` (uses big.Int)
- `17_fibonacci_big/fibonacci.rs` (uses num-bigint)
- `17_fibonacci_big/fibonacci.vp` (i64 limited)

---

### 26. Monte Carlo Pi

**Problem:** Estimate π using 1 billion random samples

**Category:** Floating Point

**What it tests:**
- Floating-point arithmetic performance
- Random number generation
- Branch prediction

**Scale differences:**
- C/Go/Rust: 1,000,000,000 samples
- Viper: 1,000,000 samples

**Viper limitations identified:**
- No built-in random number generator
- Must implement LCG manually

**Files:**
- `26_monte_carlo_pi/monte_carlo.c`
- `26_monte_carlo_pi/monte_carlo.go`
- `26_monte_carlo_pi/monte_carlo.rs` (uses rand crate)
- `26_monte_carlo_pi/monte_carlo.vp`

---

### 41. Matrix Multiplication 1000×1000

**Problem:** Multiply two 1000×1000 matrices

**Category:** Linear Algebra

**What it tests:**
- Memory bandwidth
- Cache efficiency
- Floating-point throughput
- Vectorization potential

**Scale differences:**
- C/Go/Rust: 1000×1000
- Viper: 200×200

**Viper limitations identified:**
- 2D array performance
- Nested loop optimization

**Files:**
- `41_matrix_1000/matrix_mul.c`
- `41_matrix_1000/matrix_mul.go`
- `41_matrix_1000/matrix_mul.rs`
- `41_matrix_1000/matrix_mul.vp`

---

### 66. BFS (Breadth-First Search)

**Problem:** Traverse graph with 10M nodes using BFS

**Category:** Graph Theory

**What it tests:**
- Queue data structure performance
- Memory access patterns
- Graph representation efficiency

**Scale differences:**
- C/Go/Rust: 10,000,000 nodes
- Viper: 10,000 nodes

**Viper limitations identified:**
- List concatenation performance
- Memory for large adjacency lists

**Files:**
- `66_bfs/bfs.c`
- `66_bfs/bfs.go`
- `66_bfs/bfs.rs`
- `66_bfs/bfs.vp`

---

### 86. FFT (Fast Fourier Transform)

**Problem:** Compute FFT on 1M samples

**Category:** Signal Processing

**What it tests:**
- Complex number arithmetic
- Recursive/iterative algorithms
- Trigonometric functions
- Array shuffling (bit-reversal)

**Scale differences:**
- C/Go/Rust: 1,048,576 samples
- Viper: 256 samples

**Viper limitations identified:**
- No sin/cos functions
- No complex number type
- Math library needed

**Files:**
- `86_fft/fft.c`
- `86_fft/fft.go`
- `86_fft/fft.rs`
- `86_fft/fft.vp` (simplified)

---

### 96. Cellular Automata (Game of Life)

**Problem:** Simulate Conway's Game of Life on 4096×4096 grid

**Category:** Simulation

**What it tests:**
- 2D array operations
- Neighbor counting algorithms
- Parallel computation patterns
- Memory bandwidth

**Scale differences:**
- C/Go/Rust: 4096×4096
- Viper: 100×100

**Viper limitations identified:**
- Nested loop performance
- 2D array access patterns

**Files:**
- `96_cellular_automata/cellular_automata.c`
- `96_cellular_automata/cellular_automata.go`
- `96_cellular_automata/cellular_automata.rs` (uses rand crate)
- `96_cellular_automata/cellular_automata.vp`

---

## Building New Benchmarks

### Rust Benchmarks (Workspace)

The new Rust benchmarks use a Cargo workspace:

```bash
cd benchmark
cargo build --release
```

Individual benchmarks can be run with:
```bash
cargo run --release -p factorial_bench
```

### C Benchmarks

Some C benchmarks require external libraries:

```bash
# Factorial and Fibonacci require GMP
gcc -O3 -o benchmark_c factorial.c -lgmp
```

### Go Benchmarks

```bash
cd 16_factorial
go build -o benchmark_go factorial.go
```

### Viper Benchmarks

```bash
viper build 16_factorial/factorial.vp -o benchmark_vp
```

## Priority Features for Viper

Based on these benchmarks, here are the priority features to implement:

1. **BigInt type** - Critical for benchmarks 16, 17
2. **Math library** - sin, cos, sqrt, etc. for benchmarks 26, 86
3. **Random number generation** - Built-in RNG for benchmarks 26, 96
4. **Complex number type** - For FFT benchmark
5. **2D array optimization** - For matrix and cellular automata
6. **Standard library** - Common algorithms and data structures

## Next Steps

1. Build and run all benchmarks
2. Document performance comparisons
3. Identify optimization opportunities
4. Implement missing Viper features
5. Add more benchmarks from the 100-problem list

## References

- [100 Mathematical Benchmark Problems](../BENCHMARK.md)
- [Original Benchmark README](README.md)
- [Viper Language Documentation](../README.md)
