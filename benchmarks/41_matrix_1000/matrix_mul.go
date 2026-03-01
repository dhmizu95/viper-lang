/*
 * Matrix Multiplication 1000x1000 - Linear Algebra Benchmark
 * Tests memory bandwidth and vectorization
 */
package main

import (
	"fmt"
	"time"
)

const N = 1000

func main() {
	fmt.Printf("Computing matrix multiplication %dx%d...\n", N, N)
	
	// Initialize matrices with test data
	A := make([][]float64, N)
	B := make([][]float64, N)
	C := make([][]float64, N)
	
	for i := 0; i < N; i++ {
		A[i] = make([]float64, N)
		B[i] = make([]float64, N)
		C[i] = make([]float64, N)
		for j := 0; j < N; j++ {
			A[i][j] = float64((i+j)%100) / 100.0
			B[i][j] = float64((i*j)%100) / 100.0
		}
	}
	
	start := time.Now()
	
	// Standard O(n^3) matrix multiplication with cache-friendly ordering
	for i := 0; i < N; i++ {
		for k := 0; k < N; k++ {
			a_ik := A[i][k]
			for j := 0; j < N; j++ {
				C[i][j] += a_ik * B[k][j]
			}
		}
	}
	
	elapsed := time.Since(start)
	
	// Verification: compute checksum
	var checksum float64
	for i := 0; i < N; i++ {
		for j := 0; j < N; j++ {
			checksum += C[i][j]
		}
	}
	
	fmt.Printf("Checksum: %.6f\n", checksum)
	fmt.Printf("Time: %v\n", elapsed)
	gflops := (2.0 * float64(N*N*N) / 1e9) / elapsed.Seconds()
	fmt.Printf("GFLOPS: %.2f\n", gflops)
}
