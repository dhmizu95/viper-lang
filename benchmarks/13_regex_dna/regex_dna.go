// Benchmark 13: Regex DNA Matching
// Category: String Processing / Pattern Matching
// Tests: Pattern matching, string scanning

package main

import (
	"fmt"
	"math/rand"
	"time"
)

const SEQ_LEN = 1000000 // 1 million bases

// Pattern: Find all occurrences of "ATG" (start codon) followed by any 3 bases, then "TAA", "TAG", or "TGA" (stop codons)
func matchPattern(seq []byte, pos int) bool {
	if pos+9 > len(seq) {
		return false
	}

	// Check for start codon ATG
	if seq[pos] != 'A' || seq[pos+1] != 'T' || seq[pos+2] != 'G' {
		return false
	}

	// Check for stop codon at position + 6 to + 8 (after 3 any bases)
	stop := string(seq[pos+6 : pos+9])
	return stop == "TAA" || stop == "TAG" || stop == "TGA"
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

	// Find pattern matches
	matchCount := 0
	for i := 0; i < SEQ_LEN-9; i++ {
		if matchPattern(sequence, i) {
			matchCount++
		}
	}

	elapsed := time.Since(start)

	fmt.Printf("Sequence length: %d\n", SEQ_LEN)
	fmt.Printf("Pattern matches found: %d\n", matchCount)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
