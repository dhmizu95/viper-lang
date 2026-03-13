// Integer Arithmetic Hot Loop Benchmark - C Implementation

#include <stdio.h>

int main() {
    const long n = 2000000;
    long acc = 1;

    for (long i = 1; i <= n; i++) {
        acc = acc + i;
        acc = acc - (i % 7);
        acc = acc + ((i * 3) % 11);
        if (i % 5 == 0) {
            acc = acc / 2 + 17;
        }
    }

    printf("int hotloop checksum: %ld\n", acc);
    return 0;
}
