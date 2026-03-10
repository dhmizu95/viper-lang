// String Operations Benchmark - Go Implementation
// Character counting in byte array

package main

import "fmt"

const N = 9000

func main() {
	// Create array with repeating pattern
	s := make([]int64, N)
	for i := 0; i < N; i++ {
		s[i] = (int64(i)*7 + 3) % 128
	}

	// Count occurrences of specific values
	var count1, count2, count3, count4 int64 = 0, 0, 0, 0
	for i := 0; i < N; i++ {
		if s[i] == 65 {
			count1++
		} else if s[i] == 66 {
			count2++
		} else if s[i] == 67 {
			count3++
		} else if s[i] == 68 {
			count4++
		}
	}

	// Calculate checksum
	checksum := int64(N) + count1 + count2 + count3 + count4
	fmt.Printf("string operations checksum: %d\n", checksum)
}
