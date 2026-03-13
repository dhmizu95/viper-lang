# Viper Benchmark Report

**Date:** 2026-03-13 10:04:41  
**Iterations:** 3  
**Max Memory Limit:** 4096MB  
**Max Time Limit:** 300s  

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 96 |
| Passed | 92 |
| Failed/Crashed | 4 |
| Success Rate | 95% |

## Performance (Time in ms)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 247 | 215 | 118 | 144 | 96 | 69 | 126 | 1156 |
| 02_prime_sieve | 516 | 81 | 73 | 72 | 69 | 73 | 70 | 93 |
| 03_matrix_mul | 127 | 70 | 82 | 82 | 69 | 78 | 75 | 69 |
| 04_quicksort | 126 | 72 | 78 | 79 | 77 | 74 | 70 | 71 |
| 05_matrix_mul | 123 | 81 | 72 | 76 | 73 | 72 | 72 | 67 |
| 06_prime_sieve | 103 | 76 | 63 | 81 | 75 | 70 | 72 | 73 |
| 07_string_ops | 121 | 70 | 74 | 70 | 70 | 70 | 73 | 70 |
| 08_int_hotloop | 140 | 74 | 68 | 70 | 68 | 73 | 68 | 352 |
| 09_nbody | CRASH | BUILD | BUILD | BUILD | 71 | 68 | 74 | 68 |
| 10_function_calls | 118 | 94 | 72 | 96 | 73 | 75 | 71 | 565 |
| 11_string_concat_scan | 129 | 77 | 75 | 74 | 74 | 78 | 75 | 96 |
| 12_bigint_overflow | 359 | 648 | 184 | 125 | 77 | 85 | 72 | 127 |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 66694 | 1792 | 1749 | 1792 | 1408 | 1962 | 1536 | 9984 |
| 02_prime_sieve | 67120 | 1792 | 1792 | 1792 | 1408 | 1962 | 1578 | 10154 |
| 03_matrix_mul | 67626 | 1792 | 1792 | 1792 | 1408 | 1920 | 1578 | 10112 |
| 04_quicksort | 68236 | 2133 | 2133 | 2176 | 1408 | 2048 | 1664 | 10112 |
| 05_matrix_mul | 68308 | 2176 | 2176 | 2176 | 1536 | 2048 | 1706 | 10240 |
| 06_prime_sieve | 67474 | 2133 | 2176 | 2133 | 1408 | 2048 | 1621 | 10154 |
| 07_string_ops | 67268 | 1920 | 1877 | 1920 | 1408 | 2048 | 1664 | 10026 |
| 08_int_hotloop | 67344 | 1749 | 1749 | 1749 | 1408 | 2005 | 1621 | 9984 |
| 09_nbody | N/A | N/A | N/A | N/A | 1536 | 2005 | 1664 | 10624 |
| 10_function_calls | 67270 | 1749 | 1792 | 1749 | 1408 | 1920 | 1621 | 10026 |
| 11_string_concat_scan | 67589 | 2005 | 2005 | 1962 | 1408 | 2048 | 1664 | 10112 |
| 12_bigint_overflow | 207509 | 27008 | 27008 | 27008 | 1408 | 1920 | 6485 | 9984 |

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
| 09_nbody | ❌ | 🔨 | 🔨 | 🔨 | ✅ | ✅ | ✅ | ✅ |
| 10_function_calls | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 11_string_concat_scan | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 12_bigint_overflow | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## Performance Analysis

### Performance Ratio vs C (Baseline)

| Benchmark | JIT vs C | AOT-O1 vs C | AOT-O2 vs C | AOT-O3 vs C |
|-----------|----------|-------------|-------------|-------------|
| 01_fibonacci | 2.6× | 2.2× | 1.2× | 1.5× |
| 02_prime_sieve | 7.5× | 1.2× | 1.1× | 1.0× |
| 03_matrix_mul | 1.8× | 1.0× | 1.2× | 1.2× |
| 04_quicksort | 1.6× | 0.9× | 1.0× | 1.0× |
| 05_matrix_mul | 1.7× | 1.1× | 1.0× | 1.0× |
| 06_prime_sieve | 1.4× | 1.0× | 0.8× | 1.1× |
| 07_string_ops | 1.7× | 1.0× | 1.1× | 1.0× |
| 08_int_hotloop | 2.1× | 1.1× | 1.0× | 1.0× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.6× | 1.3× | 1.0× | 1.3× |
| 11_string_concat_scan | 1.7× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 4.7× | 8.4× | 2.4× | 1.6× |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 3.6× | 3.1× | 1.7× | 2.1× |
| 02_prime_sieve | 7.1× | 1.1× | 1.0× | 1.0× |
| 03_matrix_mul | 1.6× | 0.9× | 1.1× | 1.1× |
| 04_quicksort | 1.7× | 1.0× | 1.1× | 1.1× |
| 05_matrix_mul | 1.7× | 1.1× | 1.0× | 1.1× |
| 06_prime_sieve | 1.5× | 1.1× | 0.9× | 1.2× |
| 07_string_ops | 1.7× | 1.0× | 1.1× | 1.0× |
| 08_int_hotloop | 1.9× | 1.0× | 0.9× | 1.0× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.6× | 1.3× | 1.0× | 1.3× |
| 11_string_concat_scan | 1.7× | 1.0× | 1.0× | 0.9× |
| 12_bigint_overflow | 4.2× | 7.6× | 2.2× | 1.5× |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 2.0× | 1.7× | 0.9× | 1.1× |
| 02_prime_sieve | 7.4× | 1.2× | 1.0× | 1.0× |
| 03_matrix_mul | 1.7× | 0.9× | 1.1× | 1.1× |
| 04_quicksort | 1.8× | 1.0× | 1.1× | 1.1× |
| 05_matrix_mul | 1.7× | 1.1× | 1.0× | 1.1× |
| 06_prime_sieve | 1.4× | 1.1× | 0.9× | 1.1× |
| 07_string_ops | 1.7× | 1.0× | 1.0× | 1.0× |
| 08_int_hotloop | 2.1× | 1.1× | 1.0× | 1.0× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 1.7× | 1.3× | 1.0× | 1.4× |
| 11_string_concat_scan | 1.7× | 1.0× | 1.0× | 1.0× |
| 12_bigint_overflow | 5.0× | 9.0× | 2.6× | 1.7× |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 80221 | 56.1× |
| Viper AOT-O1 | 4204 | 2.9× |
| Viper AOT-O2 | 4204 | 2.9× |
| Viper AOT-O3 | 4204 | 2.9× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~56.1× memory overhead (80221KB vs C's ~1429KB)
3. **AOT memory** is ~2.9× C baseline (4204KB vs ~1429KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
