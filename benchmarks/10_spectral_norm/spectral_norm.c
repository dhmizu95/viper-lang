// Benchmark 10: Spectral Norm
// Category: Linear Algebra
// Tests: Matrix-vector operations, power iteration

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>

#define N 1500
#define ITERATIONS 10

// A[i][j] = 1 / (i + j + 1) - simplified for 1D array access
double A_element(int i, int j) {
    return 1.0 / (i + j + 1);
}

// Multiply by A
void Av(double* v, double* Av) {
    for (int i = 0; i < N; i++) {
        Av[i] = 0.0;
        for (int j = 0; j < N; j++) {
            Av[i] += A_element(i, j) * v[j];
        }
    }
}

// Multiply by A^T
void Atv(double* v, double* Atv) {
    for (int i = 0; i < N; i++) {
        Atv[i] = 0.0;
        for (int j = 0; j < N; j++) {
            Atv[i] += A_element(j, i) * v[j];
        }
    }
}

// Multiply by A^T * A
void AtAv(double* v, double* AtAv) {
    double temp[N];
    Av(v, temp);
    Atv(temp, AtAv);
}

int main() {
    clock_t start = clock();
    
    double u[N], v[N];
    
    // Initialize vectors
    for (int i = 0; i < N; i++) u[i] = 1.0;
    for (int i = 0; i < N; i++) v[i] = 0.0;
    
    // Power iteration
    for (int iter = 0; iter < ITERATIONS; iter++) {
        AtAv(u, v);
        
        // Normalize
        double norm = 0.0;
        for (int i = 0; i < N; i++) norm += v[i] * v[i];
        norm = sqrt(norm);
        
        for (int i = 0; i < N; i++) {
            u[i] = v[i] / norm;
        }
    }
    
    // Calculate spectral norm approximation
    double Av_result[N];
    Av(u, Av_result);
    
    double spectral_norm = 0.0;
    for (int i = 0; i < N; i++) {
        spectral_norm += u[i] * Av_result[i];
    }
    spectral_norm = sqrt(spectral_norm);
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Matrix size: %dx%d\n", N, N);
    printf("Iterations: %d\n", ITERATIONS);
    printf("Spectral norm: %.10f\n", spectral_norm);
    printf("Time: %.4f seconds\n", time_spent);
    
    return 0;
}
