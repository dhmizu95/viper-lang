// Benchmark 09: Fannkuch
// Category: Discrete Mathematics / Permutations
// Tests: Array manipulation, permutations, recursion

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define N 10

int max_flips = 0;
int checksum = 0;

// Flip array elements up to index k
void flip(int* arr, int k) {
    for (int i = 0, j = k; i < j; i++, j--) {
        int temp = arr[i];
        arr[i] = arr[j];
        arr[j] = temp;
    }
}

// Calculate fannkuch for a permutation
int fannkuch(int* arr, int n) {
    int flips = 0;
    while (arr[0] != 0) {
        flip(arr, arr[0]);
        flips++;
    }
    return flips;
}

// Generate permutations and calculate fannkuch
void permute(int* arr, int* count, int n, int depth) {
    if (depth == n) {
        // Make a copy for fannkuch calculation
        int temp[N];
        for (int i = 0; i < n; i++) temp[i] = arr[i];
        
        int flips = fannkuch(temp, n);
        if (flips > max_flips) max_flips = flips;
        
        // Add to checksum with alternating sign
        int sign = (count[0] % 2 == 0) ? 1 : -1;
        checksum += sign * flips;
        
        return;
    }
    
    for (int i = depth; i < n; i++) {
        // Swap
        int swap_temp = arr[depth];
        arr[depth] = arr[i];
        arr[i] = swap_temp;

        count[depth]++;

        permute(arr, count, n, depth + 1);

        // Rotate back
        int rotate_temp = arr[depth];
        for (int j = depth; j < n - 1; j++) {
            arr[j] = arr[j + 1];
        }
        arr[n - 1] = rotate_temp;
        
        if (count[depth] >= n - depth) {
            count[depth] = 0;
        } else {
            break;
        }
    }
}

int main() {
    clock_t start = clock();
    
    int arr[N];
    int count[N] = {0};
    
    for (int i = 0; i < N; i++) arr[i] = i;
    
    permute(arr, count, N, 0);
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Permutations of %d elements\n", N);
    printf("Maximum flips: %d\n", max_flips);
    printf("Checksum: %d\n", checksum);
    printf("Time: %.4f seconds\n", time_spent);
    
    return 0;
}
