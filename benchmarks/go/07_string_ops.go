// String Operations Benchmark - Go Implementation
// String concatenation and character scanning

package main

import (
	"fmt"
	"strings"
)

func main() {
	text := strings.Repeat("alpha,beta,gamma,delta;", 400)

	var countA, countComma, countSemicolon int64
	for _, ch := range text {
		if ch == 'a' {
			countA++
		} else if ch == ',' {
			countComma++
		} else if ch == ';' {
			countSemicolon++
		}
	}

	checksum := int64(len(text)) + countA + countComma + countSemicolon
	fmt.Printf("string operations checksum: %d\n", checksum)
}
