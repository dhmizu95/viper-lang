// Benchmark 04: QuickSort
// Category: Discrete Mathematics / Sorting
// Tests: Recursion, array manipulation, comparisons

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define SIZE 100000  // 100k elements

// Swap two elements
void swap(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

// Partition function
int partition(int* arr, int low, int high) {
    int pivot = arr[high];
    int i = low - 1;
    
    for (int j = low; j < high; j++) {
        if (arr[j] <= pivot) {
            i++;
            swap(&arr[i], &arr[j]);
        }
    }
    swap(&arr[i + 1], &arr[high]);
    return i + 1;
}

// QuickSort implementation
void quickSort(int* arr, int low, int high) {
    if (low < high) {
        int pi = partition(arr, low, high);
        quickSort(arr, low, pi - 1);
        quickSort(arr, pi + 1, high);
    }
}

int main() {
    clock_t start = clock();
    
    // Allocate and initialize array
    int* arr = (int*)malloc(SIZE * sizeof(int));
    if (!arr) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    // Fill with random-like values
    for (int i = 0; i < SIZE; i++) {
        arr[i] = (SIZE - i) * 17 % SIZE;
    }
    
    // Sort
    quickSort(arr, 0, SIZE - 1);
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    // Verify sorted
    int sorted = 1;
    for (int i = 1; i < SIZE; i++) {
        if (arr[i] < arr[i-1]) {
            sorted = 0;
            break;
        }
    }
    
    printf("Array size: %d\n", SIZE);
    printf("Sorted correctly: %s\n", sorted ? "Yes" : "No");
    printf("Time: %.4f seconds\n", time_spent);
    
    free(arr);
    return 0;
}
