/*
 * Fibonacci(1,000,000) - Big Integer Benchmark
 * Tests arbitrary precision arithmetic and iterative computation
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <gmp.h>

int main() {
    const int N = 1000000;
    mpz_t a, b, temp;
    
    printf("Computing Fibonacci(%d)...\n", N);
    
    mpz_init_set_ui(a, 0);
    mpz_init_set_ui(b, 1);
    mpz_init(temp);
    
    for (int i = 0; i < N; i++) {
        mpz_set(temp, a);
        mpz_add(a, a, b);
        mpz_set(b, temp);
    }
    
    // Get number of digits for verification
    size_t digits = mpz_sizeinbase(a, 10);
    printf("Fibonacci(%d) has %zu digits\n", N, digits);
    
    // Get first 10 digits for verification
    char *str = mpz_get_str(NULL, 10, a);
    printf("First 10 digits: %.10s...\n", str);
    printf("Last 10 digits: %.10s...\n", str + digits - 10);
    free(str);
    
    mpz_clear(a);
    mpz_clear(b);
    mpz_clear(temp);
    
    return 0;
}
