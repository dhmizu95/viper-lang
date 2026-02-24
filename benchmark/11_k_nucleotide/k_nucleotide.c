// Benchmark 11: K-Nucleotide
// Category: String Processing / Bioinformatics
// Tests: String manipulation, hash tables, pattern matching

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define SEQ_LEN 100000
#define K 4

// Simple hash table for counting
typedef struct HashNode {
    char* key;
    int count;
    struct HashNode* next;
} HashNode;

#define HASH_SIZE 100003

HashNode* hash_table[HASH_SIZE];

unsigned int hash(const char* str, int len) {
    unsigned int h = 0;
    for (int i = 0; i < len; i++) {
        h = h * 31 + str[i];
    }
    return h % HASH_SIZE;
}

void insert(const char* key, int len) {
    unsigned int h = hash(key, len);
    
    // Check if exists
    HashNode* node = hash_table[h];
    while (node) {
        if (strncmp(node->key, key, len) == 0 && node->key[len] == '\0') {
            node->count++;
            return;
        }
        node = node->next;
    }
    
    // Insert new
    HashNode* new_node = (HashNode*)malloc(sizeof(HashNode));
    new_node->key = (char*)malloc(len + 1);
    strncpy(new_node->key, key, len);
    new_node->key[len] = '\0';
    new_node->count = 1;
    new_node->next = hash_table[h];
    hash_table[h] = new_node;
}

int main() {
    clock_t start = clock();
    
    // Generate random DNA sequence
    char* sequence = (char*)malloc(SEQ_LEN + 1);
    const char* bases = "ACGT";
    
    srand(42);  // Fixed seed for reproducibility
    for (int i = 0; i < SEQ_LEN; i++) {
        sequence[i] = bases[rand() % 4];
    }
    sequence[SEQ_LEN] = '\0';
    
    // Initialize hash table
    for (int i = 0; i < HASH_SIZE; i++) hash_table[i] = NULL;
    
    // Count all k-mers
    for (int i = 0; i <= SEQ_LEN - K; i++) {
        insert(sequence + i, K);
    }
    
    // Find most and least frequent
    int max_count = 0, min_count = SEQ_LEN + 1;
    char max_kmer[K + 1], min_kmer[K + 1];
    int total_unique = 0;
    
    for (int i = 0; i < HASH_SIZE; i++) {
        HashNode* node = hash_table[i];
        while (node) {
            total_unique++;
            if (node->count > max_count) {
                max_count = node->count;
                strncpy(max_kmer, node->key, K);
                max_kmer[K] = '\0';
            }
            if (node->count < min_count) {
                min_count = node->count;
                strncpy(min_kmer, node->key, K);
                min_kmer[K] = '\0';
            }
            node = node->next;
        }
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Sequence length: %d, K: %d\n", SEQ_LEN, K);
    printf("Unique %d-mers: %d\n", K, total_unique);
    printf("Most frequent: %s (%d times)\n", max_kmer, max_count);
    printf("Least frequent: %s (%d times)\n", min_kmer, min_count);
    printf("Time: %.4f seconds\n", time_spent);
    
    // Cleanup (omitted for brevity in benchmark)
    free(sequence);
    
    return 0;
}
