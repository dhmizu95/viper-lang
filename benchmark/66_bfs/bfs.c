/*
 * BFS on Large Graph - Graph Theory Benchmark
 * Tests data structures and memory access patterns
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define NUM_NODES 10000000  // 10M nodes
#define AVG_EDGES 4

typedef struct {
    int *edges;
    int count;
    int capacity;
} Node;

static Node *graph;
static int *visited;
static int *queue;

void add_edge(int from, int to) {
    Node *node = &graph[from];
    if (node->count >= node->capacity) {
        node->capacity = node->capacity ? node->capacity * 2 : 8;
        node->edges = realloc(node->edges, node->capacity * sizeof(int));
    }
    node->edges[node->count++] = to;
}

int main() {
    printf("Building graph with %d nodes...\n", NUM_NODES);
    
    graph = calloc(NUM_NODES, sizeof(Node));
    visited = calloc(NUM_NODES, sizeof(int));
    queue = malloc(NUM_NODES * sizeof(int));
    
    // Build a structured graph (grid-like with some randomness)
    srand(42);
    for (int i = 0; i < NUM_NODES; i++) {
        // Connect to nearby nodes (simulating grid)
        for (int j = 1; j <= 2 && i + j < NUM_NODES; j++) {
            add_edge(i, i + j);
            add_edge(i + j, i);
        }
        // Add some random long-range edges
        if (rand() % 10 == 0) {
            int target = rand() % NUM_NODES;
            add_edge(i, target);
        }
    }
    
    printf("Running BFS from node 0...\n");
    
    clock_t start = clock();
    
    // BFS
    int head = 0, tail = 0;
    queue[tail++] = 0;
    visited[0] = 1;
    int visited_count = 0;
    
    while (head < tail) {
        int node = queue[head++];
        visited_count++;
        
        Node *n = &graph[node];
        for (int i = 0; i < n->count; i++) {
            int neighbor = n->edges[i];
            if (!visited[neighbor]) {
                visited[neighbor] = 1;
                queue[tail++] = neighbor;
            }
        }
    }
    
    clock_t end = clock();
    double elapsed = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Visited %d nodes\n", visited_count);
    printf("Time: %.4f seconds\n", elapsed);
    printf("Nodes/sec: %.0f\n", visited_count / elapsed);
    
    // Cleanup
    for (int i = 0; i < NUM_NODES; i++) {
        free(graph[i].edges);
    }
    free(graph);
    free(visited);
    free(queue);
    
    return 0;
}
