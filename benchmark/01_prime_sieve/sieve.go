// Benchmark 01: Prime Sieve (Eratosthenes)
// Category: Integer Arithmetic
// Tests: Array operations, basic arithmetic, memory access

package main

import (
	"fmt"
	"math"
	"time"
)

const LIMIT = 10000000 // 10 million

func main() {
	start := time.Now()

	// Allocate sieve array
	isPrime := make([]bool, LIMIT+1)
	for i := 2; i <= LIMIT; i++ {
		isPrime[i] = true
	}

	// Sieve of Eratosthenes
	sqrtLimit := int(math.Sqrt(float64(LIMIT)))
	for p := 2; p <= sqrtLimit; p++ {
		if isPrime[p] {
			for i := p * p; i <= LIMIT; i += p {
				isPrime[i] = false
			}
		}
	}

	// Count primes
	count := 0
	for i := 2; i <= LIMIT; i++ {
		if isPrime[i] {
			count++
		}
	}

	elapsed := time.Since(start)

	fmt.Printf("Primes up to %d: %d\n", LIMIT, count)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
