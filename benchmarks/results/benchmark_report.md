# Viper Benchmark Report

**Date:** 2026-03-13 08:14:44  
**Iterations:** 3  
**Max Memory Limit:** 4096MB  
**Max Time Limit:** 300s  

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 96 |
| Passed | 80 |
| Failed/Crashed | 16 |
| Success Rate | 83% |

## Performance (Time in ms)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 163 | 113 | 111 | 119 | 61 | 62 | 77 | 700 |
| 02_prime_sieve | 60 | 63 | 60 | 468 | 63 | 62 | 64 | 62 |
| 03_matrix_mul | 63 | 60 | 60 | 62 | 63 | 64 | 62 | 63 |
| 04_quicksort | 62 | 68 | 65 | 64 | 67 | 70 | 63 | 82 |
| 05_matrix_mul | 97 | CRASH | CRASH | CRASH | 60 | 66 | 65 | 64 |
| 06_prime_sieve | 85 | 64 | 65 | 61 | 63 | 67 | 67 | 63 |
| 07_string_ops | CRASH | BUILD | BUILD | BUILD | 67 | 65 | 60 | 60 |
| 08_int_hotloop | 112 | 579 | 598 | 600 | 63 | 65 | 67 | 221 |
| 09_i64_hotloop | CRASH | BUILD | BUILD | BUILD | 66 | 65 | 65 | 217 |
| 10_function_calls | 112 | 458 | 423 | 616 | 63 | 63 | 64 | 269 |
| 11_string_concat_scan | CRASH | BUILD | BUILD | BUILD | 66 | 61 | 65 | 64 |
| 12_bigint_overflow | 166 | 98 | 115 | 117 | 63 | 62 | BUILD | 62 |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 66785 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 9984 |
| 02_prime_sieve | 67221 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10112 |
| 03_matrix_mul | 67658 | 3200 | 3200 | 3200 | 3200 | 3200 | 3157 | 10069 |
| 04_quicksort | 68184 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10069 |
| 05_matrix_mul | 68140 | N/A | N/A | N/A | 3200 | 3200 | 3200 | 10240 |
| 06_prime_sieve | 67429 | 3200 | 3200 | 3200 | 3200 | 3200 | 3200 | 10154 |
| 07_string_ops | N/A | N/A | N/A | N/A | 3200 | 3200 | 3200 | 10026 |
| 08_int_hotloop | 67177 | 55978 | 56021 | 56021 | 3200 | 3200 | 3200 | 10026 |
| 09_i64_hotloop | N/A | N/A | N/A | N/A | 3200 | 3200 | 3200 | 10026 |
| 10_function_calls | 67104 | 23850 | 23893 | 23893 | 3200 | 3200 | 3200 | 10026 |
| 11_string_concat_scan | N/A | N/A | N/A | N/A | 3200 | 3200 | 3200 | 10197 |
| 12_bigint_overflow | 207589 | 26880 | 26837 | 26837 | 3200 | 3200 | N/A | 10026 |

## Status

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|:---:|:------:|:------:|:------:|:-:|:----:|:---:|:------:|
| 01_fibonacci | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 02_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 03_matrix_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 04_quicksort | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 05_matrix_mul | ✅ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| 06_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 07_string_ops | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 08_int_hotloop | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 09_i64_hotloop | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 10_function_calls | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 11_string_concat_scan | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 12_bigint_overflow | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 🔨 | ✅ |

---

## Performance Analysis

### Performance Ratio vs C (Baseline)

| Benchmark | JIT vs C | AOT-O1 vs C | AOT-O2 vs C | AOT-O3 vs C |
|-----------|----------|-------------|-------------|-------------|
| 01_fibonacci | 2.7× | 1.9× | 1.8× | 2.0× |
| 02_prime_sieve | 1.0× | 1.0× | 1.0× | 7.4× |
| 03_matrix_mul | 1.0× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | 0.9× | 1.0× | 1.0× | 1.0× |
| 05_matrix_mul | 1.6× | N/A | N/A | N/A |
| 06_prime_sieve | 1.3× | 1.0× | 1.0× | 1.0× |
| 07_string_ops | N/A | N/A | N/A | N/A |
| 08_int_hotloop | 1.8× | 9.2× | 9.5× | 9.5× |
| 09_i64_hotloop | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 7.3× | 6.7× | 9.8× |
| 11_string_concat_scan | N/A | N/A | N/A | N/A |
| 12_bigint_overflow | 2.6× | 1.6× | 1.8× | 1.9× |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 2.6× | 1.8× | 1.8× | 1.9× |
| 02_prime_sieve | 1.0× | 1.0× | 1.0× | 7.5× |
| 03_matrix_mul | 1.0× | 0.9× | 0.9× | 1.0× |
| 04_quicksort | 0.9× | 1.0× | 0.9× | 0.9× |
| 05_matrix_mul | 1.5× | N/A | N/A | N/A |
| 06_prime_sieve | 1.3× | 1.0× | 1.0× | 0.9× |
| 07_string_ops | N/A | N/A | N/A | N/A |
| 08_int_hotloop | 1.7× | 8.9× | 9.2× | 9.2× |
| 09_i64_hotloop | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 7.3× | 6.7× | 9.8× |
| 11_string_concat_scan | N/A | N/A | N/A | N/A |
| 12_bigint_overflow | 2.7× | 1.6× | 1.9× | 1.9× |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 2.1× | 1.5× | 1.4× | 1.5× |
| 02_prime_sieve | 0.9× | 1.0× | 0.9× | 7.3× |
| 03_matrix_mul | 1.0× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | 1.0× | 1.1× | 1.0× | 1.0× |
| 05_matrix_mul | 1.5× | N/A | N/A | N/A |
| 06_prime_sieve | 1.3× | 1.0× | 1.0× | 0.9× |
| 07_string_ops | N/A | N/A | N/A | N/A |
| 08_int_hotloop | 1.7× | 8.6× | 8.9× | 9.0× |
| 09_i64_hotloop | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 7.2× | 6.6× | 9.6× |
| 11_string_concat_scan | N/A | N/A | N/A | N/A |
| 12_bigint_overflow | N/A | N/A | N/A | N/A |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 83031 | 25.9× |
| Viper AOT-O1 | 15338 | 4.8× |
| Viper AOT-O2 | 15343 | 4.8× |
| Viper AOT-O3 | 15343 | 4.8× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~25.9× memory overhead (83031KB vs C's ~3200KB)
3. **AOT memory** is ~4.8× C baseline (15342KB vs ~3200KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
