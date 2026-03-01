// Benchmark 04: QuickSort
// Category: Discrete Mathematics / Sorting
// Tests: Recursion, array manipulation, comparisons

package main

import (
	"fmt"
	"time"
)

const SIZE = 100000 // 100k elements

func swap(arr []int, i, j int) {
	arr[i], arr[j] = arr[j], arr[i]
}

func partition(arr []int, low, high int) int {
	pivot := arr[high]
	i := low - 1

	for j := low; j < high; j++ {
		if arr[j] <= pivot {
			i++
			swap(arr, i, j)
		}
	}
	swap(arr, i+1, high)
	return i + 1
}

func quickSort(arr []int, low, high int) {
	if low < high {
		pi := partition(arr, low, high)
		quickSort(arr, low, pi-1)
		quickSort(arr, pi+1, high)
	}
}

func main() {
	start := time.Now()

	// Allocate and initialize array
	arr := make([]int, SIZE)
	for i := 0; i < SIZE; i++ {
		arr[i] = (SIZE - i) * 17 % SIZE
	}

	// Sort
	quickSort(arr, 0, SIZE-1)

	elapsed := time.Since(start)

	// Verify sorted
	sorted := true
	for i := 1; i < SIZE; i++ {
		if arr[i] < arr[i-1] {
			sorted = false
			break
		}
	}

	fmt.Printf("Array size: %d\n", SIZE)
	fmt.Printf("Sorted correctly: %v\n", sorted)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
