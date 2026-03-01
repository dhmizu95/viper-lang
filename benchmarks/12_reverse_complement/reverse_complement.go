// Benchmark 12: Reverse Complement
// Category: String Processing / Bioinformatics
// Tests: String manipulation, I/O, character mapping

package main

import (
	"fmt"
	"math/rand"
	"time"
)

const SEQ_LEN = 5000000 // 5 million bases

func complement(c byte) byte {
	switch c {
	case 'A', 'a':
		return 'T'
	case 'T', 't':
		return 'A'
	case 'G', 'g':
		return 'C'
	case 'C', 'c':
		return 'G'
	default:
		return 'N'
	}
}

func main() {
	start := time.Now()

	// Generate random DNA sequence
	bases := []byte("ATGC")
	sequence := make([]byte, SEQ_LEN)

	rand.Seed(42)
	for i := 0; i < SEQ_LEN; i++ {
		sequence[i] = bases[rand.Intn(4)]
	}

	// Generate reverse complement
	reverseComplement := make([]byte, SEQ_LEN)
	for i := 0; i < SEQ_LEN; i++ {
		reverseComplement[i] = complement(sequence[SEQ_LEN-1-i])
	}

	// Verify by checking a few positions
	verified := true
	for i := 0; i < 1000 && verified; i++ {
		expected := complement(sequence[SEQ_LEN-1-i])
		if reverseComplement[i] != expected {
			verified = false
		}
	}

	elapsed := time.Since(start)

	fmt.Printf("Sequence length: %d\n", SEQ_LEN)
	fmt.Printf("Verification: %v\n", verified)
	fmt.Printf("First 50 bases of reverse complement: %s\n", string(reverseComplement[:50]))
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
