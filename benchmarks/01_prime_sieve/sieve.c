// Benchmark 01: Prime Sieve (Eratosthenes)
// Category: Integer Arithmetic
// Tests: Array operations, basic arithmetic, memory access

#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <math.h>
#include <time.h>

#define LIMIT 10000000  // 10 million

int main() {
    clock_t start = clock();
    
    // Allocate sieve array
    bool *is_prime = (bool*)calloc(LIMIT + 1, sizeof(bool));
    if (!is_prime) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    // Initialize all as prime
    for (int i = 2; i <= LIMIT; i++) {
        is_prime[i] = true;
    }
    
    // Sieve of Eratosthenes
    int sqrt_limit = (int)sqrt(LIMIT);
    for (int p = 2; p <= sqrt_limit; p++) {
        if (is_prime[p]) {
            for (int i = p * p; i <= LIMIT; i += p) {
                is_prime[i] = false;
            }
        }
    }
    
    // Count primes
    int count = 0;
    for (int i = 2; i <= LIMIT; i++) {
        if (is_prime[i]) count++;
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Primes up to %d: %d\n", LIMIT, count);
    printf("Time: %.4f seconds\n", time_spent);
    
    free(is_prime);
    return 0;
}
