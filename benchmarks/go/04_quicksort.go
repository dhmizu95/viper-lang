// QuickSort Benchmark - Go Implementation
// Array sorting algorithm with iterative quicksort

package main

import "fmt"

func partition(arr []int64, low, high int) int {
	pivot := arr[high]
	i := low

	for j := low; j < high; j++ {
		if arr[j] < pivot {
			arr[i], arr[j] = arr[j], arr[i]
			i++
		}
	}
	arr[i], arr[high] = arr[high], arr[i]
	return i
}

func quicksort(arr []int64, low, high int) {
	// Use stack to avoid recursion
	stack := make([]int, 100)
	top := 0

	if low >= high {
		return
	}

	stack[top] = low
	top++
	stack[top] = high
	top++

	for top > 0 {
		top--
		h := stack[top]
		top--
		l := stack[top]

		if l < h {
			p := partition(arr, l, h)

			if p > l {
				stack[top] = l
				top++
				stack[top] = p - 1
				top++
			}

			if p+1 < h {
				stack[top] = p + 1
				top++
				stack[top] = h
				top++
			}
		}
	}
}

func main() {
	// Benchmark parameter - array size
	n := 100

	// Initialize array with pseudo-random values
	arr := make([]int64, n)
	seed := int64(12345)

	for i := 0; i < n; i++ {
		seed = (seed*1103515245 + 12345) % 2147483648
		arr[i] = seed % 1000
	}

	// Sort array
	quicksort(arr, 0, n-1)

	// Calculate checksum to verify sort
	var checksum int64 = 0
	for _, v := range arr {
		checksum += v
	}

	fmt.Printf("quicksort %d elements checksum: %d\n", n, checksum)
}
