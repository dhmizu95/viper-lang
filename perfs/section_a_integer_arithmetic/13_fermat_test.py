#!/usr/bin/env python3
"""Fermat Primality Test Stress - Tests CPU integer speed and bit operations"""

import time
import random

def fermat_test(n, k=10):
    """
    Fermat primality test with k iterations.
    Returns True if probably prime, False if definitely composite.
    """
    if n < 2:
        return False
    if n == 2 or n == 3:
        return True
    if n % 2 == 0:
        return False
    
    for _ in range(k):
        a = random.randint(2, n - 2)
        if pow(a, n - 1, n) != 1:
            return False
    
    return True

def find_carmichael_numbers(limit):
    """
    Find Carmichael numbers up to limit.
    These are composite numbers that pass Fermat test for all bases coprime to n.
    """
    def is_prime(n):
        if n < 2:
            return False
        if n == 2:
            return True
        if n % 2 == 0:
            return False
        for i in range(3, int(n**0.5) + 1, 2):
            if n % i == 0:
                return False
        return True
    
    def is_carmichael(n):
        if is_prime(n):
            return False
        
        # Check Fermat condition for all bases
        for a in range(2, n):
            if math.gcd(a, n) == 1:
                if pow(a, n - 1, n) != 1:
                    return False
        return True
    
    import math
    carmichael = []
    for n in range(2, limit + 1):
        if is_carmichael(n):
            carmichael.append(n)
    
    return carmichael

def stress_test():
    """Stress test with many random numbers"""
    random.seed(42)
    
    test_count = 10000
    bit_sizes = [32, 64, 128, 256]
    
    results = {}
    
    for bits in bit_sizes:
        min_val = 1 << (bits - 1)
        max_val = (1 << bits) - 1
        
        numbers = [random.randint(min_val, max_val) for _ in range(test_count)]
        
        start = time.perf_counter()
        primes_found = sum(1 for n in numbers if fermat_test(n, k=5))
        elapsed = time.perf_counter() - start
        
        results[bits] = {
            'primes': primes_found,
            'time': elapsed,
            'per_test': elapsed / test_count * 1000
        }
    
    return results

def main():
    print("Fermat Primality Test Stress")
    print("-" * 40)
    
    # Stress test results
    results = stress_test()
    
    print("\nStress test results (10,000 numbers each):")
    print(f"{'Bits':<8} {'Primes Found':<15} {'Total Time':<12} {'Avg (ms)':<10}")
    print("-" * 45)
    
    for bits, data in results.items():
        print(f"{bits:<8} {data['primes']:<15} {data['time']:<12.3f}s {data['per_test']:<10.4f}")
    
    # Demonstrate Carmichael numbers (Fermat liars)
    print("\n" + "-" * 40)
    print("Carmichael numbers (pass Fermat test but are composite):")
    
    # First few Carmichael numbers
    carmichael = [561, 1105, 1729, 2465, 2821, 6601, 8911, 10585, 15841, 29341]
    
    for n in carmichael[:10]:
        # Test with different bases
        passes = all(pow(a, n - 1, n) == 1 for a in range(2, min(n, 100)) if math.gcd(a, n) == 1)
        print(f"  {n:,}: Fermat passes = {passes}")
    
    import math

if __name__ == "__main__":
    main()
