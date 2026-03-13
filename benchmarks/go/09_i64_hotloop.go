// Fixed-Width i64 Arithmetic Hot Loop Benchmark - Go Implementation

package main

import "fmt"

func main() {
	const n int64 = 2000000
	var acc int64 = 1

	for i := int64(1); i <= n; i++ {
		acc += i
		acc -= i % 7
		acc += (i * 3) % 11
		if i%5 == 0 {
			acc = acc/2 + 17
		}
	}

	fmt.Printf("i64 hotloop checksum: %d\n", acc)
}
