// Fast Fourier Transform (FFT) Benchmark - C Implementation
// Cooley-Tukey radix-2 FFT algorithm
// Tests: Floating point precision, Recursion performance, Memory allocation

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <complex.h>

#define N 256  // FFT size (must be power of 2)
#define PI 3.14159265358979323846

// Complex number structure
typedef struct {
    double real;
    double imag;
} Complex;

// Create complex number
Complex complex_make(double real, double imag) {
    Complex c;
    c.real = real;
    c.imag = imag;
    return c;
}

// Complex addition
Complex complex_add(Complex a, Complex b) {
    return complex_make(a.real + b.real, a.imag + b.imag);
}

// Complex subtraction
Complex complex_sub(Complex a, Complex b) {
    return complex_make(a.real - b.real, a.imag - b.imag);
}

// Complex multiplication
Complex complex_mul(Complex a, Complex b) {
    return complex_make(
        a.real * b.real - a.imag * b.imag,
        a.real * b.imag + a.imag * b.real
    );
}

// Bit-reversal permutation
int bit_reverse(int n, int bits) {
    int reversed = 0;
    for (int i = 0; i < bits; i++) {
        reversed = (reversed << 1) | (n & 1);
        n >>= 1;
    }
    return reversed;
}

// Cooley-Tukey FFT (iterative, radix-2, decimation-in-time)
void fft(Complex* x, int n, int inverse) {
    int bits = 0;
    for (int i = n; i > 1; i >>= 1) bits++;
    
    // Bit-reversal permutation
    for (int i = 0; i < n; i++) {
        int rev = bit_reverse(i, bits);
        if (i < rev) {
            Complex temp = x[i];
            x[i] = x[rev];
            x[rev] = temp;
        }
    }
    
    // Cooley-Tukey iterative FFT
    for (int len = 2; len <= n; len <<= 1) {
        double angle = 2 * PI / len * (inverse ? -1 : 1);
        Complex wlen = complex_make(cos(angle), sin(angle));
        
        for (int i = 0; i < n; i += len) {
            Complex w = complex_make(1.0, 0.0);
            for (int j = 0; j < len / 2; j++) {
                Complex u = x[i + j];
                Complex v = complex_mul(x[i + j + len/2], w);
                x[i + j] = complex_add(u, v);
                x[i + j + len/2] = complex_sub(u, v);
                w = complex_mul(w, wlen);
            }
        }
    }
    
    // Scale for inverse FFT
    if (inverse) {
        for (int i = 0; i < n; i++) {
            x[i].real /= n;
            x[i].imag /= n;
        }
    }
}

// Generate test signal: sum of two sinusoids
void generate_signal(Complex* signal, int n) {
    double freq1 = 2.0;  // 2 Hz
    double freq2 = 8.0;  // 8 Hz
    double sample_rate = 64.0;
    
    for (int i = 0; i < n; i++) {
        double t = i / sample_rate;
        double real = sin(2 * PI * freq1 * t) + 0.5 * sin(2 * PI * freq2 * t);
        signal[i] = complex_make(real, 0.0);
    }
}

// Calculate total magnitude
double total_magnitude(Complex* spectrum, int n) {
    double total = 0.0;
    for (int i = 0; i < n; i++) {
        total += sqrt(spectrum[i].real * spectrum[i].real + 
                      spectrum[i].imag * spectrum[i].imag);
    }
    return total;
}

int main() {
    // Allocate memory for signal
    Complex* signal = (Complex*)malloc(N * sizeof(Complex));
    if (!signal) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    // Generate test signal
    generate_signal(signal, N);
    
    // Perform FFT
    fft(signal, N, 0);
    
    // Calculate and print result
    double magnitude = total_magnitude(signal, N);
    printf("%.6f\n", magnitude);
    
    // Perform inverse FFT to verify
    fft(signal, N, 1);
    
    // Print first sample of reconstructed signal (verification)
    printf("%.6f\n", signal[0].real);
    
    free(signal);
    return 0;
}
