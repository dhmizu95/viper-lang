// Benchmark 02: Fibonacci Numbers
// Category: Integer Arithmetic / Recursion
// Tests: Loop performance, variable assignment, arithmetic

package main

import (
	"fmt"
	"time"
)

const ITERATIONS = 10000000 // 10 million iterations

func main() {
	start := time.Now()

	var a, b int64 = 0, 1
	var count int64 = 0

	for i := int64(0); i < ITERATIONS; i++ {
		temp := a + b
		a = b
		b = temp
		count++
	}

	elapsed := time.Since(start)

	fmt.Printf("Fibonacci iterations: %d\n", count)
	fmt.Printf("Final value (last 10 digits): %d\n", a%10000000000)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
