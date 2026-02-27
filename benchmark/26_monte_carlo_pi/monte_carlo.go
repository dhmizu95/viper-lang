/*
 * Monte Carlo Pi - Floating Point Benchmark
 * Tests floating-point performance and random number generation
 * Uses 1 billion samples
 */
package main

import (
	"fmt"
	"math"
	"math/rand"
	"time"
)

func main() {
	const SAMPLES = 1000000000 // 1 billion
	inside := int64(0)
	
	fmt.Printf("Computing Pi using Monte Carlo with %d samples...\n", SAMPLES)
	
	start := time.Now()
	
	rng := rand.New(rand.NewSource(42))
	
	for i := int64(0); i < SAMPLES; i++ {
		x := rng.Float64()
		y := rng.Float64()
		if x*x+y*y <= 1.0 {
			inside++
		}
	}
	
	elapsed := time.Since(start)
	
	pi := 4.0 * float64(inside) / float64(SAMPLES)
	
	fmt.Printf("Estimated Pi: %.15f\n", pi)
	fmt.Printf("Actual Pi:    3.141592653589793\n")
	fmt.Printf("Error:        %.15f\n", math.Abs(pi-3.141592653589793))
	fmt.Printf("Time:         %v\n", elapsed)
	fmt.Printf("Samples/sec:  %.0f\n", float64(SAMPLES)/elapsed.Seconds())
}
