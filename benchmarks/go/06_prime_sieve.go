// Prime Sieve Benchmark - Go Implementation
// Sieve of Eratosthenes algorithm

package main

import "fmt"

func main() {
	// Benchmark parameter - find primes up to n
	n := 10000

	// Initialize sieve (true = potentially prime)
	sieve := make([]bool, n+1)
	for i := range sieve {
		sieve[i] = true
	}
	sieve[0] = false
	sieve[1] = false

	// Sieve of Eratosthenes
	for p := 2; p*p <= n; p++ {
		if sieve[p] {
			// Mark all multiples of p as not prime
			for i := p * p; i <= n; i += p {
				sieve[i] = false
			}
		}
	}

	// Count primes
	count := 0
	for _, isPrime := range sieve {
		if isPrime {
			count++
		}
	}

	fmt.Printf("primes up to %d: %d\n", n, count)
}
