/*
 * Matrix Multiplication 1000x1000 - Linear Algebra Benchmark
 * Tests memory bandwidth and vectorization
 */
use std::time::Instant;

const N: usize = 1000;

fn main() {
    println!("Computing matrix multiplication {}x{}...", N, N);
    
    // Initialize matrices with test data
    let mut a = vec![vec![0.0f64; N]; N];
    let mut b = vec![vec![0.0f64; N]; N];
    let mut c = vec![vec![0.0f64; N]; N];
    
    for i in 0..N {
        for j in 0..N {
            a[i][j] = ((i + j) % 100) as f64 / 100.0;
            b[i][j] = ((i * j) % 100) as f64 / 100.0;
        }
    }
    
    let start = Instant::now();
    
    // Standard O(n^3) matrix multiplication with cache-friendly ordering
    for i in 0..N {
        for k in 0..N {
            let a_ik = a[i][k];
            for j in 0..N {
                c[i][j] += a_ik * b[k][j];
            }
        }
    }
    
    let elapsed = start.elapsed();
    
    // Verification: compute checksum
    let checksum: f64 = c.iter().flat_map(|row| row.iter()).sum();
    
    println!("Checksum: {:.6}", checksum);
    println!("Time: {:?}", elapsed);
    let gflops = (2.0 * N as f64 * N as f64 * N as f64 / 1e9) / elapsed.as_secs_f64();
    println!("GFLOPS: {:.2}", gflops);
}
