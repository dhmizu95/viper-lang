# Viper Benchmark Report

**Date:** 2026-03-15 17:58:59  
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
| 01_fibonacci | 203 | 65 | 62 | 68 | 63 | 62 | 61 | 683 |
| 02_prime_sieve | 64 | 64 | 71 | 69 | 69 | 64 | 68 | 68 |
| 03_matrix_mul | 63 | 60 | 67 | 67 | 70 | 70 | 68 | 68 |
| 04_quicksort | 83 | 68 | 67 | 67 | 67 | 64 | 65 | 64 |
| 05_matrix_mul | 64 | 65 | 67 | 65 | 67 | 66 | 67 | 69 |
| 06_prime_sieve | 63 | 67 | 70 | 65 | 69 | 64 | 65 | 65 |
| 07_string_ops | 66 | 70 | 70 | 66 | 71 | 70 | 66 | 70 |
| 08_int_hotloop | 121 | 64 | 68 | 69 | 70 | 67 | 68 | 217 |
| 09_nbody | 99 | BUILD | BUILD | BUILD | 65 | 65 | 68 | 64 |
| 10_function_calls | 119 | 60 | 67 | 67 | 68 | 69 | 65 | 273 |
| 11_string_concat_scan | 63 | 64 | 64 | 210 | 67 | 70 | 64 | 66 |
| 12_bigint_overflow | 185 | 62 | 61 | 61 | 70 | 62 | 69 | 62 |
| 13_factorial | 66 | 64 | 68 | 68 | 67 | 65 | 65 | 66 |
| 14_recursive_list_sum | CRASH | CRASH | CRASH | CRASH | 71 | 68 | 69 | CRASH |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 66310 | 2176 | 2176 | 2176 | 1408 | 1920 | 1578 | 9984 |
| 02_prime_sieve | 67736 | 2176 | 2090 | 2176 | 1408 | 1962 | 1621 | 10112 |
| 03_matrix_mul | 68053 | 2133 | 2176 | 2176 | 1408 | 1962 | 1664 | 10069 |
| 04_quicksort | 68597 | 2176 | 2176 | 2176 | 1408 | 2048 | 1621 | 10112 |
| 05_matrix_mul | 68582 | 2176 | 2133 | 2133 | 1536 | 2048 | 1706 | 10240 |
| 06_prime_sieve | 67989 | 2133 | 2048 | 2090 | 1408 | 2048 | 1664 | 10112 |
| 07_string_ops | 69648 | 4309 | 4309 | 4309 | 1408 | 2048 | 1706 | 9984 |
| 08_int_hotloop | 67766 | 2133 | 2176 | 2133 | 1408 | 1962 | 1664 | 9984 |
| 09_nbody | 69073 | N/A | N/A | N/A | 1536 | 1920 | 1664 | 10538 |
| 10_function_calls | 67793 | 2176 | 2176 | 2133 | 1408 | 1920 | 1578 | 10026 |
| 11_string_concat_scan | 67989 | 3029 | 3072 | 3029 | 1408 | 2048 | 1578 | 10112 |
| 12_bigint_overflow | 207770 | 33536 | 33536 | 33536 | 1408 | 2005 | 6186 | 9984 |
| 13_factorial | 67300 | 2133 | 2176 | 2133 | 1408 | 2005 | 1664 | 9984 |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A | 1408 | 2048 | 1578 | N/A |

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
| 01_fibonacci | 3.2× | 1.0× | 1.0× | 1.1× |
| 02_prime_sieve | 0.9× | 0.9× | 1.0× | 1.0× |
| 03_matrix_mul | 0.9× | 0.9× | 1.0× | 1.0× |
| 04_quicksort | 1.2× | 1.0× | 1.0× | 1.0× |
| 05_matrix_mul | 1.0× | 1.0× | 1.0× | 1.0× |
| 06_prime_sieve | 0.9× | 1.0× | 1.0× | 0.9× |
| 07_string_ops | 0.9× | 1.0× | 1.0× | 0.9× |
| 08_int_hotloop | 1.7× | 0.9× | 1.0× | 1.0× |
| 09_nbody | 1.5× | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 0.9× | 1.0× | 1.0× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 3.1× |
| 12_bigint_overflow | 2.6× | 0.9× | 0.9× | 0.9× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 3.3× | 1.0× | 1.0× | 1.1× |
| 02_prime_sieve | 1.0× | 1.0× | 1.1× | 1.1× |
| 03_matrix_mul | 0.9× | 0.9× | 1.0× | 1.0× |
| 04_quicksort | 1.3× | 1.1× | 1.0× | 1.0× |
| 05_matrix_mul | 1.0× | 1.0× | 1.0× | 1.0× |
| 06_prime_sieve | 1.0× | 1.0× | 1.1× | 1.0× |
| 07_string_ops | 0.9× | 1.0× | 1.0× | 0.9× |
| 08_int_hotloop | 1.8× | 1.0× | 1.0× | 1.0× |
| 09_nbody | 1.5× | N/A | N/A | N/A |
| 10_function_calls | 1.7× | 0.9× | 1.0× | 1.0× |
| 11_string_concat_scan | 0.9× | 0.9× | 0.9× | 3.0× |
| 12_bigint_overflow | 3.0× | 1.0× | 1.0× | 1.0× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 3.3× | 1.1× | 1.0× | 1.1× |
| 02_prime_sieve | 0.9× | 0.9× | 1.0× | 1.0× |
| 03_matrix_mul | 0.9× | 0.9× | 1.0× | 1.0× |
| 04_quicksort | 1.3× | 1.0× | 1.0× | 1.0× |
| 05_matrix_mul | 1.0× | 1.0× | 1.0× | 1.0× |
| 06_prime_sieve | 1.0× | 1.0× | 1.1× | 1.0× |
| 07_string_ops | 1.0× | 1.1× | 1.1× | 1.0× |
| 08_int_hotloop | 1.8× | 0.9× | 1.0× | 1.0× |
| 09_nbody | 1.5× | N/A | N/A | N/A |
| 10_function_calls | 1.8× | 0.9× | 1.0× | 1.0× |
| 11_string_concat_scan | 1.0× | 1.0× | 1.0× | 3.3× |
| 12_bigint_overflow | 2.7× | 0.9× | 0.9× | 0.9× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 78815 | 55.3× |
| Viper AOT-O1 | 5023 | 3.5× |
| Viper AOT-O2 | 5020 | 3.5× |
| Viper AOT-O3 | 5016 | 3.5× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~55.3× memory overhead (78815KB vs C's ~1426KB)
3. **AOT memory** is ~3.5× C baseline (5020KB vs ~1426KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
