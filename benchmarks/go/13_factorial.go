// Factorial Benchmark - Go Implementation
// Recursive Factorial calculation

package main

import "fmt"

func fact(n int) int {
	if n <= 1 {
		return 1
	}
	return n * fact(n-1)
}

func main() {
	n := 15
	result := fact(n)
	fmt.Printf("fact(%d) = %d\n", n, result)
}