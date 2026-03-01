// Benchmark 03: Matrix Multiplication
// Category: Linear Algebra
// Tests: Nested loops, array access, floating-point operations

package main

import (
	"fmt"
	"time"
)

const SIZE = 512 // 512x512 matrices

func main() {
	start := time.Now()

	// Allocate matrices
	A := make([]float64, SIZE*SIZE)
	B := make([]float64, SIZE*SIZE)
	C := make([]float64, SIZE*SIZE)

	// Initialize matrices
	for i := 0; i < SIZE*SIZE; i++ {
		A[i] = float64(i%100) / 100.0
		B[i] = float64(i%50) / 50.0
		C[i] = 0.0
	}

	// Matrix multiplication C = A * B
	for i := 0; i < SIZE; i++ {
		for j := 0; j < SIZE; j++ {
			sum := 0.0
			for k := 0; k < SIZE; k++ {
				sum += A[i*SIZE+k] * B[k*SIZE+j]
			}
			C[i*SIZE+j] = sum
		}
	}

	elapsed := time.Since(start)

	// Verify result (sum of first row)
	verify := 0.0
	for j := 0; j < SIZE; j++ {
		verify += C[j]
	}

	fmt.Printf("Matrix size: %dx%d\n", SIZE, SIZE)
	fmt.Printf("Verification sum: %.6f\n", verify)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
