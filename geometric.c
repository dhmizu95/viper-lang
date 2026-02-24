#include <stdio.h>

long long geometric_sum(long long n, long long r) {
    if (n == 0) return 1;
    return 1 + geometric_sum(n - 1, r) * r;
}

int main() {
    printf("%lld\n", geometric_sum(100000, 2));
    return 0;
}
