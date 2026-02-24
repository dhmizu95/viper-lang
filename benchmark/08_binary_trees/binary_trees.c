// Benchmark 08: Binary Trees
// Category: Data Structures
// Tests: Tree traversal, recursion, memory allocation

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define TREE_DEPTH 18

typedef struct Node {
    struct Node* left;
    struct Node* right;
    int value;
} Node;

Node* create_node(int value) {
    Node* node = (Node*)malloc(sizeof(Node));
    node->left = NULL;
    node->right = NULL;
    node->value = value;
    return node;
}

Node* build_tree(int depth, int value) {
    if (depth <= 0) return NULL;
    
    Node* node = create_node(value);
    node->left = build_tree(depth - 1, value * 2);
    node->right = build_tree(depth - 1, value * 2 + 1);
    return node;
}

int count_nodes(Node* node) {
    if (node == NULL) return 0;
    return 1 + count_nodes(node->left) + count_nodes(node->right);
}

long long sum_values(Node* node) {
    if (node == NULL) return 0;
    return node->value + sum_values(node->left) + sum_values(node->right);
}

void free_tree(Node* node) {
    if (node == NULL) return;
    free_tree(node->left);
    free_tree(node->right);
    free(node);
}

int main() {
    clock_t start = clock();
    
    // Build tree
    Node* root = build_tree(TREE_DEPTH, 1);
    
    // Count and sum
    int count = count_nodes(root);
    long long sum = sum_values(root);
    
    // Free and rebuild multiple times
    for (int i = 0; i < 5; i++) {
        free_tree(root);
        root = build_tree(TREE_DEPTH, 1);
        count = count_nodes(root);
        sum = sum_values(root);
    }
    
    free_tree(root);
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Tree depth: %d\n", TREE_DEPTH);
    printf("Node count: %d\n", count);
    printf("Sum of values: %lld\n", sum);
    printf("Time: %.4f seconds\n", time_spent);
    
    return 0;
}
