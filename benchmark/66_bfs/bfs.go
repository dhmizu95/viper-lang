/*
 * BFS on Large Graph - Graph Theory Benchmark
 * Tests data structures and memory access patterns
 */
package main

import (
	"fmt"
	"math/rand"
	"time"
)

const NUM_NODES = 10000000 // 10M nodes

type Node struct {
	edges []int
}

func main() {
	fmt.Printf("Building graph with %d nodes...\n", NUM_NODES)
	
	graph := make([]Node, NUM_NODES)
	visited := make([]bool, NUM_NODES)
	queue := make([]int, NUM_NODES)
	
	// Build a structured graph (grid-like with some randomness)
	rng := rand.New(rand.NewSource(42))
	for i := 0; i < NUM_NODES; i++ {
		// Connect to nearby nodes
		for j := 1; j <= 2 && i+j < NUM_NODES; j++ {
			graph[i].edges = append(graph[i].edges, i+j)
			graph[i+j].edges = append(graph[i+j].edges, i)
		}
		// Add some random long-range edges
		if rng.Intn(10) == 0 {
			target := rng.Intn(NUM_NODES)
			graph[i].edges = append(graph[i].edges, target)
		}
	}
	
	fmt.Println("Running BFS from node 0...")
	
	start := time.Now()
	
	// BFS
	head, tail := 0, 0
	queue[tail] = 0
	visited[0] = true
	tail++
	visitedCount := 0
	
	for head < tail {
		node := queue[head]
		head++
		visitedCount++
		
		for _, neighbor := range graph[node].edges {
			if !visited[neighbor] {
				visited[neighbor] = true
				queue[tail] = neighbor
				tail++
			}
		}
	}
	
	elapsed := time.Since(start)
	
	fmt.Printf("Visited %d nodes\n", visitedCount)
	fmt.Printf("Time: %v\n", elapsed)
	fmt.Printf("Nodes/sec: %.0f\n", float64(visitedCount)/elapsed.Seconds())
}
