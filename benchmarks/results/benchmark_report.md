# Viper Benchmark Report

**Date:** 2026-03-14 22:29:27  
**Iterations:** 3  
**Max Memory Limit:** 4096MB  
**Max Time Limit:** 300s  

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 96 |
| Passed | 96 |
| Failed/Crashed | 0 |
| Success Rate | 100% |

## Performance (Time in ms)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 165 | 115 | 118 | 118 | 65 | 66 | 60 | 683 |
| 02_prime_sieve | 66 | 68 | 67 | 66 | 64 | 66 | 68 | 66 |
| 03_matrix_mul | 63 | 64 | 68 | 65 | 67 | 66 | 65 | 66 |
| 04_quicksort | 61 | 65 | 69 | 67 | 60 | 69 | 66 | 66 |
| 05_matrix_mul | 60 | 67 | 68 | 69 | 68 | 67 | 66 | 66 |
| 06_prime_sieve | 64 | 69 | 67 | 69 | 69 | 68 | 70 | 66 |
| 07_string_ops | 62 | 68 | 347 | 67 | 68 | 66 | 67 | 62 |
| 08_int_hotloop | 116 | 64 | 63 | 65 | 68 | 66 | 67 | 218 |
| 09_nbody | 78 | 63 | 65 | 65 | 66 | 68 | 70 | 64 |
| 10_function_calls | 117 | 65 | 65 | 62 | 66 | 65 | 69 | 270 |
| 11_string_concat_scan | 62 | 67 | 69 | 69 | 67 | 69 | 68 | 66 |
| 12_bigint_overflow | 164 | 79 | 59 | 62 | 63 | 70 | 64 | 62 |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 66217 | 1706 | 1749 | 1706 | 1408 | 1920 | 1536 | 10026 |
| 02_prime_sieve | 67344 | 1792 | 1792 | 1792 | 1408 | 1962 | 1664 | 10112 |
| 03_matrix_mul | 67628 | 1792 | 1792 | 1749 | 1408 | 1920 | 1664 | 9941 |
| 04_quicksort | 68373 | 2176 | 2133 | 2176 | 1408 | 2048 | 1621 | 10154 |
| 05_matrix_mul | 68377 | 2176 | 2176 | 2176 | 1536 | 2048 | 1706 | 10282 |
| 06_prime_sieve | 67733 | 2133 | 2090 | 2176 | 1408 | 2048 | 1664 | 10112 |
| 07_string_ops | 67516 | 1920 | 1920 | 1877 | 1408 | 2048 | 1664 | 10026 |
| 08_int_hotloop | 67341 | 1792 | 1706 | 1749 | 1408 | 2005 | 1664 | 9984 |
| 09_nbody | 68742 | 2432 | 2389 | 2432 | 1536 | 2005 | 1664 | 10624 |
| 10_function_calls | 67466 | 1749 | 1706 | 1749 | 1408 | 1962 | 1664 | 10026 |
| 11_string_concat_scan | 67705 | 1920 | 1920 | 1920 | 1408 | 2048 | 1621 | 10154 |
| 12_bigint_overflow | 207617 | 27008 | 27008 | 27008 | 1408 | 1920 | 6528 | 10069 |

## Status

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|:---:|:------:|:------:|:------:|:-:|:----:|:---:|:------:|
| 01_fibonacci | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 02_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 03_matrix_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 04_quicksort | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 05_matrix_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 06_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 07_string_ops | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 08_int_hotloop | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 09_nbody | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 10_function_calls | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 11_string_concat_scan | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 12_bigint_overflow | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Performance Analysis

### Performance Ratio vs C (Baseline)

| Benchmark | JIT vs C | AOT-O1 vs C | AOT-O2 vs C | AOT-O3 vs C |
|-----------|----------|-------------|-------------|-------------|
| 01_fibonacci | 2.5× | 1.8× | 1.8× | 1.8× |
| 02_prime_sieve | 1.0× | 1.1× | 1.0× | 1.0× |
| 03_matrix_mul | 0.9× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | 1.0× | 1.1× | 1.1× | 1.1× |
| 05_matrix_mul | 0.9× | 1.0× | 1.0× | 1.0× |
| 06_prime_sieve | 0.9× | 1.0× | 1.0× | 1.0× |
| 07_string_ops | 0.9× | 1.0× | 5.1× | 1.0× |
| 08_int_hotloop | 1.7× | 0.9× | 0.9× | 1.0× |
| 09_nbody | 1.2× | 1.0× | 1.0× | 1.0× |
| 10_function_calls | 1.8× | 1.0× | 1.0× | 0.9× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 2.6× | 1.3× | 0.9× | 1.0× |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 2.5× | 1.7× | 1.8× | 1.8× |
| 02_prime_sieve | 1.0× | 1.0× | 1.0× | 1.0× |
| 03_matrix_mul | 1.0× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | 0.9× | 0.9× | 1.0× | 1.0× |
| 05_matrix_mul | 0.9× | 1.0× | 1.0× | 1.0× |
| 06_prime_sieve | 0.9× | 1.0× | 1.0× | 1.0× |
| 07_string_ops | 0.9× | 1.0× | 5.3× | 1.0× |
| 08_int_hotloop | 1.8× | 1.0× | 1.0× | 1.0× |
| 09_nbody | 1.1× | 0.9× | 1.0× | 1.0× |
| 10_function_calls | 1.8× | 1.0× | 1.0× | 1.0× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 2.3× | 1.1× | 0.8× | 0.9× |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 2.8× | 1.9× | 2.0× | 2.0× |
| 02_prime_sieve | 1.0× | 1.0× | 1.0× | 1.0× |
| 03_matrix_mul | 1.0× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | 0.9× | 1.0× | 1.0× | 1.0× |
| 05_matrix_mul | 0.9× | 1.0× | 1.0× | 1.0× |
| 06_prime_sieve | 0.9× | 1.0× | 1.0× | 1.0× |
| 07_string_ops | 0.9× | 1.0× | 5.2× | 1.0× |
| 08_int_hotloop | 1.7× | 1.0× | 0.9× | 1.0× |
| 09_nbody | 1.1× | 0.9× | 0.9× | 0.9× |
| 10_function_calls | 1.7× | 0.9× | 0.9× | 0.9× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 2.6× | 1.2× | 0.9× | 1.0× |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 79338 | 55.5× |
| Viper AOT-O1 | 4049 | 2.8× |
| Viper AOT-O2 | 4031 | 2.8× |
| Viper AOT-O3 | 4042 | 2.8× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~55.5× memory overhead (79338KB vs C's ~1429KB)
3. **AOT memory** is ~2.8× C baseline (4041KB vs ~1429KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
