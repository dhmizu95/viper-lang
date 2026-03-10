// Matrix Multiplication Benchmark - C Implementation
// Multiply two NxN matrices using arrays

#include <stdio.h>

#define N 50

int main() {
    // Initialize matrices
    long a[N * N];
    long b[N * N];
    long c[N * N];
    
    // Fill matrices with values
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            int idx = i * N + j;
            a[idx] = (i + j) % 10;
            b[idx] = (i * j) % 10;
            c[idx] = 0;
        }
    }
    
    // Matrix multiplication: C = A * B
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            long sum = 0;
            for (int k = 0; k < N; k++) {
                int a_idx = i * N + k;
                int b_idx = k * N + j;
                sum += a[a_idx] * b[b_idx];
            }
            int c_idx = i * N + j;
            c[c_idx] = sum;
        }
    }
    
    // Calculate checksum
    long checksum = 0;
    for (int i = 0; i < N * N; i++) {
        checksum += c[i];
    }
    
    printf("matrix %dx%d checksum: %ld\n", N, N, checksum);
    return 0;
}
