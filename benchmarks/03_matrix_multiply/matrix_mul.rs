// Benchmark 03: Matrix Multiplication
// Category: Linear Algebra
// Tests: Nested loops, array access, floating-point operations

use std::time::Instant;

const SIZE: usize = 512; // 512x512 matrices

fn main() {
    let start = Instant::now();

    // Allocate matrices
    let mut a = vec![0.0f64; SIZE * SIZE];
    let mut b = vec![0.0f64; SIZE * SIZE];
    let mut c = vec![0.0f64; SIZE * SIZE];

    // Initialize matrices
    for i in 0..SIZE * SIZE {
        a[i] = (i % 100) as f64 / 100.0;
        b[i] = (i % 50) as f64 / 50.0;
    }

    // Matrix multiplication C = A * B
    for i in 0..SIZE {
        for j in 0..SIZE {
            let mut sum = 0.0;
            for k in 0..SIZE {
                sum += a[i * SIZE + k] * b[k * SIZE + j];
            }
            c[i * SIZE + j] = sum;
        }
    }

    let elapsed = start.elapsed();

    // Verify result (sum of first row)
    let verify: f64 = c[0..SIZE].iter().sum();

    println!("Matrix size: {}x{}", SIZE, SIZE);
    println!("Verification sum: {:.6}", verify);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
