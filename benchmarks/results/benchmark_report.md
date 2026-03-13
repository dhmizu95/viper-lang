# Viper Benchmark Report

**Date:** 2026-03-13 09:38:04  
**Iterations:** 3  
**Max Memory Limit:** 4096MB  
**Max Time Limit:** 300s  

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 96 |
| Passed | 89 |
| Failed/Crashed | 7 |
| Success Rate | 92% |

## Performance (Time in ms)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 245 | 240 | 199 | 262 | 90 | 109 | 124 | 1461 |
| 02_prime_sieve | 127 | 101 | 119 | 144 | 85 | 92 | 77 | 137 |
| 03_matrix_mul | 163 | 73 | 71 | 70 | 73 | 80 | 76 | 89 |
| 04_quicksort | 122 | 103 | 74 | 81 | 75 | 79 | 68 | 116 |
| 05_matrix_mul | 303 | CRASH | CRASH | CRASH | 73 | 81 | 73 | 126 |
| 06_prime_sieve | 144 | 79 | 70 | 73 | 81 | 80 | 71 | 85 |
| 07_string_ops | 128 | 72 | 69 | 75 | 73 | 71 | 76 | 89 |
| 08_int_hotloop | 174 | 1245 | 2499 | 1136 | 75 | 72 | 75 | 461 |
| 09_nbody | CRASH | BUILD | BUILD | BUILD | 76 | 73 | 72 | 70 |
| 10_function_calls | 155 | 1371 | 790 | 833 | 73 | 69 | 95 | 637 |
| 11_string_concat_scan | 125 | 78 | 74 | 76 | 72 | 73 | 70 | 67 |
| 12_bigint_overflow | 335 | 158 | 204 | 140 | 69 | 80 | 110 | 164 |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 66766 | 3200 | 3200 | 3200 | 3114 | 3200 | 3200 | 9941 |
| 02_prime_sieve | 67304 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10069 |
| 03_matrix_mul | 67548 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10069 |
| 04_quicksort | 68241 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10197 |
| 05_matrix_mul | 68385 | N/A | N/A | N/A | 3200 | 3200 | 3200 | 10197 |
| 06_prime_sieve | 67524 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10154 |
| 07_string_ops | 67400 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 9984 |
| 08_int_hotloop | 67298 | 55936 | 56064 | 55893 | 3200 | 3200 | 3200 | 9984 |
| 09_nbody | N/A | N/A | N/A | N/A | 3200 | 3200 | 3200 | 10624 |
| 10_function_calls | 67180 | 23850 | 23936 | 23893 | 3200 | 3200 | 3200 | 9984 |
| 11_string_concat_scan | 67418 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10112 |
| 12_bigint_overflow | 207601 | 26922 | 26922 | 26922 | 3200 | 3200 | 6570 | 9941 |

## Status

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|:---:|:------:|:------:|:------:|:-:|:----:|:---:|:------:|
| 01_fibonacci | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 02_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 03_matrix_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 04_quicksort | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 05_matrix_mul | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| 06_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 07_string_ops | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 08_int_hotloop | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 09_nbody | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 10_function_calls | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 11_string_concat_scan | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 12_bigint_overflow | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Performance Analysis

### Performance Ratio vs C (Baseline)

| Benchmark | JIT vs C | AOT-O1 vs C | AOT-O2 vs C | AOT-O3 vs C |
|-----------|----------|-------------|-------------|-------------|
| 01_fibonacci | 2.7× | 2.7× | 2.2× | 2.9× |
| 02_prime_sieve | 1.5× | 1.2× | 1.4× | 1.7× |
| 03_matrix_mul | 2.2× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | 1.6× | 1.4× | 1.0× | 1.1× |
| 05_matrix_mul | 4.2× | N/A | N/A | N/A |
| 06_prime_sieve | 1.8× | 1.0× | 0.9× | 0.9× |
| 07_string_ops | 1.8× | 1.0× | 0.9× | 1.0× |
| 08_int_hotloop | 2.3× | 16.6× | 33.3× | 15.1× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 2.1× | 18.8× | 10.8× | 11.4× |
| 11_string_concat_scan | 1.7× | 1.1× | 1.0× | 1.1× |
| 12_bigint_overflow | 4.9× | 2.3× | 3.0× | 2.0× |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 2.2× | 2.2× | 1.8× | 2.4× |
| 02_prime_sieve | 1.4× | 1.1× | 1.3× | 1.6× |
| 03_matrix_mul | 2.0× | 0.9× | 0.9× | 0.9× |
| 04_quicksort | 1.5× | 1.3× | 0.9× | 1.0× |
| 05_matrix_mul | 3.7× | N/A | N/A | N/A |
| 06_prime_sieve | 1.8× | 1.0× | 0.9× | 0.9× |
| 07_string_ops | 1.8× | 1.0× | 1.0× | 1.1× |
| 08_int_hotloop | 2.4× | 17.3× | 34.7× | 15.8× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 2.2× | 19.9× | 11.4× | 12.1× |
| 11_string_concat_scan | 1.7× | 1.1× | 1.0× | 1.0× |
| 12_bigint_overflow | 4.2× | 2.0× | 2.5× | 1.8× |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 2.0× | 1.9× | 1.6× | 2.1× |
| 02_prime_sieve | 1.6× | 1.3× | 1.5× | 1.9× |
| 03_matrix_mul | 2.1× | 1.0× | 0.9× | 0.9× |
| 04_quicksort | 1.8× | 1.5× | 1.1× | 1.2× |
| 05_matrix_mul | 4.2× | N/A | N/A | N/A |
| 06_prime_sieve | 2.0× | 1.1× | 1.0× | 1.0× |
| 07_string_ops | 1.7× | 0.9× | 0.9× | 1.0× |
| 08_int_hotloop | 2.3× | 16.6× | 33.3× | 15.1× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.6× | 14.4× | 8.3× | 8.8× |
| 11_string_concat_scan | 1.8× | 1.1× | 1.1× | 1.1× |
| 12_bigint_overflow | 3.0× | 1.4× | 1.9× | 1.3× |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 80242 | 25.1× |
| Viper AOT-O1 | 12910 | 4.0× |
| Viper AOT-O2 | 12932 | 4.1× |
| Viper AOT-O3 | 12910 | 4.0× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~25.1× memory overhead (80242KB vs C's ~3192KB)
3. **AOT memory** is ~4.0× C baseline (12917KB vs ~3192KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
