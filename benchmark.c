#include <stdio.h>

volatile long long sink;

int main() {
    long long sum = 0;
    long long i = 0;
    while (i < 1000000000) {
        sum = sum + i;
        i = i + 1;
    }
    sink = sum;
    printf("%lld\n", sum);
    return 0;
}
