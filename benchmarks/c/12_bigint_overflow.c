// BigInt Overflow Path Benchmark - C Implementation

#include <stdint.h>
#include <stdio.h>

int main() {
    __int128 value = ((__int128)1 << 100);
    long long checksum = 0;

    for (int64_t i = 0; i < 200000; i++) {
        value = value + 123456789;
        value = value - 98765432;
        checksum += (long long)(value % 97);
    }

    printf("bigint overflow checksum: %lld\n", checksum);
    return 0;
}
