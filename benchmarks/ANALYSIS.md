# Viper Language Benchmark Analysis Report

**Date:** February 25, 2026  
**Compiler Version:** Viper 0.2.2 (AOT)  
**Comparison Languages:** C (GCC), Rust (rustc), Go (not installed)

---

## Executive Summary

This benchmark suite compared Viper against C and Rust across 15 computational problems. **Key finding: Viper's compiler has significant bugs preventing most benchmarks from compiling**, revealing critical issues in code generation, type handling, and standard library support.

### Benchmark Status

| Language | Compiled | Ran Successfully |
|----------|----------|------------------|
| C (GCC) | 15/15 ✅ | 15/15 ✅ |
| Rust | 15/15 ✅ | 15/15 ✅ |
| Go | N/A (not installed) | N/A |
| **Viper** | **0/15 ❌** | **0/15 ❌** |

---

## Detailed Analysis

### 1. Compilation Failures

All 15 Viper benchmarks failed to compile. Here are the specific errors:

#### Error Category 1: List/Array Handling Bugs

**Error Message:**
```
Found PointerValue but expected the IntValue variant
Location: src/codegen/mod.rs:849:39
```

**Affected Benchmarks:** 01, 03, 04, 07, 08

**Example Code That Fails:**
```python
is_prime = [True] * (LIMIT + 1)  # List creation
arr[i] = value                    # List assignment
```

**Root Cause:** The code generator incorrectly handles list pointers, trying to use pointer values as integers.

---

#### Error Category 2: Type System Issues

**Error Message:**
```
Found FloatValue but expected the IntValue variant
Location: src/codegen/mod.rs:850:39
```

**Affected Benchmarks:** 05, 06, 10, 15

**Example Code That Fails:**
```python
while x * x + y * y <= 4.0:  # Float comparison
    xtemp = x * x - y * y + x0
```

**Root Cause:** Mixed integer/float operations not properly handled in type checking.

---

#### Error Category 3: Function Call Issues

**Error Message:**
```
print() argument evaluation failed
```

**Affected Benchmarks:** 02, 11, 12, 13, 14, 15

**Example Code That Fails:**
```python
print("Fibonacci iterations: " + str(count))
```

**Root Cause:** String concatenation and conversion functions not working in print arguments.

---

#### Error Category 4: Parser Limitations

**Error Message:**
```
Unexpected token in expression: Error("Unexpected character: ';'")
Terminator found in the middle of a basic block!
```

**Affected Benchmarks:** 06, 08

**Root Cause:** Comments with certain characters confuse the lexer; control flow generation broken.

---

#### Error Category 5: Missing Standard Library Functions

**Missing Functions:**
- `sqrt()` - Square root (needed for benchmarks 06, 10)
- `ln()` - Natural logarithm (needed for benchmark 15)
- `abs()` - Absolute value (needed for benchmark 15)
- `str()` - Integer to string conversion (needed everywhere)
- `len()` - String/list length (needed for benchmarks 11-14)

---

### 2. Performance Comparison (C vs Rust)

For benchmarks that successfully ran:

| Benchmark | Problem Size | C Time | Rust Time | Ratio (Rust/C) |
|-----------|--------------|--------|-----------|----------------|
| 01 Prime Sieve | 10M primes | 0.068s | 0.075s | 1.10x |
| 02 Fibonacci | 10M iterations | 0.004s | 0.003s | 0.75x |
| 03 Matrix Mult | 512×512 | 0.092s | 0.243s | 2.64x |
| 04 QuickSort | 100k elements | 0.015s | 0.017s | 1.13x |
| 05 Mandelbrot | 1000×1000 | 0.086s | 0.094s | 1.09x |

**Observations:**
- C and Rust performance is comparable (within 2-3x)
- Matrix multiplication shows Rust is slower without explicit SIMD hints
- Both languages handle all problem sizes efficiently

---

### 3. Viper's Missing Features (Discovered During Implementation)

#### Critical Missing Features

1. **Global Constants**
   ```python
   # ❌ Doesn't work
   LIMIT = 1000000
   
   def main():
       x = LIMIT  # "Undefined variable: LIMIT"
   ```
   **Workaround:** Define constants inside functions

2. **List Comprehensions**
   ```python
   # ❌ Doesn't work
   nums = [i for i in range(10)]
   ```
   **Workaround:** Use explicit loops

3. **Dictionary/Map Types**
   ```python
   # ❌ Doesn't work
   counts = {"key": "value"}
   ```
   **Impact:** Benchmark 11 (K-Nucleotide) requires hash maps

4. **String Formatting**
   ```python
   # ❌ Doesn't work
   print(f"Value: {x}")
   ```
   **Workaround:** Use concatenation (also buggy)

5. **Math Functions**
   ```python
   # ❌ Doesn't work
   x = sqrt(2.0)
   y = ln(10.0)
   z = abs(-5.0)
   ```

6. **Struct/Class Types**
   ```python
   # ❌ Doesn't work
   class Point:
       x = 0.0
       y = 0.0
   ```
   **Impact:** Benchmark 06 (Ray Tracer) requires structs for Vec3

7. **Tuple Unpacking**
   ```python
   # ❌ Doesn't work
   a, b = b, a
   ```

