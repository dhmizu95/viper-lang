/*
 * Factorial(1,000,000) - Big Integer Benchmark
 * Tests arbitrary precision arithmetic and memory management
 */
package main

import (
	"fmt"
	"math/big"
	"time"
)

func main() {
	const N = 1000000
	
	fmt.Printf("Computing factorial(%d)...\n", N)
	
	start := time.Now()
	
	result := big.NewInt(1)
	for i := int64(2); i <= N; i++ {
		result.Mul(result, big.NewInt(i))
	}
	
	elapsed := time.Since(start)
	
	// Get number of digits for verification
	digits := len(result.String())
	fmt.Printf("Factorial(%d) has %d digits\n", N, digits)
	
	// Get first and last digits for verification
	str := result.String()
	fmt.Printf("First 10 digits: %s...\n", str[:10])
	fmt.Printf("Last 10 digits: %s...\n", str[len(str)-10:])
	
	fmt.Printf("Time: %v\n", elapsed)
}
