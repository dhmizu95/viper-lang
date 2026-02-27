/*
 * Matrix Multiplication 1000x1000 - Linear Algebra Benchmark
 * Tests memory bandwidth and vectorization
 */
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>

#define N 1000

static double A[N][N];
static double B[N][N];
static double C[N][N];

int main() {
    printf("Computing matrix multiplication %dx%d...\n", N, N);
    
    // Initialize matrices with test data
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            A[i][j] = (i + j) % 100 / 100.0;
            B[i][j] = (i * j) % 100 / 100.0;
            C[i][j] = 0.0;
        }
    }
    
    clock_t start = clock();
    
    // Standard O(n^3) matrix multiplication
    for (int i = 0; i < N; i++) {
        for (int k = 0; k < N; k++) {
            double a_ik = A[i][k];
            for (int j = 0; j < N; j++) {
                C[i][j] += a_ik * B[k][j];
            }
        }
    }
    
    clock_t end = clock();
    double elapsed = (double)(end - start) / CLOCKS_PER_SEC;
    
    // Verification: compute checksum
    double checksum = 0.0;
    for (int i = 0; i < N; i++) {
        for (int j = 0; j < N; j++) {
            checksum += C[i][j];
        }
    }
    
    printf("Checksum: %.6f\n", checksum);
    printf("Time: %.4f seconds\n", elapsed);
    printf("GFLOPS: %.2f\n", (2.0 * N * N * N / 1e9) / elapsed);
    
    return 0;
}
