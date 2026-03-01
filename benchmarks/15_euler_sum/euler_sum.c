// Benchmark 15: Euler Sum (Harmonic Series)
// Category: Floating Point / Numerical Analysis
// Tests: Floating-point summation, precision, Kahan summation

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>

#define N 100000000  // 100 million terms

// Naive summation
double naive_sum(int n) {
    double sum = 0.0;
    for (int i = 1; i <= n; i++) {
        sum += 1.0 / i;
    }
    return sum;
}

// Kahan summation (compensated summation)
double kahan_sum(int n) {
    double sum = 0.0;
    double c = 0.0;  // Running compensation for lost low-order bits
    
    for (int i = 1; i <= n; i++) {
        double y = 1.0 / i - c;
        double t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    return sum;
}

int main() {
    clock_t start = clock();
    
    double naive_result = naive_sum(N);
    double kahan_result = kahan_sum(N);
    
    // Theoretical approximation: H(n) ≈ ln(n) + γ + 1/(2n)
    // where γ (Euler-Mascheroni constant) ≈ 0.5772156649015328606
    double euler_mascheroni = 0.5772156649015328606;
    double theoretical = log(N) + euler_mascheroni + 1.0 / (2.0 * N);
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Number of terms: %d\n", N);
    printf("Naive summation:     %.15f\n", naive_result);
    printf("Kahan summation:     %.15f\n", kahan_result);
    printf("Theoretical value:   %.15f\n", theoretical);
    printf("Naive error:         %.15f\n", fabs(naive_result - theoretical));
    printf("Kahan error:         %.15f\n", fabs(kahan_result - theoretical));
    printf("Time: %.4f seconds\n", time_spent);
    
    return 0;
}
