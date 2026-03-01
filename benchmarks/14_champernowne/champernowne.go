// Benchmark 14: Champernowne Constant
// Category: Number Theory / String Processing
// Tests: Number to string conversion, string concatenation, digit counting

package main

import (
	"fmt"
	"strconv"
	"time"
)

const MAX_N = 1000000 // Concatenate numbers 1 to 1,000,000

func main() {
	start := time.Now()

	// Build buffer using byte slice
	buffer := make([]byte, 0, 6000000)

	// Concatenate all numbers from 1 to MAX_N
	for i := 1; i <= MAX_N; i++ {
		buffer = strconv.AppendInt(buffer, int64(i), 10)
	}

	// Count each digit 0-9
	digitCounts := make([]int, 10)
	for _, b := range buffer {
		digitCounts[b-'0']++
	}

	elapsed := time.Since(start)

	fmt.Printf("Numbers concatenated: 1 to %d\n", MAX_N)
	fmt.Printf("Total length: %d characters\n", len(buffer))
	fmt.Printf("Digit counts:\n")
	for d := 0; d < 10; d++ {
		fmt.Printf("  %d: %d\n", d, digitCounts[d])
	}
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
