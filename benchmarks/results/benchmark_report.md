# Viper Benchmark Report

**Date:** 2026-03-14 07:09:29  
**Iterations:** 3  
**Max Memory Limit:** 4096MB  
**Max Time Limit:** 300s  

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | 96 |
| Passed | 91 |
| Failed/Crashed | 5 |
| Success Rate | 94% |

## Performance (Time in ms)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 290 | 179 | 182 | 141 | 73 | 90 | 128 | 1204 |
| 02_prime_sieve | 126 | 76 | 77 | 78 | 83 | 79 | 77 | 74 |
| 03_matrix_mul | 132 | 77 | 69 | 76 | 83 | 79 | 82 | 94 |
| 04_quicksort | 129 | 79 | 82 | 75 | 78 | 79 | 76 | 74 |
| 05_matrix_mul | 129 | 75 | 74 | 72 | 77 | 79 | 80 | 133 |
| 06_prime_sieve | 131 | 77 | 77 | 78 | 78 | 77 | 77 | 73 |
| 07_string_ops | CRASH | 87 | 86 | 79 | 76 | 85 | 79 | 169 |
| 08_int_hotloop | 235 | 137 | 148 | 138 | 87 | 86 | 83 | 436 |
| 09_nbody | CRASH | BUILD | BUILD | BUILD | 101 | 88 | 85 | 124 |
| 10_function_calls | 306 | 137 | 140 | 141 | 106 | 92 | 101 | 1009 |
| 11_string_concat_scan | 186 | 105 | 92 | 85 | 87 | 88 | 85 | 127 |
| 12_bigint_overflow | 545 | 230 | 214 | 234 | 93 | 91 | 144 | 134 |

## Memory (Peak RSS in KB)

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|-----|--------|--------|--------|---|------|-----|--------|
| 01_fibonacci | 66140 | 1706 | 1792 | 1706 | 1408 | 2005 | 1621 | 9984 |
| 02_prime_sieve | 67076 | 1792 | 1792 | 1749 | 1408 | 1962 | 1664 | 10112 |
| 03_matrix_mul | 67498 | 1749 | 1749 | 1792 | 1408 | 1962 | 1621 | 10069 |
| 04_quicksort | 68317 | 2133 | 2176 | 2176 | 1408 | 2048 | 1664 | 10154 |
| 05_matrix_mul | 68222 | 2176 | 2176 | 2090 | 1536 | 2048 | 1749 | 10282 |
| 06_prime_sieve | 67430 | 2176 | 2133 | 2133 | 1408 | 2048 | 1664 | 10154 |
| 07_string_ops | N/A | 1877 | 1920 | 1877 | 1408 | 2048 | 1621 | 10026 |
| 08_int_hotloop | 67252 | 1749 | 1749 | 1749 | 1408 | 1920 | 1664 | 10026 |
| 09_nbody | N/A | N/A | N/A | N/A | 1536 | 1962 | 1621 | 10624 |
| 10_function_calls | 67202 | 1749 | 1792 | 1706 | 1408 | 1920 | 1621 | 10026 |
| 11_string_concat_scan | 67349 | 1920 | 1920 | 1834 | 1408 | 2048 | 1621 | 10112 |
| 12_bigint_overflow | 207542 | 27008 | 27008 | 26922 | 1408 | 1962 | 6314 | 10026 |

## Status

