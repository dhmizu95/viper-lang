// Recursive List Sum Benchmark - C Implementation
// Recursive sum of a range

#include <stdio.h>

long long sum_range(int n) {
    if (n <= 0) {
        return 0;
    }
    return n + sum_range(n - 1);
}

int main() {
    const int n = 200;
    long long result = sum_range(n);
    printf("%lld\n", result);
    return 0;
}
