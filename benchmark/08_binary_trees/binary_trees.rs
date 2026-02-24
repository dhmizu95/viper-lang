// Benchmark 08: Binary Trees
// Category: Data Structures
// Tests: Tree traversal, recursion, memory allocation

use std::time::Instant;

const TREE_DEPTH: usize = 18;

struct Node {
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
    value: i32,
}

impl Node {
    fn new(value: i32) -> Self {
        Node {
            left: None,
            right: None,
            value,
        }
    }
}

fn build_tree(depth: usize, value: i32) -> Option<Box<Node>> {
    if depth == 0 {
        return None;
    }

    let mut node = Box::new(Node::new(value));
    node.left = build_tree(depth - 1, value * 2);
    node.right = build_tree(depth - 1, value * 2 + 1);
    Some(node)
}

fn count_nodes(node: &Option<Box<Node>>) -> usize {
    match node {
        None => 0,
        Some(n) => 1 + count_nodes(&n.left) + count_nodes(&n.right),
    }
}

fn sum_values(node: &Option<Box<Node>>) -> i64 {
    match node {
        None => 0,
        Some(n) => n.value as i64 + sum_values(&n.left) + sum_values(&n.right),
    }
}

fn main() {
    let start = Instant::now();

    // Build tree
    let mut root = build_tree(TREE_DEPTH, 1);

    // Count and sum
    let mut count = count_nodes(&root);
    let mut sum = sum_values(&root);

    // Free and rebuild multiple times
    for _ in 0..5 {
        drop(root);
        root = build_tree(TREE_DEPTH, 1);
        count = count_nodes(&root);
        sum = sum_values(&root);
    }

    drop(root);

    let elapsed = start.elapsed();

    println!("Tree depth: {}", TREE_DEPTH);
    println!("Node count: {}", count);
    println!("Sum of values: {}", sum);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
