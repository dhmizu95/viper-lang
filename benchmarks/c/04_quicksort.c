// QuickSort Benchmark - C Implementation
// Array sorting algorithm with iterative quicksort

#include <stdio.h>

void swap(long* a, long* b) {
    long temp = *a;
    *a = *b;
    *b = temp;
}

int partition(long arr[], int low, int high) {
    long pivot = arr[high];
    int i = low;
    
    for (int j = low; j < high; j++) {
        if (arr[j] < pivot) {
            swap(&arr[i], &arr[j]);
            i++;
        }
    }
    swap(&arr[i], &arr[high]);
    return i;
}

void quicksort(long arr[], int low, int high) {
    // Use stack to avoid recursion
    int stack[100];
    int top = 0;
    
    if (low >= high) {
        return;
    }
    
    stack[top++] = low;
    stack[top++] = high;
    
    while (top > 0) {
        int h = stack[--top];
        int l = stack[--top];
        
        if (l < h) {
            int p = partition(arr, l, h);
            
            if (p > l) {
                stack[top++] = l;
                stack[top++] = p - 1;
            }
            
            if (p + 1 < h) {
                stack[top++] = p + 1;
                stack[top++] = h;
            }
        }
    }
}

int main() {
    // Benchmark parameter - array size
    int n = 100;
    long arr[100];
    
    // Initialize array with pseudo-random values
    long seed = 12345;
    for (int i = 0; i < n; i++) {
        seed = (seed * 1103515245 + 12345) % 2147483648;
        arr[i] = seed % 1000;
    }
    
    // Sort array
    quicksort(arr, 0, n - 1);
    
    // Calculate checksum to verify sort
    long checksum = 0;
    for (int i = 0; i < n; i++) {
        checksum += arr[i];
    }
    
    printf("quicksort %d elements checksum: %ld\n", n, checksum);
    return 0;
}
