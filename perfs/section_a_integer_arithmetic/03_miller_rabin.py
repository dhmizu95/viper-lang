#!/usr/bin/env python3
"""Miller-Rabin primality test - Tests CPU integer speed and bit operations"""

import time
import random

def miller_rabin(n, witnesses=None):
    """
    Miller-Rabin primality test.
    For n < 3,317,044,064,679,887,385,961,981, these witnesses are sufficient:
    [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    """
    if n < 2:
        return False
    if n == 2 or n == 3:
        return True
    if n % 2 == 0:
        return False
    
    # Write n-1 as 2^r * d
    r, d = 0, n - 1
    while d % 2 == 0:
        r += 1
        d //= 2
    
    if witnesses is None:
        witnesses = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
    
    for a in witnesses:
        if a >= n:
            continue
        
        x = pow(a, d, n)
        
        if x == 1 or x == n - 1:
            continue
        
        for _ in range(r - 1):
            x = pow(x, 2, n)
            if x == n - 1:
                break
        else:
            return False
    
    return True

def main():
    # Test a range of large numbers
    test_numbers = []
    random.seed(42)
    
    for _ in range(1000):
        # Generate random 64-bit numbers
        test_numbers.append(random.randint(2**60, 2**64 - 1))
    
    print("Miller-Rabin Primality Test")
    print("-" * 40)
    print(f"Testing {len(test_numbers)} random 64-bit integers")
    
    start = time.perf_counter()
    
    primes_found = 0
    for n in test_numbers:
        if miller_rabin(n):
            primes_found += 1
    
    elapsed = time.perf_counter() - start
    
    print(f"Primes found: {primes_found}")
    print(f"Time: {elapsed:.3f} seconds")
    print(f"Average per number: {elapsed/len(test_numbers)*1000:.3f} ms")

if __name__ == "__main__":
    main()
