// Factorial Benchmark - C Implementation
// Recursive Factorial calculation

#include <stdio.h>

long long fact(int n) {
    if (n <= 1) {
        return 1;
    }
    return n * fact(n - 1);
}

int main() {
    int n = 15;
    long long result = fact(n);
    printf("fact(%d) = %lld\n", n, result);
    return 0;
}