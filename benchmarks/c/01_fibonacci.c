// Fibonacci Benchmark - C Implementation
// Recursive Fibonacci calculation

#include <stdio.h>

long fib(int n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

int main() {
    int n = 35;
    long result = fib(n);
    printf("fib(%d) = %ld\n", n, result);
    return 0;
}
