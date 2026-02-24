// Benchmark 11: K-Nucleotide
// Category: String Processing / Bioinformatics
// Tests: String manipulation, hash tables, pattern matching

package main

import (
	"fmt"
	"math/rand"
	"time"
)

const (
	SEQ_LEN = 100000
	K       = 4
)

func main() {
	start := time.Now()

	// Generate random DNA sequence
	bases := []byte("ACGT")
	sequence := make([]byte, SEQ_LEN)

	rand.Seed(42)
	for i := 0; i < SEQ_LEN; i++ {
		sequence[i] = bases[rand.Intn(4)]
	}

	// Count all k-mers using map
	kmerCounts := make(map[string]int)

	for i := 0; i <= SEQ_LEN-K; i++ {
		kmer := string(sequence[i : i+K])
		kmerCounts[kmer]++
	}

	// Find most and least frequent
	maxCount := 0
	minCount := SEQ_LEN + 1
	var maxKmer, minKmer string
	totalUnique := len(kmerCounts)

	for kmer, count := range kmerCounts {
		if count > maxCount {
			maxCount = count
			maxKmer = kmer
		}
		if count < minCount {
			minCount = count
			minKmer = kmer
		}
	}

	elapsed := time.Since(start)

	fmt.Printf("Sequence length: %d, K: %d\n", SEQ_LEN, K)
	fmt.Printf("Unique %d-mers: %d\n", K, totalUnique)
	fmt.Printf("Most frequent: %s (%d times)\n", maxKmer, maxCount)
	fmt.Printf("Least frequent: %s (%d times)\n", minKmer, minCount)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
