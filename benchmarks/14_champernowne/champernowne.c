// Benchmark 14: Champernowne Constant
// Category: Number Theory / String Processing
// Tests: Number to string conversion, string concatenation, digit counting

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define MAX_N 1000000  // Concatenate numbers 1 to 1,000,000

// Count occurrences of a digit in the Champernowne constant
int count_digit(const char* str, char digit, long len) {
    int count = 0;
    for (long i = 0; i < len; i++) {
        if (str[i] == digit) count++;
    }
    return count;
}

int main() {
    clock_t start = clock();
    
    // Estimate buffer size (average ~5.5 digits per number for 1 to 1M)
    char* buffer = (char*)malloc(6000000);
    if (!buffer) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    long pos = 0;
    char num_str[12];
    
    // Concatenate all numbers from 1 to MAX_N
    for (int i = 1; i <= MAX_N; i++) {
        int len = sprintf(num_str, "%d", i);
        memcpy(buffer + pos, num_str, len);
        pos += len;
    }
    buffer[pos] = '\0';
    
    // Count each digit 0-9
    int digit_counts[10];
    for (int d = 0; d < 10; d++) {
        digit_counts[d] = count_digit(buffer, '0' + d, pos);
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Numbers concatenated: 1 to %d\n", MAX_N);
    printf("Total length: %ld characters\n", pos);
    printf("Digit counts:\n");
    for (int d = 0; d < 10; d++) {
        printf("  %d: %d\n", d, digit_counts[d]);
    }
    printf("Time: %.4f seconds\n", time_spent);
    
    free(buffer);
    
    return 0;
}
