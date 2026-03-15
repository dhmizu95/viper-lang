# Fast Fourier Transform (FFT) Benchmark - Python Implementation
# Cooley-Tukey radix-2 FFT algorithm

import math
import cmath

N = 256  # FFT size (must be power of 2)
PI = math.pi

def bit_reverse(n, bits):
    reversed_val = 0
    for _ in range(bits):
        reversed_val = (reversed_val << 1) | (n & 1)
        n >>= 1
    return reversed_val

def fft(x, inverse=False):
    n = len(x)
    if n <= 1:
        return x
    
    bits = int(math.log2(n))
    
    # Bit-reversal permutation
    for i in range(n):
        rev = bit_reverse(i, bits)
        if i < rev:
            x[i], x[rev] = x[rev], x[i]
    
    # Cooley-Tukey iterative FFT
    length = 2
    while length <= n:
        angle = 2 * PI / length * (-1 if inverse else 1)
        wlen = complex(math.cos(angle), math.sin(angle))
        
        i = 0
        while i < n:
            w = 1 + 0j
            for j in range(length // 2):
                u = x[i + j]
                v = x[i + j + length//2] * w
                x[i + j] = u + v
                x[i + j + length//2] = u - v
                w *= wlen
            i += length
        length <<= 1
    
    # Scale for inverse FFT
    if inverse:
        for i in range(n):
            x[i] /= n
    
    return x

def generate_signal(n):
    freq1 = 2.0  # 2 Hz
    freq2 = 8.0  # 8 Hz
    sample_rate = 64.0
    
    signal = []
    for i in range(n):
        t = i / sample_rate
        real = math.sin(2 * PI * freq1 * t) + 0.5 * math.sin(2 * PI * freq2 * t)
        signal.append(complex(real, 0.0))
    return signal

def total_magnitude(spectrum):
    total = 0.0
    for c in spectrum:
        total += abs(c)
    return total

def main():
    # Generate test signal
    signal = generate_signal(N)
    
    # Perform FFT
    fft(signal)
    
    # Calculate and print result
    magnitude = total_magnitude(signal)
    print(f"{magnitude:.6f}")
    
    # Perform inverse FFT to verify
    fft(signal, inverse=True)
    
    # Print first sample of reconstructed signal (verification)
    print(f"{signal[0].real:.6f}")

if __name__ == "__main__":
    main()
