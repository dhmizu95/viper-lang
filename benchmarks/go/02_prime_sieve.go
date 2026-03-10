// Prime Count Benchmark - Go Implementation
// Count primes using trial division (matches Viper algorithm)

package main

import "fmt"

func isPrime(n int) int {
	if n < 2 { return 0 }
	if n == 2 { return 1 }
	if n % 2 == 0 { return 0 }
	
	i := 3
	for i * i <= n {
		if n % i == 0 { return 0 }
		i = i + 2
	}
	return 1
}

func countPrimes(n int) int {
	count := 0
	i := 2
	for i <= n {
		if isPrime(i) == 1 { count++ }
		i = i + 1
	}
	return count
}

func main() {
	n := 5000
	result := countPrimes(n)
	fmt.Printf("primes up to %d: %d\n", n, result)
}
