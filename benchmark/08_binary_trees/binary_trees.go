// Benchmark 08: Binary Trees
// Category: Data Structures
// Tests: Tree traversal, recursion, memory allocation

package main

import (
	"fmt"
	"time"
)

const TREE_DEPTH = 18

type Node struct {
	Left  *Node
	Right *Node
	Value int
}

func createNode(value int) *Node {
	return &Node{Value: value}
}

func buildTree(depth, value int) *Node {
	if depth <= 0 {
		return nil
	}
	node := createNode(value)
	node.Left = buildTree(depth-1, value*2)
	node.Right = buildTree(depth-1, value*2+1)
	return node
}

func countNodes(node *Node) int {
	if node == nil {
		return 0
	}
	return 1 + countNodes(node.Left) + countNodes(node.Right)
}

func sumValues(node *Node) int64 {
	if node == nil {
		return 0
	}
	return int64(node.Value) + sumValues(node.Left) + sumValues(node.Right)
}

func main() {
	start := time.Now()

	// Build tree
	root := buildTree(TREE_DEPTH, 1)

	// Count and sum
	count := countNodes(root)
	sum := sumValues(root)

	// Free and rebuild multiple times (Go handles GC automatically)
	for i := 0; i < 5; i++ {
		root = buildTree(TREE_DEPTH, 1)
		count = countNodes(root)
		sum = sumValues(root)
	}

	elapsed := time.Since(start)

	fmt.Printf("Tree depth: %d\n", TREE_DEPTH)
	fmt.Printf("Node count: %d\n", count)
	fmt.Printf("Sum of values: %d\n", sum)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
