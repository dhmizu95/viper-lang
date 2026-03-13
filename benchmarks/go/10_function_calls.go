// Function Call Overhead Benchmark - Go Implementation

package main

import "fmt"

func mix(x int64, y int64) int64 {
	return ((x * 3) + (y * 5) - (x % 7) + (y % 11)) % 1000003
}

func main() {
	const n int64 = 1500000
	var acc int64

	for i := int64(0); i < n; i++ {
		acc += mix(i, acc+i)
	}

	fmt.Printf("function call checksum: %d\n", acc)
}
