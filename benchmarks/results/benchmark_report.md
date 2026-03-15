# Viper Benchmark Report

**Date:** 2026-03-15 06:51:19  
**Iterations:** 3  
**Max Memory Limit:** 4096MB  
**Max Time Limit:** 300s  

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 112 |
| Passed | 104 |
| Failed/Crashed | 8 |
| Success Rate | 92% |

## Performance (Time in ms)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 148 | 114 | 116 | 119 | 62 | 62 | 81 | 828 |
| 02_prime_sieve | 82 | 68 | 66 | 63 | 63 | 62 | 63 | 63 |
| 03_matrix_mul | 85 | 63 | 63 | 61 | 65 | 64 | 68 | 65 |
| 04_quicksort | 347 | 68 | 82 | 76 | 74 | 80 | 62 | 67 |
| 05_matrix_mul | 83 | 70 | 70 | 66 | 64 | 63 | 66 | 62 |
| 06_prime_sieve | 64 | 70 | 69 | 72 | 68 | 67 | 67 | 66 |
| 07_string_ops | 80 | 69 | 64 | 66 | 67 | 68 | 72 | 86 |
| 08_int_hotloop | 117 | 114 | 119 | 117 | 64 | 63 | 68 | 234 |
| 09_nbody | 104 | BUILD | BUILD | BUILD | 67 | 75 | 70 | 69 |
| 10_function_calls | 118 | 61 | 63 | 62 | 66 | 67 | 69 | 273 |
| 11_string_concat_scan | 61 | 65 | 66 | 67 | 66 | 68 | 68 | 67 |
| 12_bigint_overflow | 201 | 115 | 113 | 116 | 63 | 64 | 82 | 66 |
| 13_factorial | 66 | 68 | 66 | 73 | 70 | 71 | 67 | 66 |
| 14_recursive_list_sum | CRASH | CRASH | CRASH | CRASH | 64 | 65 | 76 | CRASH |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 67341 | 2517 | 2517 | 2517 | 1408 | 1920 | 1621 | 9984 |
| 02_prime_sieve | 67865 | 2517 | 2474 | 2517 | 1408 | 1962 | 1621 | 10197 |
| 03_matrix_mul | 68217 | 2474 | 2474 | 2517 | 1408 | 1962 | 1706 | 10197 |
| 04_quicksort | 68838 | 2517 | 2474 | 2560 | 1408 | 2048 | 1536 | 10112 |
| 05_matrix_mul | 68893 | 2474 | 2517 | 2517 | 1536 | 2048 | 1706 | 10240 |
| 06_prime_sieve | 68065 | 2474 | 2474 | 2474 | 1408 | 2048 | 1621 | 10154 |
| 07_string_ops | 69870 | 4650 | 4693 | 4778 | 1408 | 2048 | 1664 | 10026 |
| 08_int_hotloop | 67884 | 2517 | 2432 | 2560 | 1408 | 1920 | 1621 | 10026 |
| 09_nbody | 69192 | N/A | N/A | N/A | 1536 | 2005 | 1578 | 10624 |
| 10_function_calls | 67829 | 2517 | 2517 | 2560 | 1408 | 2048 | 1664 | 9984 |
| 11_string_concat_scan | 68012 | 3456 | 3456 | 3370 | 1365 | 1962 | 1578 | 10154 |
| 12_bigint_overflow | 207896 | 33877 | 33920 | 33920 | 1408 | 1962 | 6400 | 9984 |
| 13_factorial | 67356 | 2474 | 2517 | 2517 | 1408 | 2005 | 1621 | 10112 |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A | 1408 | 2048 | 1749 | N/A |

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
| 09_nbody | ✅ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 10_function_calls | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 11_string_concat_scan | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 12_bigint_overflow | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 13_factorial | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 14_recursive_list_sum | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ | ✅ | ❌ |

---

## Performance Analysis

### Performance Ratio vs C (Baseline)

| Benchmark | JIT vs C | AOT-O1 vs C | AOT-O2 vs C | AOT-O3 vs C |
|-----------|----------|-------------|-------------|-------------|
| 01_fibonacci | 2.4× | 1.8× | 1.9× | 1.9× |
| 02_prime_sieve | 1.3× | 1.1× | 1.0× | 1.0× |
| 03_matrix_mul | 1.3× | 1.0× | 1.0× | 0.9× |
| 04_quicksort | 4.7× | 0.9× | 1.1× | 1.0× |
| 05_matrix_mul | 1.3× | 1.1× | 1.1× | 1.0× |
| 06_prime_sieve | 0.9× | 1.0× | 1.0× | 1.1× |
| 07_string_ops | 1.2× | 1.0× | 1.0× | 1.0× |
| 08_int_hotloop | 1.8× | 1.8× | 1.9× | 1.8× |
| 09_nbody | 1.6× | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 0.9× | 1.0× | 0.9× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 3.2× | 1.8× | 1.8× | 1.8× |
| 13_factorial | 0.9× | 1.0× | 0.9× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 2.4× | 1.8× | 1.9× | 1.9× |
| 02_prime_sieve | 1.3× | 1.1× | 1.1× | 1.0× |
| 03_matrix_mul | 1.3× | 1.0× | 1.0× | 1.0× |
| 04_quicksort | 4.3× | 0.8× | 1.0× | 0.9× |
| 05_matrix_mul | 1.3× | 1.1× | 1.1× | 1.0× |
| 06_prime_sieve | 1.0× | 1.0× | 1.0× | 1.1× |
| 07_string_ops | 1.2× | 1.0× | 0.9× | 1.0× |
| 08_int_hotloop | 1.9× | 1.8× | 1.9× | 1.9× |
| 09_nbody | 1.4× | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 0.9× | 0.9× | 0.9× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 3.1× | 1.8× | 1.8× | 1.8× |
| 13_factorial | 0.9× | 1.0× | 0.9× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 1.8× | 1.4× | 1.4× | 1.5× |
| 02_prime_sieve | 1.3× | 1.1× | 1.0× | 1.0× |
| 03_matrix_mul | 1.2× | 0.9× | 0.9× | 0.9× |
| 04_quicksort | 5.6× | 1.1× | 1.3× | 1.2× |
| 05_matrix_mul | 1.3× | 1.1× | 1.1× | 1.0× |
| 06_prime_sieve | 1.0× | 1.0× | 1.0× | 1.1× |
| 07_string_ops | 1.1× | 1.0× | 0.9× | 0.9× |
| 08_int_hotloop | 1.7× | 1.7× | 1.8× | 1.7× |
| 09_nbody | 1.5× | N/A | N/A | N/A |
| 10_function_calls | 1.7× | 0.9× | 0.9× | 0.9× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 2.5× | 1.4× | 1.4× | 1.4× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.1× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 79019 | 55.5× |
| Viper AOT-O1 | 5372 | 3.8× |
| Viper AOT-O2 | 5372 | 3.8× |
| Viper AOT-O3 | 5400 | 3.8× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~55.5× memory overhead (79019KB vs C's ~1423KB)
3. **AOT memory** is ~3.8× C baseline (5381KB vs ~1423KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
