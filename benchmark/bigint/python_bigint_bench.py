# BigInt Comparison Benchmark - Python Version
# Focused comparison of core BigInt operations

import time
import sys

# Increase digit limit for large integer string conversion
sys.set_int_max_str_digits(100000)

def benchmark_factorial(n):
    """Calculate n! and measure time"""
    start = time.perf_counter()
    result = 1
    for i in range(1, n + 1):
        result *= i
    end = time.perf_counter()
    return result, end - start

def benchmark_fibonacci(n):
    """Calculate nth Fibonacci and measure time"""
    start = time.perf_counter()
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    end = time.perf_counter()
    return a, end - start

def benchmark_power(base, exp):
    """Calculate base^exp and measure time"""
    start = time.perf_counter()
    result = base ** exp
    end = time.perf_counter()
    return result, end - start

def benchmark_multiplication(a, b):
    """Multiply two large numbers and measure time"""
    start = time.perf_counter()
    result = a * b
    end = time.perf_counter()
    return result, end - start

def benchmark_division(a, b):
    """Divide two large numbers and measure time"""
    start = time.perf_counter()
    q = a // b
    r = a % b
    end = time.perf_counter()
    return q, r, end - start

def benchmark_mod_pow(base, exp, mod):
    """Calculate (base^exp) % mod and measure time"""
    start = time.perf_counter()
    result = pow(base, exp, mod)
    end = time.perf_counter()
    return result, end - start

def main():
    print("=" * 60)
    print("Python BigInt Benchmark")
    print("=" * 60)
    print()
    
    # Test 1: Factorial
    print("1. Factorial(5000)")
    fact, t = benchmark_factorial(5000)
    print(f"   Result: {len(str(fact))} digits")
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    # Test 2: Fibonacci
    print("2. Fibonacci(50000)")
    fib, t = benchmark_fibonacci(50000)
    print(f"   Result: {len(str(fib))} digits")
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    # Test 3: Power
    print("3. 2^50000")
    pow_result, t = benchmark_power(2, 50000)
    print(f"   Result: {len(str(pow_result))} digits")
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    # Test 4: Large Multiplication
    print("4. Large Multiplication (1000! * 999!)")
    a = benchmark_factorial(1000)[0]
    b = benchmark_factorial(999)[0]
    _, t = benchmark_multiplication(a, b)
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    # Test 5: Large Division
    print("5. Large Division (1000! / 999!)")
    a = benchmark_factorial(1000)[0]
    b = benchmark_factorial(999)[0]
    q, r, t = benchmark_division(a, b)
    print(f"   Quotient: {len(str(q))} digits, Remainder: {r}")
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    # Test 6: Modular Exponentiation
    print("6. Modular Exponentiation (large base ^ large exp % mod)")
    base = 123456789012345678901234567890
    exp = 123456789
    mod = 1000000007
    result, t = benchmark_mod_pow(base, exp, mod)
    print(f"   Result: {result}")
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    # Test 7: Bitwise Operations
    print("7. Bitwise Operations")
    x = (1 << 2048) - 1
    y = (1 << 1024) - 1
    start = time.perf_counter()
    and_r = x & y
    or_r = x | y
    xor_r = x ^ y
    t = time.perf_counter() - start
    print(f"   AND/OR/XOR on 2048-bit numbers")
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    # Test 8: Shift Operations
    print("8. Shift Operations")
    start = time.perf_counter()
    val = 1 << 10000
    val = val >> 5000
    t = time.perf_counter() - start
    print(f"   (1 << 10000) >> 5000")
    print(f"   Result: {len(str(val))} digits")
    print(f"   Time: {t*1000:.2f} ms")
    print()
    
    print("=" * 60)
    print("All benchmarks completed")
    print("=" * 60)

if __name__ == "__main__":
    main()
