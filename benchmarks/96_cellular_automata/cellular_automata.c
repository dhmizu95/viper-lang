/*
 * Cellular Automata (Game of Life) - Simulation Benchmark
 * Tests array operations and parallel computation patterns
 * Uses large grid (4096x4096)
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#define SIZE 4096
#define GENERATIONS 100

static unsigned char grid[SIZE][SIZE];
static unsigned char next_grid[SIZE][SIZE];

void initialize() {
    srand(42);
    for (int i = 0; i < SIZE; i++) {
        for (int j = 0; j < SIZE; j++) {
            grid[i][j] = rand() % 2;
        }
    }
}

int count_neighbors(int x, int y) {
    int count = 0;
    for (int dx = -1; dx <= 1; dx++) {
        for (int dy = -1; dy <= 1; dy++) {
            if (dx == 0 && dy == 0) continue;
            int nx = (x + dx + SIZE) % SIZE;
            int ny = (y + dy + SIZE) % SIZE;
            count += grid[nx][ny];
        }
    }
    return count;
}

void step() {
    for (int i = 1; i < SIZE - 1; i++) {
        for (int j = 1; j < SIZE - 1; j++) {
            int neighbors = count_neighbors(i, j);
            if (grid[i][j]) {
                next_grid[i][j] = (neighbors == 2 || neighbors == 3) ? 1 : 0;
            } else {
                next_grid[i][j] = (neighbors == 3) ? 1 : 0;
            }
        }
    }
    memcpy(grid, next_grid, SIZE * SIZE);
}

int main() {
    printf("Running Game of Life on %dx%d grid for %d generations...\n", 
           SIZE, SIZE, GENERATIONS);
    
    initialize();
    
    // Count initial population
    long initial_pop = 0;
    for (int i = 0; i < SIZE; i++) {
        for (int j = 0; j < SIZE; j++) {
            initial_pop += grid[i][j];
        }
    }
    printf("Initial population: %ld\n", initial_pop);
    
    clock_t start = clock();
    
    for (int gen = 0; gen < GENERATIONS; gen++) {
        step();
        if ((gen + 1) % 10 == 0) {
            printf("Generation %d...\n", gen + 1);
        }
    }
    
    clock_t end = clock();
    double elapsed = (double)(end - start) / CLOCKS_PER_SEC;
    
    // Count final population
    long final_pop = 0;
    for (int i = 0; i < SIZE; i++) {
        for (int j = 0; j < SIZE; j++) {
            final_pop += grid[i][j];
        }
    }
    
    printf("Final population: %ld\n", final_pop);
    printf("Time: %.4f seconds\n", elapsed);
    printf("Cells/sec: %.0f\n", (long long)SIZE * SIZE * GENERATIONS / elapsed);
    
    return 0;
}
