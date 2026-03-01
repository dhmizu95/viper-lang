// Benchmark 03: Matrix Multiplication
// Category: Linear Algebra
// Tests: Nested loops, array access, floating-point operations

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define SIZE 512  // 512x512 matrices

int main() {
    clock_t start = clock();
    
    // Allocate matrices
    double *A = (double*)malloc(SIZE * SIZE * sizeof(double));
    double *B = (double*)malloc(SIZE * SIZE * sizeof(double));
    double *C = (double*)malloc(SIZE * SIZE * sizeof(double));
    
    if (!A || !B || !C) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    // Initialize matrices
    for (int i = 0; i < SIZE * SIZE; i++) {
        A[i] = (double)(i % 100) / 100.0;
        B[i] = (double)(i % 50) / 50.0;
        C[i] = 0.0;
    }
    
    // Matrix multiplication C = A * B
    for (int i = 0; i < SIZE; i++) {
        for (int j = 0; j < SIZE; j++) {
            double sum = 0.0;
            for (int k = 0; k < SIZE; k++) {
                sum += A[i * SIZE + k] * B[k * SIZE + j];
            }
            C[i * SIZE + j] = sum;
        }
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    // Verify result (sum of first row)
    double verify = 0.0;
    for (int j = 0; j < SIZE; j++) {
        verify += C[j];
    }
    
    printf("Matrix size: %dx%d\n", SIZE, SIZE);
    printf("Verification sum: %.6f\n", verify);
    printf("Time: %.4f seconds\n", time_spent);
    
    free(A);
    free(B);
    free(C);
    return 0;
}
