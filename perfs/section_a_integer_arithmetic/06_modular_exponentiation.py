#!/usr/bin/env python3
"""Modular Exponentiation (10^12 exponent) - Tests CPU integer speed and bit operations"""

import time

def mod_pow(base, exp, mod):
    """
    Modular exponentiation using binary method.
    Computes (base^exp) % mod efficiently.
    """
    result = 1
    base = base % mod
    
    while exp > 0:
        if exp & 1:
            result = (result * base) % mod
        exp >>= 1
        base = (base * base) % mod
    
    return result

def mod_pow_builtin(base, exp, mod):
    """Python's built-in modular exponentiation"""
    return pow(base, exp, mod)

def main():
    # Test with very large exponent (10^12)
    base = 123456789
    exp = 10**12
    mod = 10**9 + 7
    
    print("Modular Exponentiation Benchmark")
    print("-" * 40)
    print(f"Computing {base}^{exp:,} mod {mod:,}")
    print()
    
    # Custom implementation
    start = time.perf_counter()
    result_custom = mod_pow(base, exp, mod)
    time_custom = time.perf_counter() - start
    
    print(f"Custom implementation:")
    print(f"  Result: {result_custom:,}")
    print(f"  Time: {time_custom:.4f} seconds")
    
    # Built-in implementation
    start = time.perf_counter()
    result_builtin = pow(base, exp, mod)
    time_builtin = time.perf_counter() - start
    
    print(f"\nPython built-in pow():")
    print(f"  Result: {result_builtin:,}")
    print(f"  Time: {time_builtin:.4f} seconds")
    
    print(f"\nSpeedup (builtin vs custom): {time_custom/time_builtin:.2f}x")
    
    # Stress test with multiple computations
    print("\n" + "-" * 40)
    print("Stress test: 10000 modular exponentiations")
    
    start = time.perf_counter()
    for i in range(10000):
        pow(base + i, exp, mod)
    elapsed = time.perf_counter() - start
    
    print(f"Time: {elapsed:.3f} seconds")
    print(f"Average: {elapsed/10000*1000:.4f} ms per operation")

if __name__ == "__main__":
    main()
