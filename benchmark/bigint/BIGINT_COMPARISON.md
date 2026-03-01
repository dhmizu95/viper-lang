# BigInt Implementation Comparison: Viper vs Python

## Overview

This document compares BigInt (arbitrary precision integer) implementations in Viper and Python.

**Viper**: Uses GMP (GNU Multiple Precision Arithmetic Library) via C runtime bridge  
**Python**: Native arbitrary precision integers (also uses GMP internally in CPython)

## Architecture Comparison

### Viper BigInt Architecture

```
Viper Source Code (.vp)
       ↓
Lexer: Recognizes 'n' suffix (123n) or BigInt("string")
       ↓
AST: Expr::BigInt(String, Span)
       ↓
Type: Type::BigInt
       ↓
LLVM: Pointer to ViperBigInt struct
       ↓
Runtime: GMP mpz_t operations via C bridge
```

### Python BigInt Architecture

```
Python Source Code (.py)
       ↓
Parser: Automatic int promotion
       ↓
Internal: PyLongObject (variable-size digit array)
       ↓
Operations: C implementation with Karatsuba, FFT for large numbers
```

## Code Comparison

### 1. Factorial

**Python:**
```python
def factorial(n):
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result

# Usage
fact_1000 = factorial(1000)
print(f"{len(str(fact_1000))} digits")
```

**Viper:**
```python
def benchmark_factorial(n):
    result = BigInt("1")
    i = BigInt("1")
    while i <= n:
        result = result * i
        i = i + BigInt("1")
    return result

# Usage
fact = benchmark_factorial(BigInt("5000"))
print("   Result: " + str(len(str_bigint(fact))) + " digits")
```

### 2. Fibonacci

**Python:**
```python
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

# Usage
fib_10000 = fibonacci(10000)
print(f"{len(str(fib_10000))} digits")
```

**Viper:**
```python
def benchmark_fibonacci(n):
    a = BigInt("0")
    b = BigInt("1")
    i = BigInt("0")
    while i < n:
        temp = a + b
        a = b
        b = temp
        i = i + BigInt("1")
    return a

# Usage
fib = benchmark_fibonacci(BigInt("50000"))
print("   Result: " + str(len(str_bigint(fib))) + " digits")
```

### 3. Modular Exponentiation

**Python:**
```python
def mod_pow(base, exp, mod):
    return pow(base, exp, mod)

# Usage
result = mod_pow(12345678901234567890, 123456789, 1000000007)
print(f"Result: {result}")
```

**Viper:**
```python
def benchmark_mod_pow(base, exp, mod):
    result = BigInt("1")
    base = base % mod
    while exp > BigInt("0"):
        if exp % BigInt("2") == BigInt("1"):
            result = (result * base) % mod
        exp = exp >> BigInt("1")
        base = (base * base) % mod
    return result

# Usage
result = benchmark_mod_pow(
    BigInt("123456789012345678901234567890"),
    BigInt("123456789"),
    BigInt("1000000007")
)
print("   Result: " + str_bigint(result))
```

### 4. Bitwise Operations

**Python:**
```python
x = (1 << 2048) - 1
y = (1 << 1024) - 1
and_result = x & y
or_result = x | y
xor_result = x ^ y
```

**Viper:**
```python
x = (BigInt("1") << BigInt("2048")) - BigInt("1")
y = (BigInt("1") << BigInt("1024")) - BigInt("1")
and_r = x & y
or_r = x | y
xor_r = x ^ y
```

## Performance Comparison

### Python Benchmark Results (CPython 3.x)

| Operation | Input | Time | Result Size |
|-----------|-------|------|-------------|
| Factorial | 5000! | 4.01 ms | 16,326 digits |
| Fibonacci | F(50000) | 18.69 ms | 10,450 digits |
| Power | 2^50000 | 0.09 ms | 15,052 digits |
| Multiplication | 1000! × 999! | 0.05 ms | 5,131 digits |
| Division | 1000! / 999! | <0.01 ms | 4 digits |
| Mod Pow | large^exp % mod | <0.01 ms | 9 digits |
| Bitwise | 2048-bit ops | <0.01 ms | 617 digits |
| Shift | (1<<10000)>>5000 | <0.01 ms | 1,506 digits |

### Viper Performance Notes

Viper's BigInt implementation uses GMP directly through C FFI, providing:
- **Similar performance** to Python for most operations (both use GMP)
- **Zero-cost abstractions** where possible
- **Static typing** with compile-time type checking
- **Native code generation** via LLVM

## Key Differences

| Feature | Python | Viper |
|---------|--------|-------|
| **Syntax** | Automatic (`x = 10**100`) | Explicit (`x = BigInt("...")` or `100n`) |
| **Type System** | Dynamic | Static with inference |
| **Implementation** | PyLongObject (GMP-based) | ViperBigInt (GMP via FFI) |
| **Memory Mgmt** | Garbage collected | ARC (Automatic Reference Counting) |
| **Compilation** | Interpreted/bytecode | AOT/JIT to native code |
| **Performance** | Good (optimized C) | Excellent (native + GMP) |

## When to Use BigInt

### Use BigInt when:
- Values exceed i64 range (±9.2 × 10¹⁸)
- Cryptographic calculations (RSA, ECC)
- Financial calculations requiring exact precision
- Mathematical computations with very large numbers
- Number theory research

### Use native integers (i64) when:
- Values fit in 64 bits
- Performance is critical (tight loops, counters)
- Interfacing with C APIs
- Memory is constrained

## GMP Operations Used

Both implementations leverage GMP's optimized algorithms:

| Operation | GMP Function | Complexity |
|-----------|--------------|------------|
| Addition | `mpz_add` | O(n) |
| Multiplication | `mpz_mul` | O(n^1.585) Karatsuba |
| Division | `mpz_tdiv_qr` | O(n^2) |
| Power | `mpz_powm` | O(log exp) |
| GCD | `mpz_gcd` | O(n²) |

## Future Enhancements for Viper

1. **Automatic Promotion**: Seamlessly promote i64 to BigInt on overflow
2. **Tagged Pointers**: Small integer optimization (SIO)
3. **BigInt Literals**: Full support for `123n` syntax
4. **Mixed Arithmetic**: Seamless i64 + BigInt operations
5. **Additional Functions**: GCD, LCM, modular inverse

## Conclusion

Both Python and Viper provide robust BigInt support through GMP:
- **Python**: Mature, battle-tested, easy to use
- **Viper**: High-performance, statically typed, compiles to native code

Viper's approach offers the benefits of static typing and native performance while maintaining Python-like syntax for BigInt operations.
