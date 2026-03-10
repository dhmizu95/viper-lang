// Matrix Multiplication Benchmark - Go Implementation
// Multiply two NxN matrices using arrays

package main

import "fmt"

func main() {
	// Benchmark parameter
	n := 50

	// Initialize matrices
	a := make([]int64, n*n)
	b := make([]int64, n*n)
	c := make([]int64, n*n)

	// Fill matrices with values
	for i := 0; i < n; i++ {
		for j := 0; j < n; j++ {
			idx := i*n + j
			a[idx] = int64((i + j) % 10)
			b[idx] = int64((i * j) % 10)
		}
	}

	// Matrix multiplication: C = A * B
	for i := 0; i < n; i++ {
		for j := 0; j < n; j++ {
			var sum int64 = 0
			for k := 0; k < n; k++ {
				a_idx := i*n + k
				b_idx := k*n + j
				sum += a[a_idx] * b[b_idx]
			}
			c_idx := i*n + j
			c[c_idx] = sum
		}
	}

	// Calculate checksum
	var checksum int64 = 0
	for _, v := range c {
		checksum += v
	}

	fmt.Printf("matrix %dx%d checksum: %d\n", n, n, checksum)
}
