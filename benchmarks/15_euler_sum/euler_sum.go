// Benchmark 15: Euler Sum (Harmonic Series)
// Category: Floating Point / Numerical Analysis
// Tests: Floating-point summation, precision, Kahan summation

package main

import (
	"fmt"
	"math"
	"time"
)

const N = 100000000 // 100 million terms

// Naive summation
func naiveSum(n int) float64 {
	sum := 0.0
	for i := 1; i <= n; i++ {
		sum += 1.0 / float64(i)
	}
	return sum
}

// Kahan summation (compensated summation)
func kahanSum(n int) float64 {
	sum := 0.0
	c := 0.0 // Running compensation for lost low-order bits

	for i := 1; i <= n; i++ {
		y := 1.0/float64(i) - c
		t := sum + y
		c = (t - sum) - y
		sum = t
	}
	return sum
}

func main() {
	start := time.Now()

	naiveResult := naiveSum(N)
	kahanResult := kahanSum(N)

	// Theoretical approximation: H(n) ≈ ln(n) + γ + 1/(2n)
	// where γ (Euler-Mascheroni constant) ≈ 0.5772156649015328606
	eulerMascheroni := 0.5772156649015328606
	theoretical := math.Log(float64(N)) + eulerMascheroni + 1.0/(2.0*float64(N))

	elapsed := time.Since(start)

	fmt.Printf("Number of terms: %d\n", N)
	fmt.Printf("Naive summation:     %.15f\n", naiveResult)
	fmt.Printf("Kahan summation:     %.15f\n", kahanResult)
	fmt.Printf("Theoretical value:   %.15f\n", theoretical)
	fmt.Printf("Naive error:         %.15f\n", math.Abs(naiveResult-theoretical))
	fmt.Printf("Kahan error:         %.15f\n", math.Abs(kahanResult-theoretical))
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
