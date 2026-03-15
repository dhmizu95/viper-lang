// Recursive List Sum Benchmark - Go Implementation
// Recursive sum of a range

package main

import "fmt"

func sumRange(n int) int {
	if n <= 0 {
		return 0
	}
	return n + sumRange(n-1)
}

func main() {
	n := 200
	result := sumRange(n)
	fmt.Println(result)
}
