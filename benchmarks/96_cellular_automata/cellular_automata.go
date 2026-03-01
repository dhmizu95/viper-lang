/*
 * Cellular Automata (Game of Life) - Simulation Benchmark
 * Tests array operations and parallel computation patterns
 * Uses large grid (4096x4096)
 */
package main

import (
	"fmt"
	"math/rand"
	"time"
)

const SIZE = 4096
const GENERATIONS = 100

func main() {
	fmt.Printf("Running Game of Life on %dx%d grid for %d generations...\n",
		SIZE, SIZE, GENERATIONS)
	
	// Initialize grids
	grid := make([][]byte, SIZE)
	nextGrid := make([][]byte, SIZE)
	for i := range grid {
		grid[i] = make([]byte, SIZE)
		nextGrid[i] = make([]byte, SIZE)
	}
	
	rng := rand.New(rand.NewSource(42))
	for i := 0; i < SIZE; i++ {
		for j := 0; j < SIZE; j++ {
			grid[i][j] = byte(rng.Intn(2))
		}
	}
	
	// Count initial population
	initialPop := 0
	for i := 0; i < SIZE; i++ {
		for j := 0; j < SIZE; j++ {
			initialPop += int(grid[i][j])
		}
	}
	fmt.Printf("Initial population: %d\n", initialPop)
	
	start := time.Now()
	
	// Run simulation
	for gen := 0; gen < GENERATIONS; gen++ {
		for i := 1; i < SIZE-1; i++ {
			for j := 1; j < SIZE-1; j++ {
				neighbors := 0
				for dx := -1; dx <= 1; dx++ {
					for dy := -1; dy <= 1; dy++ {
						if dx == 0 && dy == 0 {
							continue
						}
						nx := (i + dx + SIZE) % SIZE
						ny := (j + dy + SIZE) % SIZE
						neighbors += int(grid[nx][ny])
					}
				}
				if grid[i][j] == 1 {
					if neighbors == 2 || neighbors == 3 {
						nextGrid[i][j] = 1
					} else {
						nextGrid[i][j] = 0
					}
				} else {
					if neighbors == 3 {
						nextGrid[i][j] = 1
					} else {
						nextGrid[i][j] = 0
					}
				}
			}
		}
		// Swap grids
		grid, nextGrid = nextGrid, grid
		
		if (gen+1)%10 == 0 {
			fmt.Printf("Generation %d...\n", gen+1)
		}
	}
	
	elapsed := time.Since(start)
	
	// Count final population
	finalPop := 0
	for i := 0; i < SIZE; i++ {
		for j := 0; j < SIZE; j++ {
			finalPop += int(grid[i][j])
		}
	}
	
	fmt.Printf("Final population: %d\n", finalPop)
	fmt.Printf("Time: %v\n", elapsed)
	fmt.Printf("Cells/sec: %.0f\n", float64(SIZE*SIZE*GENERATIONS)/elapsed.Seconds())
}
