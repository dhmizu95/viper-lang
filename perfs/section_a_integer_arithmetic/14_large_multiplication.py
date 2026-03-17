#!/usr/bin/env python3
"""Random 1024-bit Multiplication - Tests CPU integer speed and bit operations"""

import time
import random

def multiply_1024bit(a, b):
    """Multiply two 1024-bit numbers"""
    return a * b

def karatsuba_multiply(x, y):
    """
    Karatsuba multiplication for large numbers.
    Python uses this internally, but we implement it for comparison.
    """
    if x < 10 or y < 10:
        return x * y
    
    n = max(x.bit_length(), y.bit_length())
    m = n // 2
    
    # Split the numbers
    high1, low1 = x >> m, x & ((1 << m) - 1)
    high2, low2 = y >> m, y & ((1 << m) - 1)
    
    # Three recursive multiplications
    z0 = karatsuba_multiply(low1, low2)
    z2 = karatsuba_multiply(high1, high2)
    z1 = karatsuba_multiply(low1 + high1, low2 + high2) - z2 - z0
    
    return (z2 << (2 * m)) + (z1 << m) + z0

def generate_random_bits(n):
    """Generate a random n-bit number"""
    return random.getrandbits(n)

def main():
    random.seed(42)
    
    print("Large Integer Multiplication (1024-bit)")
    print("-" * 40)
    
    # Generate test pairs
    num_tests = 1000
    test_pairs = [(generate_random_bits(1024), generate_random_bits(1024)) 
                  for _ in range(num_tests)]
    
    # Python's built-in multiplication
    print(f"\nPython built-in multiplication ({num_tests} operations):")
    start = time.perf_counter()
    results_builtin = [a * b for a, b in test_pairs]
    time_builtin = time.perf_counter() - start
    
    print(f"  Total time: {time_builtin:.4f} seconds")
    print(f"  Average: {time_builtin/num_tests*1000:.4f} ms")
    print(f"  Throughput: {num_tests/time_builtin:.0f} mults/sec")
    
    # Show sample result
    sample_result = results_builtin[0]
    print(f"\n  Sample result: {len(str(sample_result))} digits")
    
    # Karatsuba implementation (slower, for comparison)
    print(f"\nKaratsuba implementation (100 operations):")
    start = time.perf_counter()
    results_karatsuba = [karatsuba_multiply(a, b) for a, b in test_pairs[:100]]
    time_karatsuba = time.perf_counter() - start
    
    print(f"  Total time: {time_karatsuba:.4f} seconds")
    print(f"  Average: {time_karatsuba/100*1000:.4f} ms")
    
    # Verify correctness
    match = all(results_builtin[i] == results_karatsuba[i] for i in range(100))
    print(f"  Results match: {match}")
    
    # Different bit sizes
    print("\n" + "-" * 40)
    print("Performance by bit size (1000 multiplications each):")
    
    bit_sizes = [64, 128, 256, 512, 1024, 2048, 4096]
    
    for bits in bit_sizes:
        pairs = [(generate_random_bits(bits), generate_random_bits(bits)) 
                 for _ in range(1000)]
        
        start = time.perf_counter()
        for a, b in pairs:
            _ = a * b
        elapsed = time.perf_counter() - start
        
        print(f"  {bits} bits: {elapsed:.4f}s ({elapsed:.2f} µs/mult)")

if __name__ == "__main__":
    main()
