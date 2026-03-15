// Recursive List Sum Benchmark - C Implementation
// Recursive sum of an array

#include <stdio.h>

long long sum_array(long long arr[], int size, int idx) {
    if (idx >= size) {
        return 0;
    }
    return arr[idx] + sum_array(arr, size, idx + 1);
}

int main() {
    const int size = 1000;
    long long arr[size];
    for (int i = 0; i < size; i++) {
        arr[i] = i + 1;
    }
    
    long long result = sum_array(arr, size, 0);
    printf("%lld\n", result);
    return 0;
}