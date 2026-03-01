/*
 * Factorial(1,000,000) - Big Integer Benchmark
 * Tests arbitrary precision arithmetic and memory management
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <gmp.h>

int main() {
    const int N = 1000000;
    mpz_t result;
    
    printf("Computing factorial(%d)...\n", N);
    
    mpz_init(result);
    mpz_fac_ui(result, N);
    
    // Get number of digits for verification
    size_t digits = mpz_sizeinbase(result, 10);
    printf("Factorial(%d) has %zu digits\n", N, digits);
    
    // Get first 10 digits for verification
    char *str = mpz_get_str(NULL, 10, result);
    printf("First 10 digits: %.10s...\n", str);
    printf("Last 10 digits: %.10s...\n", str + digits - 10);
    free(str);
    
    mpz_clear(result);
    
    return 0;
}
