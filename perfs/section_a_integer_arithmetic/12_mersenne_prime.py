#!/usr/bin/env python3
"""Mersenne Prime Testing - Tests CPU integer speed and bit operations"""

import time

def lucas_lehmer(p):
    """
    Lucas-Lehmer test for Mersenne primes.
    Tests if M_p = 2^p - 1 is prime.
    """
    if p == 2:
        return True
    
    m_p = (1 << p) - 1
    s = 4
    
    for _ in range(p - 2):
        s = (s * s - 2) % m_p
    
    return s == 0

def find_mersenne_primes(max_p):
    """Find all Mersenne primes for exponents up to max_p"""
    mersenne_primes = []
    
    # Only test prime exponents
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
    
    for p in range(2, max_p + 1):
        if is_prime(p):
            if lucas_lehmer(p):
                mersenne_primes.append(p)
    
    return mersenne_primes

def main():
    print("Mersenne Prime Testing (Lucas-Lehmer)")
    print("-" * 40)
    
    # Test known Mersenne prime exponents
    known_exponents = [2, 3, 5, 7, 13, 17, 19, 31]
    
    print("\nTesting known Mersenne prime exponents:")
    
    for p in known_exponents:
        start = time.perf_counter()
        is_mersenne_prime = lucas_lehmer(p)
        elapsed = time.perf_counter() - start
        
        m_p = (1 << p) - 1
        digits = len(str(m_p))
        status = "✓ Prime" if is_mersenne_prime else "✗ Composite"
        print(f"  M_{p}: {digits} digits - {status} ({elapsed:.4f}s)")
    
    # Search for Mersenne primes up to a limit
    print("\n" + "-" * 40)
    print("Searching for Mersenne primes (p ≤ 100):")
    
    start = time.perf_counter()
    mersenne_primes = find_mersenne_primes(100)
    elapsed = time.perf_counter() - start
    
    print(f"  Exponents found: {mersenne_primes}")
    print(f"  Time: {elapsed:.3f} seconds")
    
    # Show the Mersenne primes
    print("\n  Mersenne primes M_p = 2^p - 1:")
    for p in mersenne_primes:
        m_p = (1 << p) - 1
        digits = len(str(m_p))
        print(f"    p={p}: {digits} digits")

if __name__ == "__main__":
    main()
