#!/usr/bin/env python3
"""Pollard Rho factorization - Tests CPU integer speed and bit operations"""

import time
import math
import random

def gcd(a, b):
    """Euclidean GCD"""
    while b:
        a, b = b, a % b
    return a

def pollard_rho(n, max_iterations=10**7):
    """
    Pollard's rho algorithm for integer factorization.
    Returns a non-trivial factor of n, or None if it fails.
    """
    if n % 2 == 0:
        return 2
    if n == 1:
        return None
    
    x = random.randint(2, n - 1)
    y = x
    c = random.randint(1, n - 1)
    d = 1
    
    f = lambda x: (x * x + c) % n
    
    iterations = 0
    while d == 1 and iterations < max_iterations:
        x = f(x)
        y = f(f(y))
        d = gcd(abs(x - y), n)
        iterations += 1
    
    if d != n and d != 1:
        return d
    return None

def factorize(n):
    """Complete factorization using Pollard rho"""
    if n <= 1:
        return []
    
    factors = []
    
    # Handle factor of 2
    while n % 2 == 0:
        factors.append(2)
        n //= 2
    
    if n == 1:
        return factors
    
    # Check for small primes first
    small_primes = [3, 5, 7, 11, 13, 17, 19, 23, 29, 31]
    for p in small_primes:
        while n % p == 0:
            factors.append(p)
            n //= p
    
    if n == 1:
        return factors
    
    # Use Pollard rho for remaining
    stack = [n]
    while stack:
        num = stack.pop()
        if num == 1:
            continue
        
        # Check if prime (simple test)
        is_prime = True
        for i in range(2, min(1000, int(math.isqrt(num)) + 1)):
            if num % i == 0:
                is_prime = False
                break
        
        if is_prime and num > 1:
            factors.append(num)
            continue
        
        factor = pollard_rho(num)
        if factor and factor != num:
            stack.append(factor)
            stack.append(num // factor)
        else:
            # Failed to factor, add as is
            factors.append(num)
    
    return sorted(factors)

def main():
    # Test numbers - semiprimes and composites
    test_numbers = [
        1000000007 * 1000000009,  # Product of two large primes
        2**64 - 59,               # Large number
        12345678901234567,        # Random large number
        999999999999999989,       # Large prime candidate
    ]
    
    print("Pollard Rho Factorization")
    print("-" * 40)
    
    total_start = time.perf_counter()
    
    for n in test_numbers:
        start = time.perf_counter()
        factors = factorize(n)
        elapsed = time.perf_counter() - start
        
        print(f"\nn = {n:,}")
        print(f"Factors: {factors}")
        print(f"Time: {elapsed:.3f} seconds")
    
    total_elapsed = time.perf_counter() - total_start
    print(f"\nTotal time: {total_elapsed:.3f} seconds")

if __name__ == "__main__":
    main()
