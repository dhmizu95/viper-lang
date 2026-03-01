// Benchmark 05: Mandelbrot Set
// Category: Floating Point / Simulation
// Tests: Complex arithmetic, nested loops, floating-point comparisons

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define WIDTH 1000
#define HEIGHT 1000
#define MAX_ITER 256

int main() {
    clock_t start = clock();
    
    int count = 0;
    
    // Mandelbrot calculation
    for (int py = 0; py < HEIGHT; py++) {
        for (int px = 0; px < WIDTH; px++) {
            // Map pixel to complex plane
            double x0 = (px - WIDTH / 2) * 4.0 / WIDTH;
            double y0 = (py - HEIGHT / 2) * 4.0 / HEIGHT;
            
            double x = 0.0, y = 0.0;
            int iter = 0;
            
            while (x * x + y * y <= 4.0 && iter < MAX_ITER) {
                double xtemp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xtemp;
                iter++;
            }
            
            if (iter == MAX_ITER) count++;
        }
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Image size: %dx%d\n", WIDTH, HEIGHT);
    printf("Points in Mandelbrot set: %d\n", count);
    printf("Time: %.4f seconds\n", time_spent);
    
    return 0;
}
