/*
 * FFT (Fast Fourier Transform) - Signal Processing Benchmark
 * Tests recursion, floating-point math, and array operations
 * Uses 1M samples
 */
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>

#define N 1048576  // 2^20 = 1M samples

typedef struct {
    double real;
    double imag;
} Complex;

static Complex data[N];
static int bit_reverse[N];

void precompute_bit_reverse() {
    int bits = 20;  // log2(N)
    for (int i = 0; i < N; i++) {
        int rev = 0;
        for (int j = 0; j < bits; j++) {
            rev = (rev << 1) | ((i >> j) & 1);
        }
        bit_reverse[i] = rev;
    }
}

void fft(Complex *x, int n) {
    // Bit-reversal permutation
    for (int i = 0; i < n; i++) {
        if (i < bit_reverse[i]) {
            Complex temp = x[i];
            x[i] = x[bit_reverse[i]];
            x[bit_reverse[i]] = temp;
        }
    }
    
    // Cooley-Tukey FFT
    for (int len = 2; len <= n; len *= 2) {
        double angle = -2.0 * M_PI / len;
        Complex wlen = {cos(angle), sin(angle)};
        
        for (int i = 0; i < n; i += len) {
            Complex w = {1.0, 0.0};
            for (int j = 0; j < len / 2; j++) {
                Complex u = x[i + j];
                Complex v = {
                    w.real * x[i + j + len/2].real - w.imag * x[i + j + len/2].imag,
                    w.real * x[i + j + len/2].imag + w.imag * x[i + j + len/2].real
                };
                x[i + j].real = u.real + v.real;
                x[i + j].imag = u.imag + v.imag;
                x[i + j + len/2].real = u.real - v.real;
                x[i + j + len/2].imag = u.imag - v.imag;
                
                Complex temp = {
                    w.real * wlen.real - w.imag * wlen.imag,
                    w.real * wlen.imag + w.imag * wlen.real
                };
                w = temp;
            }
        }
    }
}

int main() {
    printf("Computing FFT with %d samples...\n", N);
    
    precompute_bit_reverse();
    
    // Initialize with test signal
    for (int i = 0; i < N; i++) {
        double t = (double)i / N;
        data[i].real = sin(2.0 * M_PI * 10.0 * t) + 0.5 * sin(2.0 * M_PI * 25.0 * t);
        data[i].imag = 0.0;
    }
    
    clock_t start = clock();
    
    fft(data, N);
    
    clock_t end = clock();
    double elapsed = (double)(end - start) / CLOCKS_PER_SEC;
    
    // Compute magnitude spectrum checksum
    double checksum = 0.0;
    for (int i = 0; i < N; i++) {
        checksum += sqrt(data[i].real * data[i].real + data[i].imag * data[i].imag);
    }
    
    printf("Magnitude checksum: %.6f\n", checksum);
    printf("Time: %.4f seconds\n", elapsed);
    printf("Samples/sec: %.0f\n", N / elapsed);
    
    return 0;
}
