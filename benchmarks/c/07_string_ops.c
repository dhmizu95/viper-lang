// String Operations Benchmark - C Implementation
// Character counting in byte array

#include <stdio.h>

#define N 9000

int main() {
    // Create array with repeating pattern
    long s[N];
    for (int i = 0; i < N; i++) {
        s[i] = (i * 7 + 3) % 128;
    }
    
    // Count occurrences of specific values
    long count1 = 0, count2 = 0, count3 = 0, count4 = 0;
    for (int i = 0; i < N; i++) {
        if (s[i] == 65) {
            count1++;
        } else if (s[i] == 66) {
            count2++;
        } else if (s[i] == 67) {
            count3++;
        } else if (s[i] == 68) {
            count4++;
        }
    }
    
    // Calculate checksum
    long checksum = N + count1 + count2 + count3 + count4;
    printf("string operations checksum: %ld\n", checksum);
    return 0;
}
