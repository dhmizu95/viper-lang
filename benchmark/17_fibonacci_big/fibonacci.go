/*
 * Fibonacci(1,000,000) - Big Integer Benchmark
 * Tests arbitrary precision arithmetic and iterative computation
 */
package main

import (
	"fmt"
	"math/big"
	"time"
)

func main() {
	const N = 1000000
	
	fmt.Printf("Computing Fibonacci(%d)...\n", N)
	
	start := time.Now()
	
	a := big.NewInt(0)
	b := big.NewInt(1)
	temp := new(big.Int)
	
	for i := 0; i < N; i++ {
		temp.Set(a)
		a.Add(a, b)
		b.Set(temp)
	}
	
	elapsed := time.Since(start)
	
	// Get number of digits for verification
	digits := len(a.String())
	fmt.Printf("Fibonacci(%d) has %d digits\n", N, digits)
	
	// Get first and last digits for verification
	str := a.String()
	fmt.Printf("First 10 digits: %s...\n", str[:10])
	fmt.Printf("Last 10 digits: %s...\n", str[len(str)-10:])
	
	fmt.Printf("Time: %v\n", elapsed)
}
