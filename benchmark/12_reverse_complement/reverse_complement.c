// Benchmark 12: Reverse Complement
// Category: String Processing / Bioinformatics
// Tests: String manipulation, I/O, character mapping

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <ctype.h>

#define SEQ_LEN 5000000  // 5 million bases

char complement(char c) {
    switch (toupper(c)) {
        case 'A': return 'T';
        case 'T': return 'A';
        case 'G': return 'C';
        case 'C': return 'G';
        default: return 'N';
    }
}

int main() {
    clock_t start = clock();
    
    // Generate random DNA sequence
    char* sequence = (char*)malloc(SEQ_LEN + 1);
    char* reverse_complement = (char*)malloc(SEQ_LEN + 1);
    
    if (!sequence || !reverse_complement) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    const char* bases = "ATGC";
    srand(42);
    
    for (int i = 0; i < SEQ_LEN; i++) {
        sequence[i] = bases[rand() % 4];
    }
    sequence[SEQ_LEN] = '\0';
    
    // Generate reverse complement
    for (int i = 0; i < SEQ_LEN; i++) {
        reverse_complement[i] = complement(sequence[SEQ_LEN - 1 - i]);
    }
    reverse_complement[SEQ_LEN] = '\0';
    
    // Verify by checking a few positions
    int verified = 1;
    for (int i = 0; i < 1000 && verified; i++) {
        char expected = complement(sequence[SEQ_LEN - 1 - i]);
        if (reverse_complement[i] != expected) {
            verified = 0;
        }
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Sequence length: %d\n", SEQ_LEN);
    printf("Verification: %s\n", verified ? "Passed" : "Failed");
    printf("First 50 bases of reverse complement: %.50s\n", reverse_complement);
    printf("Time: %.4f seconds\n", time_spent);
    
    free(sequence);
    free(reverse_complement);
    
    return 0;
}
