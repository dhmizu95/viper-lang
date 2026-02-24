// Benchmark 05: Mandelbrot Set
// Category: Floating Point / Simulation
// Tests: Complex arithmetic, nested loops, floating-point comparisons

use std::time::Instant;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const MAX_ITER: usize = 256;

fn main() {
    let start = Instant::now();

    let mut count = 0;

    // Mandelbrot calculation
    for py in 0..HEIGHT {
        for px in 0..WIDTH {
            // Map pixel to complex plane
            let x0 = (px as f64 - WIDTH as f64 / 2.0) * 4.0 / WIDTH as f64;
            let y0 = (py as f64 - HEIGHT as f64 / 2.0) * 4.0 / HEIGHT as f64;

            let mut x = 0.0f64;
            let mut y = 0.0f64;
            let mut iter = 0;

            while x * x + y * y <= 4.0 && iter < MAX_ITER {
                let x_temp = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = x_temp;
                iter += 1;
            }

            if iter == MAX_ITER {
                count += 1;
            }
        }
    }

    let elapsed = start.elapsed();

    println!("Image size: {}x{}", WIDTH, HEIGHT);
    println!("Points in Mandelbrot set: {}", count);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