| Benchmark | JIT | AOT-O1 | AOT-O2 | AOT-O3 | C | Rust | Go | Python |
|-----------|:---:|:------:|:------:|:------:|:-:|:----:|:---:|:------:|
| 01_fibonacci | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 02_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 03_matrix_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 04_quicksort | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 05_matrix_mul | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 06_prime_sieve | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 07_string_ops | ❌ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
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
| 01_fibonacci | 4.0× | 2.5× | 2.5× | 1.9× |
| 02_prime_sieve | 1.5× | 0.9× | 0.9× | 0.9× |
| 03_matrix_mul | 1.6× | 0.9× | 0.8× | 0.9× |
| 04_quicksort | 1.7× | 1.0× | 1.1× | 1.0× |
| 05_matrix_mul | 1.7× | 1.0× | 1.0× | 0.9× |
| 06_prime_sieve | 1.7× | 1.0× | 1.0× | 1.0× |
| 07_string_ops | N/A | 1.1× | 1.1× | 1.0× |
| 08_int_hotloop | 2.7× | 1.6× | 1.7× | 1.6× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 2.9× | 1.3× | 1.3× | 1.3× |
| 11_string_concat_scan | 2.1× | 1.2× | 1.1× | 1.0× |
| 12_bigint_overflow | 5.9× | 2.5× | 2.3× | 2.5× |

### Performance Ratio vs Rust

| Benchmark | JIT vs Rust | AOT-O1 vs Rust | AOT-O2 vs Rust | AOT-O3 vs Rust |
|-----------|-------------|----------------|----------------|----------------|
| 01_fibonacci | 3.2× | 2.0× | 2.0× | 1.6× |
| 02_prime_sieve | 1.6× | 1.0× | 1.0× | 1.0× |
| 03_matrix_mul | 1.7× | 1.0× | 0.9× | 1.0× |
| 04_quicksort | 1.6× | 1.0× | 1.0× | 0.9× |
| 05_matrix_mul | 1.6× | 0.9× | 0.9× | 0.9× |
| 06_prime_sieve | 1.7× | 1.0× | 1.0× | 1.0× |
| 07_string_ops | N/A | 1.0× | 1.0× | 0.9× |
| 08_int_hotloop | 2.7× | 1.6× | 1.7× | 1.6× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 3.3× | 1.5× | 1.5× | 1.5× |
| 11_string_concat_scan | 2.1× | 1.2× | 1.0× | 1.0× |
| 12_bigint_overflow | 6.0× | 2.5× | 2.4× | 2.6× |

### Performance Ratio vs Go

| Benchmark | JIT vs Go | AOT-O1 vs Go | AOT-O2 vs Go | AOT-O3 vs Go |
|-----------|-----------|--------------|--------------|--------------|
| 01_fibonacci | 2.3× | 1.4× | 1.4× | 1.1× |
| 02_prime_sieve | 1.6× | 1.0× | 1.0× | 1.0× |
| 03_matrix_mul | 1.6× | 0.9× | 0.8× | 0.9× |
| 04_quicksort | 1.7× | 1.0× | 1.1× | 1.0× |
| 05_matrix_mul | 1.6× | 0.9× | 0.9× | 0.9× |
| 06_prime_sieve | 1.7× | 1.0× | 1.0× | 1.0× |
| 07_string_ops | N/A | 1.1× | 1.1× | 1.0× |
| 08_int_hotloop | 2.8× | 1.7× | 1.8× | 1.7× |
| 09_nbody | N/A | N/A | N/A | N/A |
| 10_function_calls | 3.0× | 1.4× | 1.4× | 1.4× |
| 11_string_concat_scan | 2.2× | 1.2× | 1.1× | 1.0× |
| 12_bigint_overflow | 3.8× | 1.6× | 1.5× | 1.6× |

### Memory Efficiency

| Mode | Avg Memory (KB) | vs C |
|------|-----------------|------|
| Viper JIT | 81402 | 57.0× |
| Viper AOT-O1 | 4185 | 2.9× |
| Viper AOT-O2 | 4200 | 2.9× |
| Viper AOT-O3 | 4157 | 2.9× |

### Key Findings

1. **AOT-O1** typically offers the best performance/memory balance
2. **JIT mode** has ~57.0× memory overhead (81402KB vs C's ~1429KB)
3. **AOT memory** is ~2.9× C baseline (4181KB vs ~1429KB)
4. Performance varies by workload - see individual benchmark ratios above

---
*Generated by Viper Benchmark Runner*
