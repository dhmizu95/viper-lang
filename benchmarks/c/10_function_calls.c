// Function Call Overhead Benchmark - C Implementation

#include <stdio.h>

static inline long mix(long x, long y) {
    return ((x * 3) + (y * 5) - (x % 7) + (y % 11)) % 1000003;
}

int main() {
    const long n = 1500000;
    long acc = 0;

    for (long i = 0; i < n; i++) {
        acc = acc + mix(i, acc + i);
    }

    printf("function call checksum: %ld\n", acc);
    return 0;
}
