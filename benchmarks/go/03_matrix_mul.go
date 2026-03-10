// Matrix Multiplication Benchmark - Go Implementation
// Multiply two NxN matrices (matching Viper algorithm)

package main

import "fmt"

func main() {
	n := 30
	checksum := 0
	
	for row := 0; row < n; row++ {
		for col := 0; col < n; col++ {
			sum := 0
			for k := 0; k < n; k++ {
				a_val := (row*n + k) % 10
				b_val := (k*n + col + 1) % 10
				sum += a_val * b_val
			}
			checksum += sum
		}
	}
	
	fmt.Printf("matrix %dx%d checksum: %d\n", n, n, checksum)
}
