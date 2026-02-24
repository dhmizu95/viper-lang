// Benchmark 10: Spectral Norm
// Category: Linear Algebra
// Tests: Matrix-vector operations, power iteration

use std::time::Instant;

const N: usize = 1500;
const ITERATIONS: usize = 10;

// A[i][j] = 1 / (i + j + 1)
fn a_element(i: usize, j: usize) -> f64 {
    1.0 / (i + j + 1) as f64
}

// Multiply by A
fn av(v: &[f64], av: &mut [f64]) {
    for i in 0..N {
        av[i] = (0..N).map(|j| a_element(i, j) * v[j]).sum();
    }
}

// Multiply by A^T
fn atv(v: &[f64], atv: &mut [f64]) {
    for i in 0..N {
        atv[i] = (0..N).map(|j| a_element(j, i) * v[j]).sum();
    }
}

// Multiply by A^T * A
fn atav(v: &[f64], result: &mut [f64]) {
    let mut temp = vec![0.0; N];
    av(v, &mut temp);
    atv(&temp, result);
}

fn main() {
    let start = Instant::now();

    let mut u = vec![1.0; N];
    let mut v = vec![0.0; N];

    // Power iteration
    for _ in 0..ITERATIONS {
        atav(&u, &mut v);

        // Normalize
        let norm: f64 = v.iter().map(|&x| x * x).sum::<f64>().sqrt();

        for i in 0..N {
            u[i] = v[i] / norm;
        }
    }

    // Calculate spectral norm approximation
    let mut av_result = vec![0.0; N];
    av(&u, &mut av_result);

    let spectral_norm: f64 = u.iter().zip(av_result.iter()).map(|(&a, &b)| a * b).sum();
    let spectral_norm = spectral_norm.sqrt();

    let elapsed = start.elapsed();

    println!("Matrix size: {}x{}", N, N);
    println!("Iterations: {}", ITERATIONS);
    println!("Spectral norm: {:.10}", spectral_norm);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
