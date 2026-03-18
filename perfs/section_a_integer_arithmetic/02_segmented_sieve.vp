#!/usr/bin/env python3
"""Segmented sieve up to 10^12 - Tests CPU integer speed and bit operations"""

import time
import math

def simple_sieve(limit):
    """Generate primes up to limit using simple sieve"""
    sieve = bytearray([1]) * (limit + 1)
    sieve[0:2] = b'\x00\x00'
    
    for i in range(2, int(math.isqrt(limit)) + 1):
        if sieve[i]:
            sieve[i*i:limit+1:i] = bytearray([0]) * len(sieve[i*i:limit+1:i])
    
    return [i for i, is_prime in enumerate(sieve) if is_prime]

def segmented_sieve(low, high, small_primes):
    """Segmented sieve for range [low, high]"""
    segment_size = high - low + 1
    is_prime = bytearray([1]) * segment_size
    
    for p in small_primes:
        start = max(p * p, ((low + p - 1) // p) * p)
        for j in range(start, high + 1, p):
            is_prime[j - low] = 0
    
    if low == 0:
        is_prime[0:2] = b'\x00\x00'
    elif low == 1:
        is_prime[0] = 0
    
    return [low + i for i, prime in enumerate(is_prime) if prime]

def main():
    low = 10**11
    high = 10**11 + 10**7  # Segment of 10 million starting at 10^11
    
    print(f"Segmented Sieve from {low:,} to {high:,}")
    print("-" * 40)
    
    start = time.perf_counter()
    
    # Generate small primes up to sqrt(high)
    sqrt_high = int(math.isqrt(high)) + 1
    small_primes = simple_sieve(sqrt_high)
    
    # Use segmented sieve for the large range
    primes = segmented_sieve(low, high, small_primes)
    
    elapsed = time.perf_counter() - start
    
    print(f"Primes found in segment: {len(primes):,}")
    print(f"Time: {elapsed:.3f} seconds")
    if primes:
        print(f"First prime: {primes[0]:,}")
        print(f"Last prime: {primes[-1]:,}")

if __name__ == "__main__":
    main()
