#!/usr/bin/env python3
"""Large Binomial Coefficients C(100000, 50000) - Tests CPU integer speed and bit operations"""

import time
import math

def binomial_coefficient(n, k):
    """
    Compute C(n, k) = n! / (k! * (n-k)!)
    Uses the multiplicative formula for efficiency.
    """
    if k < 0 or k > n:
        return 0
    if k == 0 or k == n:
        return 1
    
    # Use symmetry: C(n, k) = C(n, n-k)
    k = min(k, n - k)
    
    result = 1
    for i in range(k):
        result = result * (n - i) // (i + 1)
    
    return result

def binomial_with_powers(n, k):
    """
    Compute binomial coefficient using prime factorization.
    More efficient for very large values.
    """
    if k < 0 or k > n:
        return 0
    if k == 0 or k == n:
        return 1
    
    k = min(k, n - k)
    
    # Sieve to find primes up to n
    sieve = bytearray([1]) * (n + 1)
    sieve[0:2] = b'\x00\x00'
    for i in range(2, int(math.isqrt(n)) + 1):
        if sieve[i]:
            sieve[i*i:n+1:i] = bytearray([0]) * len(sieve[i*i:n+1:i])
    
    primes = [i for i, is_prime in enumerate(sieve) if is_prime]
    
    # Legendre's formula: count prime powers in n!
    def count_prime_in_factorial(p, n):
        count = 0
        while n > 0:
            n //= p
            count += n
        return count
    
    result = 1
    for p in primes:
        if p > n:
            break
        # Exponent of p in C(n, k)
        exp = count_prime_in_factorial(p, n) - count_prime_in_factorial(p, k) - count_prime_in_factorial(p, n - k)
        if exp > 0:
            result *= p ** exp
    
    return result

def main():
    print("Large Binomial Coefficients")
    print("-" * 40)
    
    test_cases = [
        (1000, 500),
        (10000, 5000),
        (100000, 50000),
    ]
    
    for n, k in test_cases:
        print(f"\nC({n:,}, {k:,})")
        
        # Multiplicative method
        start = time.perf_counter()
        result1 = binomial_coefficient(n, k)
        time1 = time.perf_counter() - start
        
        digits = len(str(result1))
        print(f"  Multiplicative method: {digits:,} digits in {time1:.4f} seconds")
        
        # Prime factorization method (only for smaller cases)
        if n <= 10000:
            start = time.perf_counter()
            result2 = binomial_with_powers(n, k)
            time2 = time.perf_counter() - start
            print(f"  Prime factorization: {time2:.4f} seconds")
            print(f"  Results match: {result1 == result2}")

if __name__ == "__main__":
    main()
