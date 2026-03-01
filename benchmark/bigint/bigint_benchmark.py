# BigInt Benchmark Suite for Python
# Comprehensive arbitrary precision integer operations

import time

# ============= Factorial =============
def factorial(n):
    """Calculate n! using Python's native BigInt"""
    result = 1
    for i in range(1, n + 1):
        result *= i
    return result

# ============= Fibonacci =============
def fibonacci(n):
    """Calculate nth Fibonacci number using Python's native BigInt"""
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a

# ============= Power =============
def power(base, exp):
    """Calculate base^exp using Python's native BigInt"""
    return base ** exp

# ============= GCD =============
def gcd(a, b):
    """Calculate GCD using Euclidean algorithm"""
    while b != 0:
        a, b = b, a % b
    return a

# ============= Prime Check =============
def is_prime(n):
    """Simple primality test"""
    if n < 2:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False
    
    i = 3
    while i * i <= n:
        if n % i == 0:
            return False
        i += 2
    return True

# ============= Modular Exponentiation =============
def mod_pow(base, exp, mod):
    """Calculate (base^exp) % mod efficiently"""
    return pow(base, exp, mod)

# ============= Sum of Digits =============
def sum_of_digits(n):
    """Sum all digits of a BigInt"""
    return sum(int(d) for d in str(n))

# ============= Main Benchmark =============
def main():
    print("=== Python BigInt Benchmark Suite ===")
    print("")
    
    # Benchmark 1: Factorial
    print("Benchmark 1: Factorial(1000)")
    start = time.perf_counter()
    fact_1000 = factorial(1000)
    fact_str = str(fact_1000)
    end = time.perf_counter()
    print(f"  Result has {len(fact_str)} digits")
    print(f"  First 50 digits: {fact_str[:50]}")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 2: Fibonacci
    print("Benchmark 2: Fibonacci(10000)")
    start = time.perf_counter()
    fib_10000 = fibonacci(10000)
    fib_str = str(fib_10000)
    end = time.perf_counter()
    print(f"  Result has {len(fib_str)} digits")
    print(f"  First 50 digits: {fib_str[:50]}")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 3: Large Power
    print("Benchmark 3: 2^10000")
    start = time.perf_counter()
    pow_result = power(2, 10000)
    pow_str = str(pow_result)
    end = time.perf_counter()
    print(f"  Result has {len(pow_str)} digits")
    print(f"  First 50 digits: {pow_str[:50]}")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 4: GCD
    print("Benchmark 4: GCD of two large numbers")
    start = time.perf_counter()
    a = factorial(500)
    b = factorial(499)
    g = gcd(a, b)
    end = time.perf_counter()
    print(f"  GCD(500!, 499!) has {len(str(g))} digits")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 5: Modular Exponentiation
    print("Benchmark 5: Modular Exponentiation")
    start = time.perf_counter()
    base = 12345678901234567890
    exp = 987654321
    mod = 1000000007
    mod_result = mod_pow(base, exp, mod)
    end = time.perf_counter()
    print(f"  (12345678901234567890 ^ 987654321) % 1000000007")
    print(f"  Result: {mod_result}")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 6: Prime Check
    print("Benchmark 6: Prime Check")
    start = time.perf_counter()
    large_prime = 1234567890123456789012345678901234567891
    result = is_prime(large_prime)
    end = time.perf_counter()
    if result:
        print(f"  {large_prime} is prime")
    else:
        print(f"  {large_prime} is composite")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 7: Sum of Digits
    print("Benchmark 7: Sum of digits of 1000!")
    start = time.perf_counter()
    digit_sum = sum_of_digits(fact_1000)
    end = time.perf_counter()
    print(f"  Sum of digits: {digit_sum}")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 8: Bitwise Operations
    print("Benchmark 8: Bitwise Operations")
    start = time.perf_counter()
    x = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    y = 0x0000000000000000FFFFFFFFFFFFFFFF
    and_result = x & y
    or_result = x | y
    xor_result = x ^ y
    end = time.perf_counter()
    print(f"  x & y = {and_result}")
    print(f"  x | y = {or_result}")
    print(f"  x ^ y = {xor_result}")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 9: Shift Operations
    print("Benchmark 9: Shift Operations")
    start = time.perf_counter()
    one = 1
    shifted_left = one << 1000
    print(f"  1 << 1000 has {len(str(shifted_left))} digits")
    shifted_right = shifted_left >> 500
    print(f"  (1 << 1000) >> 500 has {len(str(shifted_right))} digits")
    end = time.perf_counter()
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    # Benchmark 10: Large Multiplication
    print("Benchmark 10: Large Multiplication")
    start = time.perf_counter()
    num1 = factorial(300)
    num2 = factorial(200)
    product = num1 * num2
    end = time.perf_counter()
    print(f"  300! * 200! has {len(str(product))} digits")
    print(f"  Time: {end - start:.4f}s")
    print("")
    
    print("=== All benchmarks completed ===")


if __name__ == "__main__":
    main()
