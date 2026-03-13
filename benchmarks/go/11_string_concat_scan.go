// String Concat and Scan Benchmark - Go Implementation

package main

import (
	"fmt"
	"strconv"
	"strings"
)

func main() {
	var builder strings.Builder
	builder.Grow(4096)

	for i := 0; i < 250; i++ {
		builder.WriteString("item=")
		builder.WriteString(strconv.Itoa(i))
		builder.WriteByte(';')
	}

	text := builder.String()
	var digits, equals, semicolons int64
	for _, ch := range text {
		if ch >= '0' && ch <= '9' {
			digits++
		} else if ch == '=' {
			equals++
		} else if ch == ';' {
			semicolons++
		}
	}

	fmt.Printf("string concat checksum: %d\n", int64(len(text))+digits+equals+semicolons)
}
