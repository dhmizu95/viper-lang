// Benchmark 10: Spectral Norm
// Category: Linear Algebra
// Tests: Matrix-vector operations, power iteration

package main

import (
	"fmt"
	"math"
	"time"
)

const (
	N          = 1500
	ITERATIONS = 10
)

// A[i][j] = 1 / (i + j + 1)
func aElement(i, j int) float64 {
	return 1.0 / float64(i+j+1)
}

// Multiply by A
func av(v []float64, Av []float64) {
	for i := 0; i < N; i++ {
		Av[i] = 0.0
		for j := 0; j < N; j++ {
			Av[i] += aElement(i, j) * v[j]
		}
	}
}

// Multiply by A^T
func atv(v []float64, Atv []float64) {
	for i := 0; i < N; i++ {
		Atv[i] = 0.0
		for j := 0; j < N; j++ {
			Atv[i] += aElement(j, i) * v[j]
		}
	}
}

// Multiply by A^T * A
func atav(v, result []float64) {
	temp := make([]float64, N)
	av(v, temp)
	atv(temp, result)
}

func main() {
	start := time.Now()

	u := make([]float64, N)
	v := make([]float64, N)

	// Initialize vectors
	for i := 0; i < N; i++ {
		u[i] = 1.0
	}

	// Power iteration
	for iter := 0; iter < ITERATIONS; iter++ {
		atav(u, v)

		// Normalize
		norm := 0.0
		for i := 0; i < N; i++ {
			norm += v[i] * v[i]
		}
		norm = math.Sqrt(norm)

		for i := 0; i < N; i++ {
			u[i] = v[i] / norm
		}
	}

	// Calculate spectral norm approximation
	AvResult := make([]float64, N)
	av(u, AvResult)

	spectralNorm := 0.0
	for i := 0; i < N; i++ {
		spectralNorm += u[i] * AvResult[i]
	}
	spectralNorm = math.Sqrt(spectralNorm)

	elapsed := time.Since(start)

	fmt.Printf("Matrix size: %dx%d\n", N, N)
	fmt.Printf("Iterations: %d\n", ITERATIONS)
	fmt.Printf("Spectral norm: %.10f\n", spectralNorm)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
