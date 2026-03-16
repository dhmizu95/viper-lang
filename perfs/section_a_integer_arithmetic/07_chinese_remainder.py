#!/usr/bin/env python3
"""Chinese Remainder Theorem Solver - Tests CPU integer speed and bit operations"""

import time
import random

def extended_gcd(a, b):
    """Extended Euclidean Algorithm"""
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

def mod_inverse(a, m):
    """Modular multiplicative inverse"""
    gcd, x, _ = extended_gcd(a % m, m)
    if gcd != 1:
        return None
    return x % m

def chinese_remainder_theorem(remainders, moduli):
    """
    Chinese Remainder Theorem solver.
    Given remainders r_i and moduli m_i (pairwise coprime),
    finds x such that x ≡ r_i (mod m_i) for all i.
    """
    n = len(remainders)
    assert n == len(moduli), "Remainders and moduli must have same length"
    
    # Compute product of all moduli
    M = 1
    for m in moduli:
        M *= m
    
    result = 0
    for i in range(n):
        m_i = moduli[i]
        M_i = M // m_i
        y_i = mod_inverse(M_i, m_i)
        if y_i is None:
            raise ValueError(f"Moduli are not pairwise coprime: {moduli}")
        result += remainders[i] * M_i * y_i
    
    return result % M

def generate_coprime_moduli(count, min_val=1000, max_val=10000):
    """Generate a list of pairwise coprime moduli"""
    random.seed(42)
    moduli = []
    attempts = 0
    
    while len(moduli) < count and attempts < count * 100:
        candidate = random.randint(min_val, max_val)
        # Check if candidate is coprime with all existing moduli
        is_coprime = True
        for m in moduli:
            a, b = m, candidate
            while b:
                a, b = b, a % b
            if a != 1:
                is_coprime = False
                break
        if is_coprime:
            moduli.append(candidate)
        attempts += 1
    
    return moduli

def main():
    print("Chinese Remainder Theorem Solver")
    print("-" * 40)
    
    # Test with different sizes
    test_sizes = [5, 10, 20, 50]
    
    for size in test_sizes:
        moduli = generate_coprime_moduli(size)
        remainders = [random.randint(0, m - 1) for m in moduli]
        
        start = time.perf_counter()
        result = chinese_remainder_theorem(remainders, moduli)
        elapsed = time.perf_counter() - start
        
        # Verify solution
        valid = all(result % moduli[i] == remainders[i] for i in range(size))
        
        print(f"\n{size} congruences:")
        print(f"  Product of moduli: {eval('*'.join(map(str, moduli))):,}")
        print(f"  Solution: {result:,}")
        print(f"  Valid: {valid}")
        print(f"  Time: {elapsed:.4f} seconds")

if __name__ == "__main__":
    main()
