/*
 * BFS on Large Graph - Graph Theory Benchmark
 * Tests data structures and memory access patterns
 */
use rand::{Rng, SeedableRng};
use std::collections::VecDeque;
use std::time::Instant;

const NUM_NODES: usize = 10_000_000; // 10M nodes

fn main() {
    println!("Building graph with {} nodes...", NUM_NODES);
    
    let mut graph: Vec<Vec<usize>> = vec![Vec::new(); NUM_NODES];
    
    // Build a structured graph (grid-like with some randomness)
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for i in 0..NUM_NODES {
        // Connect to nearby nodes
        for j in 1..=2 {
            if i + j < NUM_NODES {
                graph[i].push(i + j);
                graph[i + j].push(i);
            }
        }
        // Add some random long-range edges
        if rng.random_bool(0.1) {
            let target = rng.random_range(0..NUM_NODES);
            graph[i].push(target);
        }
    }
    
    println!("Running BFS from node 0...");
    
    let start = Instant::now();
    
    // BFS
    let mut visited = vec![false; NUM_NODES];
    let mut queue = VecDeque::new();
    queue.push_back(0);
    visited[0] = true;
    let mut visited_count = 0;
    
    while let Some(node) = queue.pop_front() {
        visited_count += 1;
        
        for &neighbor in &graph[node] {
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back(neighbor);
            }
        }
    }
    
    let elapsed = start.elapsed();
    
    println!("Visited {} nodes", visited_count);
    println!("Time: {:?}", elapsed);
    println!("Nodes/sec: {:.0}", visited_count as f64 / elapsed.as_secs_f64());
}
