#!/usr/bin/env python3
"""Extended Euclidean Algorithm - Tests CPU integer speed and bit operations"""

import time
import random

def extended_gcd(a, b):
    """
    Extended Euclidean Algorithm.
    Returns (gcd, x, y) such that a*x + b*y = gcd(a, b)
    """
    if b == 0:
        return a, 1, 0
    
    old_r, r = a, b
    old_s, s = 1, 0
    old_t, t = 0, 1
    
    while r != 0:
        quotient = old_r // r
        old_r, r = r, old_r - quotient * r
        old_s, s = s, old_s - quotient * s
        old_t, t = t, old_t - quotient * t
    
    return old_r, old_s, old_t

def modular_inverse(a, m):
    """
    Find modular inverse of a mod m.
    Returns x such that (a * x) % m == 1
    """
    gcd, x, _ = extended_gcd(a % m, m)
    if gcd != 1:
        return None  # Inverse doesn't exist
    return x % m

def main():
    random.seed(42)
    
    # Generate test cases with large numbers
    test_cases = []
    for _ in range(10000):
        a = random.randint(10**15, 10**18)
        b = random.randint(10**15, 10**18)
        test_cases.append((a, b))
    
    print("Extended Euclidean Algorithm")
    print("-" * 40)
    print(f"Running {len(test_cases)} iterations with 64-bit integers")
    
    start = time.perf_counter()
    
    results = []
    for a, b in test_cases:
        gcd, x, y = extended_gcd(a, b)
        # Verify: a*x + b*y = gcd
        assert a * x + b * y == gcd, "Extended GCD verification failed"
        results.append((gcd, x, y))
    
    elapsed = time.perf_counter() - start
    
    print(f"Completed: {len(results)} extended GCD computations")
    print(f"Time: {elapsed:.3f} seconds")
    print(f"Average per computation: {elapsed/len(test_cases)*1000:.4f} ms")
    
    # Show a sample result
    print(f"\nSample: gcd({test_cases[0][0]:,}, {test_cases[0][1]:,}) = {results[0][0]:,}")

if __name__ == "__main__":
    main()
