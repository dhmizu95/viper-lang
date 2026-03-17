#!/usr/bin/env python3
"""Large GCD Computation - Tests CPU integer speed and bit operations"""

import time
import random
import math

def euclidean_gcd(a, b):
    """Classic Euclidean GCD algorithm"""
    while b:
        a, b = b, a % b
    return a

def binary_gcd(a, b):
    """
    Binary GCD (Stein's algorithm).
    Uses only subtraction, division by 2, and parity testing.
    """
    if a == 0:
        return b
    if b == 0:
        return a
    
    # Find common factors of 2
    shift = 0
    while ((a | b) & 1) == 0:
        a >>= 1
        b >>= 1
        shift += 1
    
    # Remove remaining factors of 2 from a
    while (a & 1) == 0:
        a >>= 1
    
    while b != 0:
        # Remove factors of 2 from b
        while (b & 1) == 0:
            b >>= 1
        
        # Ensure a <= b
        if a > b:
            a, b = b, a
        
        b -= a
    
    return a << shift

def extended_gcd(a, b):
    """Extended Euclidean algorithm returning (gcd, x, y) where ax + by = gcd"""
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

def generate_coprime_pair(bits):
    """Generate two random coprime numbers of specified bit size"""
    while True:
        a = random.getrandbits(bits)
        b = random.getrandbits(bits)
        if a > 0 and b > 0:
            return a, b

def generate_same_gcd_pair(bits, gcd_bits):
    """Generate two numbers with a specific GCD size"""
    g = random.getrandbits(gcd_bits) | (1 << (gcd_bits - 1))
    a = g * (random.getrandbits(bits - gcd_bits) | 1)
    b = g * (random.getrandbits(bits - gcd_bits) | 1)
    return a, b, g

def main():
    random.seed(42)
    
    print("Large GCD Computation")
    print("-" * 40)
    
    num_tests = 1000
    
    # Test different bit sizes
    bit_sizes = [64, 128, 256, 512, 1024, 2048]
    
    print(f"\nEuclidean GCD ({num_tests} computations each):")
    print(f"{'Bits':<8} {'Total Time':<12} {'Avg (µs)':<10}")
    print("-" * 35)
    
    for bits in bit_sizes:
        pairs = [(random.getrandbits(bits), random.getrandbits(bits)) 
                 for _ in range(num_tests)]
        
        start = time.perf_counter()
        for a, b in pairs:
            _ = euclidean_gcd(a, b)
        elapsed = time.perf_counter() - start
        
        print(f"{bits:<8} {elapsed:<12.4f}s {elapsed/num_tests*1000000:<10.2f}")
    
    # Compare algorithms
    print("\n" + "-" * 40)
    print("Algorithm comparison (1024-bit numbers, 1000 tests):")
    
    pairs = [(random.getrandbits(1024), random.getrandbits(1024)) 
             for _ in range(num_tests)]
    
    # Euclidean
    start = time.perf_counter()
    results_euclid = [euclidean_gcd(a, b) for a, b in pairs]
    time_euclid = time.perf_counter() - start
    
    # Binary (Stein's)
    start = time.perf_counter()
    results_binary = [binary_gcd(a, b) for a, b in pairs]
    time_binary = time.perf_counter() - start
    
    # Python's math.gcd
    start = time.perf_counter()
    results_math = [math.gcd(a, b) for a, b in pairs]
    time_math = time.perf_counter() - start
    
    print(f"  Euclidean:     {time_euclid:.4f}s")
    print(f"  Binary (Stein): {time_binary:.4f}s")
    print(f"  Python math.gcd: {time_math:.4f}s")
    
    # Verify all match
    match = all(results_euclid[i] == results_binary[i] == results_math[i] 
                for i in range(num_tests))
    print(f"\n  All results match: {match}")
    
    # Extended GCD verification
    print("\n" + "-" * 40)
    print("Extended GCD verification (ax + by = gcd):")
    
    for _ in range(5):
        a = random.getrandbits(256)
        b = random.getrandbits(256)
        gcd, x, y = extended_gcd(a, b)
        assert a * x + b * y == gcd, "Extended GCD verification failed"
        print(f"  gcd({a.bit_length()}bit, {b.bit_length()}bit) = {gcd.bit_length()}bit ✓")

if __name__ == "__main__":
    main()
