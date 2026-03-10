// Matrix Multiplication Benchmark - C Implementation
// Multiply two NxN matrices (matching Viper algorithm)

#include <stdio.h>

int main() {
    int n = 30;
    long checksum = 0;
    
    for (int row = 0; row < n; row++) {
        for (int col = 0; col < n; col++) {
            int sum = 0;
            for (int k = 0; k < n; k++) {
                int a_val = (row * n + k) % 10;
                int b_val = (k * n + col + 1) % 10;
                sum += a_val * b_val;
            }
            checksum += sum;
        }
    }
    
    printf("matrix %dx%d checksum: %ld\n", n, n, checksum);
    return 0;
}
