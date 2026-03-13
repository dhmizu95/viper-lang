// Fixed-Width i64 Arithmetic Hot Loop Benchmark - C Implementation

#include <stdint.h>
#include <stdio.h>

int main() {
    const int64_t n = 2000000;
    int64_t acc = 1;

    for (int64_t i = 1; i <= n; i++) {
        acc = acc + i;
        acc = acc - (i % 7);
        acc = acc + ((i * 3) % 11);
        if (i % 5 == 0) {
            acc = acc / 2 + 17;
        }
    }

    printf("i64 hotloop checksum: %lld\n", (long long)acc);
    return 0;
}
