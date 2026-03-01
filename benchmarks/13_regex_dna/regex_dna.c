// Benchmark 13: Regex DNA Matching
// Category: String Processing / Pattern Matching
// Tests: Pattern matching, string scanning

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <ctype.h>

#define SEQ_LEN 1000000  // 1 million bases

// Simple pattern matching (since we can't use regex library for fair comparison)
// Pattern: Find all occurrences of "ATG" (start codon) followed by any 3 bases, then "TAA", "TAG", or "TGA" (stop codons)

int match_pattern(const char* seq, int len, int pos) {
    // Check for start codon ATG
    if (pos + 11 > len) return 0;
    if (seq[pos] != 'A' || seq[pos+1] != 'T' || seq[pos+2] != 'G') return 0;
    
    // Check for stop codon at position + 6 to + 8 (after 3 any bases)
    char stop[4];
    strncpy(stop, seq + pos + 6, 3);
    stop[3] = '\0';
    
    if (strcmp(stop, "TAA") == 0 || strcmp(stop, "TAG") == 0 || strcmp(stop, "TGA") == 0) {
        return 1;
    }
    return 0;
}

int main() {
    clock_t start = clock();
    
    // Generate random DNA sequence
    char* sequence = (char*)malloc(SEQ_LEN + 1);
    const char* bases = "ATGC";
    
    srand(42);
    for (int i = 0; i < SEQ_LEN; i++) {
        sequence[i] = bases[rand() % 4];
    }
    sequence[SEQ_LEN] = '\0';
    
    // Find pattern matches
    int match_count = 0;
    for (int i = 0; i < SEQ_LEN - 11; i++) {
        if (match_pattern(sequence, SEQ_LEN, i)) {
            match_count++;
        }
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Sequence length: %d\n", SEQ_LEN);
    printf("Pattern matches found: %d\n", match_count);
    printf("Time: %.4f seconds\n", time_spent);
    
    free(sequence);
    
    return 0;
}
