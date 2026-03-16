#!/usr/bin/env python3
"""Euler Totient Function for 10^9 numbers - Tests CPU integer speed and bit operations"""

import time
import math

def euler_totient(n):
    """
    Compute Euler's totient function φ(n).
    Returns count of integers from 1 to n coprime to n.
    """
    result = n
    p = 2
    while p * p <= n:
        if n % p == 0:
            while n % p == 0:
                n //= p
            result -= result // p
        p += 1
    if n > 1:
        result -= result // n
    return result

def euler_totient_sieve(limit):
    """
    Compute φ(n) for all n from 1 to limit using a sieve.
    Much faster for computing many totients.
    """
    phi = list(range(limit + 1))
    
    for i in range(2, limit + 1):
        if phi[i] == i:  # i is prime
            for j in range(i, limit + 1, i):
                phi[j] -= phi[j] // i
    
    return phi

def main():
    print("Euler Totient Function")
    print("-" * 40)
    
    # Single large computation
    n = 10**9
    print(f"\nφ({n:,}) - Single computation")
    
    start = time.perf_counter()
    result = euler_totient(n)
    elapsed = time.perf_counter() - start
    
    print(f"  Result: {result:,}")
    print(f"  Time: {elapsed:.4f} seconds")
    
    # Sieve for smaller range
    limit = 10**7
    print(f"\nφ(1) to φ({limit:,}) - Sieve computation")
    
    start = time.perf_counter()
    phi_values = euler_totient_sieve(limit)
    elapsed = time.perf_counter() - start
    
    print(f"  Time: {elapsed:.3f} seconds")
    print(f"  Sum of φ(1) to φ({limit:,}): {sum(phi_values[1:]):,}")
    
    # Verify a few values
    print("\nSample values:")
    for i in [1, 2, 10, 100, 1000, 10000]:
        print(f"  φ({i:,}) = {phi_values[i]:,}")

if __name__ == "__main__":
    main()
