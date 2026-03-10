// Prime Count Benchmark - C Implementation
// Count primes using trial division (matches Viper algorithm)

#include <stdio.h>

int is_prime(int n) {
    if (n < 2) return 0;
    if (n == 2) return 1;
    if (n % 2 == 0) return 0;
    
    int i = 3;
    while (i * i <= n) {
        if (n % i == 0) return 0;
        i = i + 2;
    }
    return 1;
}

int count_primes(int n) {
    int count = 0;
    int i = 2;
    while (i <= n) {
        if (is_prime(i)) count++;
        i = i + 1;
    }
    return count;
}

int main() {
    int n = 5000;
    int result = count_primes(n);
    printf("primes up to %d: %d\n", n, result);
    return 0;
}
