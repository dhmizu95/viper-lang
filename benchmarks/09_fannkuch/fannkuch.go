// Benchmark 09: Fannkuch
// Category: Discrete Mathematics / Permutations
// Tests: Array manipulation, permutations, recursion

package main

import (
	"fmt"
	"time"
)

const N = 10

var maxFlips int
var checksum int

// Flip array elements up to index k
func flip(arr []int, k int) {
	for i, j := 0, k; i < j; i, j = i+1, j-1 {
		arr[i], arr[j] = arr[j], arr[i]
	}
}

// Calculate fannkuch for a permutation
func fannkuch(arr []int, n int) int {
	flips := 0
	temp := make([]int, n)
	copy(temp, arr)

	for temp[0] != 0 {
		flip(temp, temp[0])
		flips++
	}
	return flips
}

// Generate permutations and calculate fannkuch
func permute(arr, count []int, n, depth int) {
	if depth == n {
		flips := fannkuch(arr, n)
		if flips > maxFlips {
			maxFlips = flips
		}

		// Add to checksum with alternating sign
		sign := 1
		if count[0]%2 == 1 {
			sign = -1
		}
		checksum += sign * flips
		return
	}

	for i := depth; i < n; i++ {
		// Swap
		arr[depth], arr[i] = arr[i], arr[depth]

		count[depth]++

		permute(arr, count, n, depth+1)

		// Rotate back
		temp := arr[depth]
		for j := depth; j < n-1; j++ {
			arr[j] = arr[j+1]
		}
		arr[n-1] = temp

		if count[depth] >= n-depth {
			count[depth] = 0
		} else {
			break
		}
	}
}

func main() {
	start := time.Now()

	arr := make([]int, N)
	count := make([]int, N)

	for i := 0; i < N; i++ {
		arr[i] = i
	}

	permute(arr, count, N, 0)

	elapsed := time.Since(start)

	fmt.Printf("Permutations of %d elements\n", N)
	fmt.Printf("Maximum flips: %d\n", maxFlips)
	fmt.Printf("Checksum: %d\n", checksum)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