8. **Negative Array Indexing**
   ```python
   # ❌ Doesn't work
   last = arr[-1]
   ```

9. **String Slicing with Negative Indices**
   ```python
   # ❌ Doesn't work
   reversed = text[::-1]
   ```

10. **Boolean Operators in Expressions**
    ```python
    # ❌ Doesn't work reliably
    if x > 0 and x < 10:
    ```

---

### 4. Scale Reductions Required

Due to Viper's limitations, benchmarks had to be significantly scaled down:

| Benchmark | C/Rust Scale | Viper Scale | Reduction Factor |
|-----------|--------------|-------------|------------------|
| Prime Sieve | 10M | 100k | 100x |
| Matrix Mult | 512×512 | 50×50 | 100x |
| Fibonacci | 10M iter | 1M iter | 10x |
| Mandelbrot | 1000×1000 | 100×100 | 100x |
| N-Body | 500 bodies | 30 bodies | 16x |
| QuickSort | 100k elems | 5k elems | 20x |

---

## Recommendations for Viper Improvement

### Priority 1: Critical Compiler Bugs (Block Compilation)

1. **Fix List Code Generation**
   - File: `src/codegen/mod.rs`
   - Issue: PointerValue/IntValue confusion
   - Impact: Prevents all array/list operations

2. **Fix Type System for Floats**
   - File: `src/codegen/mod.rs`
   - Issue: FloatValue/IntValue confusion
   - Impact: Prevents floating-point math

3. **Fix Function Call Evaluation**
   - File: `src/codegen/mod.rs`
   - Issue: print() argument evaluation
   - Impact: Prevents all I/O

### Priority 2: Standard Library Implementation

4. **Add Math Builtins**
   ```python
   sqrt(x)    # Square root
   ln(x)      # Natural log
   abs(x)     # Absolute value
   pow(x, y)  # Power
   ```

5. **Add String Functions**
   ```python
   str(x)     # Convert to string
   len(s)     # String/list length
   ```

6. **Fix List Operations**
   - List creation with initial values
   - List indexing and assignment
   - List slicing

### Priority 3: Language Features

7. **Add Global Constants**
   - Allow `CONST_NAME = value` at module level
   - Support const folding in compiler

8. **Add Struct Types**
   ```python
   struct Point:
       x: f64
       y: f64
   ```

9. **Add Dictionary Type**
   ```python
   counts = {"key": 0}
   counts["key"] += 1
   ```

10. **Add Better String Support**
    - f-strings: `f"Value: {x}"`
    - String interpolation
    - Better concatenation

### Priority 4: Performance Optimizations

11. **Add Loop Optimizations**
    - Loop unrolling
    - Strength reduction
    - Induction variable elimination

12. **Add SIMD Vectorization**
    - Auto-vectorize simple loops
    - Add explicit SIMD types

13. **Add Escape Analysis**
    - Stack allocate when possible
    - Reduce ARC overhead

---

## Test Cases for Verification

After fixes, these minimal test cases should work:

### Test 1: Basic List Operations
```python
def main():
    nums = [1, 2, 3, 4, 5]
    nums.append(6)
    print(len(nums))  # Should print: 6
```

### Test 2: Floating Point Math
```python
def main():
    x = 4.0
    y = sqrt(x)
    print(y)  # Should print: 2.0
```

### Test 3: String Operations
```python
def main():
    name = "Viper"
    print("Hello, " + name)  # Should print: Hello, Viper
```

### Test 4: Global Constants
```python
LIMIT = 1000

def main():
    print(LIMIT)  # Should print: 1000
```

### Test 5: Struct Types
```python
struct Point:
    x: f64
    y: f64

def main():
    p = Point(1.0, 2.0)
    print(p.x)  # Should print: 1.0
```

---

## Conclusion

The benchmark suite successfully identified **critical bugs** in Viper's compiler that prevent it from running real-world code. The primary issues are:

1. **Code generation bugs** in list/array handling
2. **Type system issues** with floating-point values
3. **Missing standard library** functions
4. **Parser limitations** with certain syntax

**Next Steps:**
1. Fix the code generation bugs in `src/codegen/mod.rs`
2. Implement missing math/string builtins
3. Add global constant support
4. Re-run benchmarks after fixes

**Estimated Effort:**
- Critical bugs: 2-3 weeks
- Standard library: 1-2 weeks
- Language features: 2-3 weeks
- **Total: 5-8 weeks** to run basic benchmarks

---

## Appendix: Benchmark Files

All benchmark source code is available in:
```
/home/stl/viper-lang/benchmark/
├── 01_prime_sieve/
├── 02_fibonacci/
├── 03_matrix_multiply/
├── 04_quicksort/
├── 05_mandelbrot/
├── 06_raytracer/
├── 07_nbody/
├── 08_binary_trees/
├── 09_fannkuch/
├── 10_spectral_norm/
├── 11_k_nucleotide/
├── 12_reverse_complement/
├── 13_regex_dna/
├── 14_champernowne/
└── 15_euler_sum/
```

Each directory contains implementations in:
- `.c` - C source
- `.go` - Go source (not built)
- `.rs` - Rust source
- `.vp` - Viper source (doesn't compile)
