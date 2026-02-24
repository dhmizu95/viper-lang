/* Prime Sieve of Eratosthenes - C Implementation */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>
#include <string.h>

int64_t sieve(int64_t n) {
    // Create sieve array (1 = potentially prime)
    uint8_t *is_prime = (uint8_t *)calloc(n + 1, sizeof(uint8_t));
    if (!is_prime) {
        fprintf(stderr, "Memory allocation failed for n=%ld\n", n);
        return -1;
    }
    
    // Initialize all to 1 (potentially prime)
    memset(is_prime, 1, n + 1);
    is_prime[0] = 0;
    is_prime[1] = 0;
    
    // Sieve of Eratosthenes
    for (int64_t i = 2; i * i <= n; i++) {
        if (is_prime[i]) {
            // Mark multiples as composite
            for (int64_t j = i * i; j <= n; j += i) {
                is_prime[j] = 0;
            }
        }
    }
    
    // Count primes
    int64_t count = 0;
    for (int64_t i = 2; i <= n; i++) {
        if (is_prime[i]) {
            count++;
        }
    }
    
    free(is_prime);
    return count;
}

int main() {
    printf("Prime Sieve Benchmark\n");
    printf("=====================\n");
    
    int64_t sizes[] = {100000, 500000, 1000000, 5000000, 10000000};
    int num_sizes = sizeof(sizes) / sizeof(sizes[0]);
    
    for (int i = 0; i < num_sizes; i++) {
        int64_t n = sizes[i];
        printf("Sieving up to: %ld\n", n);
        
        clock_t start = clock();
        int64_t count = sieve(n);
        clock_t end = clock();
        
        double elapsed = (double)(end - start) / CLOCKS_PER_SEC * 1000.0;
        printf("Primes found: %ld\n", count);
        printf("Time: %.2f ms\n", elapsed);
        printf("\n");
    }
    
    printf("Benchmark complete!\n");
    return 0;
}
