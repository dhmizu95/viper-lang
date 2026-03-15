# Viper Benchmark Report

**Date:** 2026-03-15 07:36:35  
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
| 01_fibonacci | 169 | 121 | 60 | 60 | 67 | 66 | 63 | 685 |
| 02_prime_sieve | 81 | 71 | 68 | 71 | 68 | 69 | 73 | 69 |
| 03_matrix_mul | 64 | 66 | 73 | 68 | 66 | 67 | 65 | 68 |
| 04_quicksort | 63 | 69 | 68 | 66 | 71 | 67 | 64 | 62 |
| 05_matrix_mul | 80 | 68 | 71 | 67 | 66 | 65 | 63 | 65 |
| 06_prime_sieve | 61 | 77 | 359 | 69 | 65 | 64 | 65 | 68 |
| 07_string_ops | 64 | 65 | 67 | 66 | 70 | 72 | 66 | 67 |
| 08_int_hotloop | 115 | 355 | 67 | 63 | 66 | 66 | 68 | 223 |
| 09_nbody | 63 | BUILD | BUILD | BUILD | 66 | 68 | 70 | 69 |
| 10_function_calls | 115 | 85 | 84 | 63 | 72 | 69 | 69 | 273 |
| 11_string_concat_scan | 60 | 67 | 68 | 67 | 67 | 67 | 62 | 69 |
| 12_bigint_overflow | 256 | 367 | 61 | 85 | 70 | 70 | 67 | 64 |
| 13_factorial | 63 | 66 | 63 | 68 | 64 | 64 | 69 | 67 |
| 14_recursive_list_sum | CRASH | CRASH | CRASH | CRASH | 65 | 66 | 66 | CRASH |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 67304 | 2133 | 2133 | 2133 | 1408 | 1962 | 1621 | 9984 |
| 02_prime_sieve | 67893 | 2133 | 2176 | 2176 | 1408 | 2005 | 1621 | 10154 |
| 03_matrix_mul | 68165 | 2176 | 2176 | 2176 | 1408 | 1920 | 1664 | 10026 |
| 04_quicksort | 68714 | 2176 | 2090 | 2176 | 1408 | 2048 | 1664 | 10112 |
| 05_matrix_mul | 68933 | 2176 | 2176 | 2090 | 1536 | 2048 | 1706 | 10282 |
| 06_prime_sieve | 68068 | 2133 | 2176 | 2176 | 1408 | 2048 | 1664 | 10154 |
| 07_string_ops | 69880 | 4309 | 4352 | 4309 | 1408 | 2048 | 1706 | 10026 |
| 08_int_hotloop | 67845 | 2176 | 2176 | 2090 | 1408 | 1920 | 1664 | 10026 |
| 09_nbody | 69134 | N/A | N/A | N/A | 1536 | 1920 | 1664 | 10624 |
| 10_function_calls | 67830 | 2176 | 2176 | 2090 | 1408 | 1962 | 1621 | 9984 |
| 11_string_concat_scan | 68038 | 3072 | 3072 | 3072 | 1408 | 2048 | 1578 | 10069 |
| 12_bigint_overflow | 207777 | 33536 | 33493 | 33450 | 1408 | 1962 | 6826 | 10069 |
| 13_factorial | 67377 | 2176 | 2176 | 2176 | 1408 | 1962 | 1664 | 10112 |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A | 1408 | 2048 | 1621 | N/A |

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
| 01_fibonacci | 2.5× | 1.8× | 0.9× | 0.9× |
| 02_prime_sieve | 1.2× | 1.0× | 1.0× | 1.0× |
| 03_matrix_mul | 1.0× | 1.0× | 1.1× | 1.0× |
| 04_quicksort | 0.9× | 1.0× | 1.0× | 0.9× |
| 05_matrix_mul | 1.2× | 1.0× | 1.1× | 1.0× |
| 06_prime_sieve | 0.9× | 1.2× | 5.5× | 1.1× |
| 07_string_ops | 0.9× | 0.9× | 1.0× | 0.9× |
| 08_int_hotloop | 1.7× | 5.4× | 1.0× | 1.0× |
| 09_nbody | 1.0× | N/A | N/A | N/A |
| 10_function_calls | 1.6× | 1.2× | 1.2× | 0.9× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 3.7× | 5.2× | 0.9× | 1.2× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.1× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 2.6× | 1.8× | 0.9× | 0.9× |
| 02_prime_sieve | 1.2× | 1.0× | 1.0× | 1.0× |
| 03_matrix_mul | 1.0× | 1.0× | 1.1× | 1.0× |
| 04_quicksort | 0.9× | 1.0× | 1.0× | 1.0× |
| 05_matrix_mul | 1.2× | 1.0× | 1.1× | 1.0× |
| 06_prime_sieve | 1.0× | 1.2× | 5.6× | 1.1× |
| 07_string_ops | 0.9× | 0.9× | 0.9× | 0.9× |
| 08_int_hotloop | 1.7× | 5.4× | 1.0× | 1.0× |
| 09_nbody | 0.9× | N/A | N/A | N/A |
| 10_function_calls | 1.7× | 1.2× | 1.2× | 0.9× |
| 11_string_concat_scan | 0.9× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 3.7× | 5.2× | 0.9× | 1.2× |
| 13_factorial | 1.0× | 1.0× | 1.0× | 1.1× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 2.7× | 1.9× | 1.0× | 1.0× |
| 02_prime_sieve | 1.1× | 1.0× | 0.9× | 1.0× |
| 03_matrix_mul | 1.0× | 1.0× | 1.1× | 1.0× |
| 04_quicksort | 1.0× | 1.1× | 1.1× | 1.0× |
| 05_matrix_mul | 1.3× | 1.1× | 1.1× | 1.1× |
| 06_prime_sieve | 0.9× | 1.2× | 5.5× | 1.1× |
| 07_string_ops | 1.0× | 1.0× | 1.0× | 1.0× |
| 08_int_hotloop | 1.7× | 5.2× | 1.0× | 0.9× |
| 09_nbody | 0.9× | N/A | N/A | N/A |
| 10_function_calls | 1.7× | 1.2× | 1.2× | 0.9× |
| 11_string_concat_scan | 1.0× | 1.1× | 1.1× | 1.1× |
| 12_bigint_overflow | 3.8× | 5.5× | 0.9× | 1.3× |
| 13_factorial | 0.9× | 1.0× | 0.9× | 1.0× |
| 14_recursive_list_sum | N/A | N/A | N/A | N/A |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 78996 | 55.4× |
| Viper AOT-O1 | 5031 | 3.5× |
| Viper AOT-O2 | 5031 | 3.5× |
| Viper AOT-O3 | 5009 | 3.5× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~55.4× memory overhead (78996KB vs C's ~1426KB)
3. **AOT memory** is ~3.5× C baseline (5023KB vs ~1426KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
