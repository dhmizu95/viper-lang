// Recursive List Sum Benchmark - Go Implementation
// Recursive sum of a slice

package main

import "fmt"

func sumSlice(slice []int, idx int) int {
	if idx >= len(slice) {
		return 0
	}
	return slice[idx] + sumSlice(slice, idx+1)
}

func main() {
	slice := make([]int, 1000)
	for i := 0; i < 1000; i++ {
		slice[i] = i + 1
	}
	
	result := sumSlice(slice, 0)
	fmt.Println(result)
}