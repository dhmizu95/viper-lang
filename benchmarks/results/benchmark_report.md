# Viper Benchmark Report

**Date:** 2026-03-15 08:05:28  
**Iterations:** 3  
**Max Memory Limit:** 4096MB  
**Max Time Limit:** 300s  

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 112 |
| Passed | 83 |
| Failed/Crashed | 29 |
| Success Rate | 74% |

## Performance (Time in ms)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 215 | 111 | 113 | 111 | 62 | 65 | 58 | 699 |
| 02_prime_sieve | 63 | 64 | 63 | 63 | 65 | 69 | 67 | 89 |
| 03_matrix_mul | 92 | 69 | 71 | 69 | 69 | 69 | 69 | 65 |
| 04_quicksort | CRASH | CRASH | CRASH | CRASH | 67 | 64 | 67 | 64 |
| 05_matrix_mul | CRASH | CRASH | CRASH | CRASH | 63 | 66 | 62 | 63 |
| 06_prime_sieve | CRASH | CRASH | CRASH | CRASH | 65 | 70 | 70 | 66 |
| 07_string_ops | CRASH | BUILD | BUILD | BUILD | 69 | 71 | 68 | 72 |
| 08_int_hotloop | 120 | 61 | 59 | 60 | 65 | 65 | 65 | 219 |
| 09_nbody | CRASH | BUILD | BUILD | BUILD | 67 | 63 | 62 | 63 |
| 10_function_calls | 118 | 59 | 61 | 62 | 66 | 68 | 63 | 267 |
| 11_string_concat_scan | CRASH | BUILD | BUILD | BUILD | 68 | 67 | 67 | 64 |
| 12_bigint_overflow | 183 | 100 | 82 | 61 | 59 | 62 | 63 | 64 |
| 13_factorial | 62 | 63 | 66 | 62 | 64 | 65 | 65 | 62 |
| 14_recursive_list_sum | CRASH | BUILD | BUILD | BUILD | 64 | 65 | 62 | CRASH |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 66082 | 2176 | 2133 | 2133 | 1408 | 1920 | 1706 | 10026 |
| 02_prime_sieve | 66589 | 2176 | 2176 | 2176 | 1408 | 1962 | 1621 | 10112 |
| 03_matrix_mul | 66517 | 2090 | 2176 | 2090 | 1408 | 2005 | 1664 | 10112 |
| 04_quicksort | N/A | N/A | N/A | N/A | 1408 | 2048 | 1664 | 10112 |
| 05_matrix_mul | N/A | N/A | N/A | N/A | 1536 | 2048 | 1749 | 10240 |
| 06_prime_sieve | N/A | N/A | N/A | N/A | 1408 | 2048 | 1706 | 10240 |
| 07_string_ops | N/A | N/A | N/A | N/A | 1408 | 2048 | 1706 | 9984 |
| 08_int_hotloop | 66364 | 2133 | 2176 | 2133 | 1408 | 1920 | 1664 | 10026 |
| 09_nbody | N/A | N/A | N/A | N/A | 1536 | 1962 | 1706 | 10624 |
| 10_function_calls | 66420 | 2133 | 2176 | 2176 | 1408 | 1962 | 1664 | 9984 |
| 11_string_concat_scan | N/A | N/A | N/A | N/A | 1408 | 2048 | 1664 | 10112 |
| 12_bigint_overflow | 206964 | 33450 | 33536 | 33536 | 1408 | 1962 | 6485 | 10026 |
| 13_factorial | 66172 | 2176 | 2133 | 2176 | 1408 | 1920 | 1664 | 10069 |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A | 1408 | 2048 | 1621 | N/A |

## Status

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|:---:|:------:|:------:|:------:|:-:|:----:|:---:|:------:|
| 01_fibonacci | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 02_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 03_matrix_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 04_quicksort | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| 05_matrix_mul | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| 06_prime_sieve | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| 07_string_ops | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 08_int_hotloop | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 09_nbody | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 10_function_calls | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 11_string_concat_scan | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 12_bigint_overflow | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 13_factorial | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 14_recursive_list_sum | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ❌ |

---

## Performance Analysis

### Performance Ratio vs C (Baseline)

| Benchmark | JIT vs C | AOT-O1 vs C | AOT-O2 vs C | AOT-O3 vs C |
|-----------|----------|-------------|-------------|-------------|
| 01_fibonacci | 3.5× | 1.8× | 1.8× | 1.8× |
| 02_prime_sieve | 1.0× | 1.0× | 1.0× | 1.0× |
| 03_matrix_mul | 1.3× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | N/A | N/A | N/A | N/A |
| 05_matrix_mul | N/A | N/A | N/A | N/A |
| 06_prime_sieve | N/A | N/A | N/A | N/A |
| 07_string_ops | N/A | N/A | N/A | N/A |
| 08_int_hotloop | 1.8× | 0.9× | 0.9× | 0.9× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 0.9× | 0.9× | 0.9× |
| 11_string_concat_scan | N/A | N/A | N/A | N/A |
| 12_bigint_overflow | 3.1× | 1.7× | 1.4× | 1.0× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 3.3× | 1.7× | 1.7× | 1.7× |
| 02_prime_sieve | 0.9× | 0.9× | 0.9× | 0.9× |
| 03_matrix_mul | 1.3× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | N/A | N/A | N/A | N/A |
| 05_matrix_mul | N/A | N/A | N/A | N/A |
| 06_prime_sieve | N/A | N/A | N/A | N/A |
| 07_string_ops | N/A | N/A | N/A | N/A |
| 08_int_hotloop | 1.8× | 0.9× | 0.9× | 0.9× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.7× | 0.9× | 0.9× | 0.9× |
| 11_string_concat_scan | N/A | N/A | N/A | N/A |
| 12_bigint_overflow | 3.0× | 1.6× | 1.3× | 1.0× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 3.7× | 1.9× | 1.9× | 1.9× |
| 02_prime_sieve | 0.9× | 1.0× | 0.9× | 0.9× |
| 03_matrix_mul | 1.3× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | N/A | N/A | N/A | N/A |
| 05_matrix_mul | N/A | N/A | N/A | N/A |
| 06_prime_sieve | N/A | N/A | N/A | N/A |
| 07_string_ops | N/A | N/A | N/A | N/A |
| 08_int_hotloop | 1.8× | 0.9× | 0.9× | 0.9× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.9× | 0.9× | 1.0× | 1.0× |
| 11_string_concat_scan | N/A | N/A | N/A | N/A |
| 12_bigint_overflow | 2.9× | 1.6× | 1.3× | 1.0× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 86444 | 60.6× |
| Viper AOT-O1 | 6619 | 4.6× |
| Viper AOT-O2 | 6643 | 4.7× |
| Viper AOT-O3 | 6631 | 4.7× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~60.6× memory overhead (86444KB vs C's ~1426KB)
3. **AOT memory** is ~4.7× C baseline (6631KB vs ~1426KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
