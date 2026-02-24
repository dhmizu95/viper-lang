// Prime Sieve of Eratosthenes - Go Implementation
package main

import (
	"fmt"
	"time"
)

func sieve(n int64) int64 {
	// Create sieve slice (true = potentially prime)
	isPrime := make([]bool, n+1)
	for i := range isPrime {
		isPrime[i] = true
	}
	isPrime[0] = false
	isPrime[1] = false

	// Sieve of Eratosthenes
	for i := int64(2); i*i <= n; i++ {
		if isPrime[i] {
			// Mark multiples as composite
			for j := i * i; j <= n; j += i {
				isPrime[j] = false
			}
		}
	}

	// Count primes
	var count int64 = 0
	for i := int64(2); i <= n; i++ {
		if isPrime[i] {
			count++
		}
	}
	return count
}

func main() {
	fmt.Println("Prime Sieve Benchmark")
	fmt.Println("=====================")

	sizes := []int64{100000, 500000, 1000000, 5000000, 10000000}

	for _, n := range sizes {
		fmt.Printf("Sieving up to: %d\n", n)

		start := time.Now()
		count := sieve(n)
		elapsed := time.Since(start)

		fmt.Printf("Primes found: %d\n", count)
		fmt.Printf("Time: %.2f ms\n\n", float64(elapsed.Nanoseconds())/1e6)
	}

	fmt.Println("Benchmark complete!")
}
