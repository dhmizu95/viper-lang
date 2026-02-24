// Benchmark 05: Mandelbrot Set
// Category: Floating Point / Simulation
// Tests: Complex arithmetic, nested loops, floating-point comparisons

package main

import (
	"fmt"
	"time"
)

const (
	WIDTH    = 1000
	HEIGHT   = 1000
	MAX_ITER = 256
)

func main() {
	start := time.Now()

	count := 0

	// Mandelbrot calculation
	for py := 0; py < HEIGHT; py++ {
		for px := 0; px < WIDTH; px++ {
			// Map pixel to complex plane
			x0 := float64(px-WIDTH/2) * 4.0 / float64(WIDTH)
			y0 := float64(py-HEIGHT/2) * 4.0 / float64(HEIGHT)

			x, y := 0.0, 0.0
			iter := 0

			for x*x+y*y <= 4.0 && iter < MAX_ITER {
				xTemp := x*x - y*y + x0
				y = 2.0*x*y + y0
				x = xTemp
				iter++
			}

			if iter == MAX_ITER {
				count++
			}
		}
	}

	elapsed := time.Since(start)

	fmt.Printf("Image size: %dx%d\n", WIDTH, HEIGHT)
	fmt.Printf("Points in Mandelbrot set: %d\n", count)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
