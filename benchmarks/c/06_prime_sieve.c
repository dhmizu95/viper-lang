// Prime Sieve Benchmark - C Implementation
// Sieve of Eratosthenes algorithm

#include <stdio.h>
#include <stdbool.h>

#define N 10000

int main() {
    // Initialize sieve (true = potentially prime)
    bool sieve[N + 1];
    for (int i = 0; i <= N; i++) {
        sieve[i] = true;
    }
    sieve[0] = false;
    sieve[1] = false;
    
    // Sieve of Eratosthenes
    for (int p = 2; p * p <= N; p++) {
        if (sieve[p]) {
            // Mark all multiples of p as not prime
            for (int i = p * p; i <= N; i += p) {
                sieve[i] = false;
            }
        }
    }
    
    // Count primes
    int count = 0;
    for (int i = 0; i <= N; i++) {
        if (sieve[i]) {
            count++;
        }
    }
    
    printf("primes up to %d: %d\n", N, count);
    return 0;
}
