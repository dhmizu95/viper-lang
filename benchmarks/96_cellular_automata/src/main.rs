/*
 * Cellular Automata (Game of Life) - Simulation Benchmark
 * Tests array operations and parallel computation patterns
 * Uses large grid (4096x4096)
 */
use rand::{Rng, SeedableRng};
use std::time::Instant;

const SIZE: usize = 4096;
const GENERATIONS: usize = 100;

fn main() {
    println!("Running Game of Life on {}x{} grid for {} generations...",
             SIZE, SIZE, GENERATIONS);
    
    // Initialize grids
    let mut grid = vec![vec![0u8; SIZE]; SIZE];
    let mut next_grid = vec![vec![0u8; SIZE]; SIZE];
    
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    for i in 0..SIZE {
        for j in 0..SIZE {
            grid[i][j] = if rng.random_bool(0.5) { 1 } else { 0 };
        }
    }
    
    // Count initial population
    let initial_pop: usize = grid.iter().flat_map(|row| row.iter()).map(|&x| x as usize).sum();
    println!("Initial population: {}", initial_pop);
    
    let start = Instant::now();
    
    // Run simulation
    for gen in 0..GENERATIONS {
        for i in 1..SIZE-1 {
            for j in 1..SIZE-1 {
                let mut neighbors = 0;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (i as isize + dx + SIZE as isize) as usize % SIZE;
                        let ny = (j as isize + dy + SIZE as isize) as usize % SIZE;
                        neighbors += grid[nx][ny] as usize;
                    }
                }
                next_grid[i][j] = if grid[i][j] == 1 {
                    if neighbors == 2 || neighbors == 3 { 1 } else { 0 }
                } else {
                    if neighbors == 3 { 1 } else { 0 }
                };
            }
        }
        // Swap grids
        std::mem::swap(&mut grid, &mut next_grid);
        
        if (gen + 1) % 10 == 0 {
            println!("Generation {}...", gen + 1);
        }
    }
    
    let elapsed = start.elapsed();
    
    // Count final population
    let final_pop: usize = grid.iter().flat_map(|row| row.iter()).map(|&x| x as usize).sum();
    
    println!("Final population: {}", final_pop);
    println!("Time: {:?}", elapsed);
    println!("Cells/sec: {:.0}", (SIZE * SIZE * GENERATIONS) as f64 / elapsed.as_secs_f64());
}
