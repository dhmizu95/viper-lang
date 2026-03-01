/*
 * Monte Carlo Pi - Floating Point Benchmark
 * Tests floating-point performance and random number generation
 * Uses 1 billion samples
 */
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <math.h>
#include <stdint.h>

// Fast random number generator (xorshift64)
static uint64_t x = 123456789ULL;

static inline uint64_t random_uint64() {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    return x;
}

static inline double random_double() {
    return (random_uint64() >> 11) * (1.0 / 9007199254740992.0);
}

int main() {
    const long SAMPLES = 1000000000L;  // 1 billion
    long inside = 0;
    
    printf("Computing Pi using Monte Carlo with %ld samples...\n", SAMPLES);
    
    clock_t start = clock();
    
    for (long i = 0; i < SAMPLES; i++) {
        double x = random_double();
        double y = random_double();
        if (x * x + y * y <= 1.0) {
            inside++;
        }
    }
    
    clock_t end = clock();
    double elapsed = (double)(end - start) / CLOCKS_PER_SEC;
    
    double pi = 4.0 * (double)inside / (double)SAMPLES;
    
    printf("Estimated Pi: %.15f\n", pi);
    printf("Actual Pi:    3.141592653589793\n");
    printf("Error:        %.15f\n", fabs(pi - 3.141592653589793));
    printf("Time:         %.4f seconds\n", elapsed);
    printf("Samples/sec:  %.0f\n", (double)SAMPLES / elapsed);
    
    return 0;
}
