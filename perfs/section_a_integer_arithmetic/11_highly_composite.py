#!/usr/bin/env python3
"""Highly Composite Number Search - Tests CPU integer speed and bit operations"""

import time
import math

def count_divisors(n):
    """Count the number of divisors of n"""
    count = 0
    sqrt_n = int(math.isqrt(n))
    
    for i in range(1, sqrt_n + 1):
        if n % i == 0:
            count += 1
            if i != n // i:
                count += 1
    
    return count

def count_divisors_prime_factorization(n):
    """
    Count divisors using prime factorization.
    If n = p1^a1 * p2^a2 * ... * pk^ak, then d(n) = (a1+1)(a2+1)...(ak+1)
    """
    if n <= 0:
        return 0
    
    total = 1
    d = 2
    temp = n
    
    while d * d <= temp:
        if temp % d == 0:
            exponent = 0
            while temp % d == 0:
                temp //= d
                exponent += 1
            total *= (exponent + 1)
        d += 1
    
    if temp > 1:
        total *= 2  # Remaining prime factor
    
    return total

def find_highly_composite_numbers(limit):
    """
    Find all highly composite numbers up to limit.
    A highly composite number has more divisors than any smaller positive integer.
    """
    hcn = []
    max_divisors = 0
    
    for n in range(1, limit + 1):
        d = count_divisors_prime_factorization(n)
        if d > max_divisors:
            max_divisors = d
            hcn.append((n, d))
    
    return hcn

def generate_hcn_recursive(primes, max_exponents, current_val, current_divisors, idx, results):
    """Generate candidate HCN using recursive prime exponent assignment"""
    if idx == len(primes):
        results.append((current_val, current_divisors))
        return
    
    prime = primes[idx]
    max_exp = max_exponents[idx] if idx < len(max_exponents) else 10
    
    for exp in range(max_exp + 1):
        generate_hcn_recursive(
            primes, max_exponents,
            current_val * (prime ** exp),
            current_divisors * (exp + 1),
            idx + 1,
            results
        )

def main():
    print("Highly Composite Number Search")
    print("-" * 40)
    
    # Direct search for smaller range
    limit = 100000
    print(f"\nSearching up to {limit:,}:")
    
    start = time.perf_counter()
    hcn_list = find_highly_composite_numbers(limit)
    elapsed = time.perf_counter() - start
    
    print(f"  Found {len(hcn_list)} highly composite numbers")
    print(f"  Time: {elapsed:.3f} seconds")
    print("\n  First 20 HCN:")
    for n, d in hcn_list[:20]:
        print(f"    {n:,} has {d:,} divisors")
    
    # Extended search using prime factorization approach
    print("\n" + "-" * 40)
    print("Extended search using prime generation:")
    
    primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]
    max_exponents = [15, 10, 8, 6, 5, 4, 3, 3, 2, 2, 2, 1, 1, 1, 1]
    
    start = time.perf_counter()
    candidates = []
    generate_hcn_recursive(primes, max_exponents, 1, 1, 0, candidates)
    elapsed = time.perf_counter() - start
    
    # Filter to actual HCN
    candidates.sort()
    hcn_extended = []
    max_div = 0
    for val, div in candidates:
        if div > max_div:
            max_div = div
            hcn_extended.append((val, div))
    
    print(f"  Generated {len(hcn_extended)} HCN candidates")
    print(f"  Time: {elapsed:.3f} seconds")
    print(f"  Largest: {hcn_extended[-1][0]:,} with {hcn_extended[-1][1]:,} divisors")

if __name__ == "__main__":
    main()
