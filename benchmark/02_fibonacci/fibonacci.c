// Benchmark 02: Fibonacci Numbers
// Category: Integer Arithmetic / Recursion
// Tests: Loop performance, variable assignment, arithmetic

#include <stdio.h>
#include <time.h>

#define ITERATIONS 10000000  // 10 million iterations

int main() {
    clock_t start = clock();
    
    long long a = 0, b = 1;
    long long count = 0;
    
    for (long long i = 0; i < ITERATIONS; i++) {
        long long temp = a + b;
        a = b;
        b = temp;
        count++;
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Fibonacci iterations: %lld\n", count);
    printf("Final value (last 10 digits): %lld\n", a % 10000000000LL);
    printf("Time: %.4f seconds\n", time_spent);
    
    return 0;
}
