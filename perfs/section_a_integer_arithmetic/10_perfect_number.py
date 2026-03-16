#!/usr/bin/env python3
"""Perfect Number Search - Tests CPU integer speed and bit operations"""

import time
import math

def sum_of_divisors(n):
    """Compute sum of all proper divisors of n (excluding n itself)"""
    if n <= 1:
        return 0
    
    total = 1  # 1 is always a divisor
    sqrt_n = int(math.isqrt(n))
    
    for i in range(2, sqrt_n + 1):
        if n % i == 0:
            total += i
            if i != n // i:  # Don't count square root twice
                total += n // i
    
    return total

def is_perfect(n):
    """Check if n is a perfect number"""
    return sum_of_divisors(n) == n

def find_perfect_numbers(limit):
    """Find all perfect numbers up to limit"""
    perfect_numbers = []
    
    for n in range(2, limit + 1):
        if is_perfect(n):
            perfect_numbers.append(n)
    
    return perfect_numbers

def perfect_from_mersenne(p):
    """
    Generate even perfect number from Mersenne prime exponent.
    If 2^p - 1 is prime, then 2^(p-1) * (2^p - 1) is perfect.
    """
    mersenne = (1 << p) - 1
    # Quick primality check for Mersenne
    if mersenne < 2:
        return None
    
    # Lucas-Lehmer test for Mersenne primes
    if p == 2:
        return (1 << (p - 1)) * mersenne
    
    s = 4
    for _ in range(p - 2):
        s = (s * s - 2) % mersenne
    
    if s == 0:
        return (1 << (p - 1)) * mersenne
    return None

def main():
    print("Perfect Number Search")
    print("-" * 40)
    
    # Search using divisor sum (slow but straightforward)
    limit = 10000
    print(f"\nSearching up to {limit:,} using divisor sum:")
    
    start = time.perf_counter()
    perfect_nums = find_perfect_numbers(limit)
    elapsed = time.perf_counter() - start
    
    print(f"  Perfect numbers found: {perfect_nums}")
    print(f"  Time: {elapsed:.4f} seconds")
    
    # Generate using Mersenne primes (much faster for large perfect numbers)
    print("\nGenerating from Mersenne primes:")
    
    mersenne_exponents = [2, 3, 5, 7, 13, 17, 19, 31]
    
    start = time.perf_counter()
    large_perfect = []
    for p in mersenne_exponents:
        perfect = perfect_from_mersenne(p)
        if perfect:
            large_perfect.append(perfect)
            digits = len(str(perfect))
            print(f"  p={p}: {digits:,} digits")
    elapsed = time.perf_counter() - start
    
    print(f"\nTotal time: {elapsed:.4f} seconds")
    print(f"Largest perfect number has {len(str(large_perfect[-1])):,} digits")

if __name__ == "__main__":
    main()
