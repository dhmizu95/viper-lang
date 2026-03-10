// Matrix Multiplication Benchmark - Rust Implementation
// Multiply two NxN matrices using arrays

fn main() {
    // Benchmark parameter
    let n = 50;
    
    // Initialize matrices
    let mut a = vec![0i64; n * n];
    let mut b = vec![0i64; n * n];
    let mut c = vec![0i64; n * n];
    
    // Fill matrices with values
    for i in 0..n {
        for j in 0..n {
            let idx = i * n + j;
            a[idx] = ((i + j) % 10) as i64;
            b[idx] = ((i * j) % 10) as i64;
        }
    }
    
    // Matrix multiplication: C = A * B
    for i in 0..n {
        for j in 0..n {
            let mut sum = 0;
            for k in 0..n {
                let a_idx = i * n + k;
                let b_idx = k * n + j;
                sum += a[a_idx] * b[b_idx];
            }
            let c_idx = i * n + j;
            c[c_idx] = sum;
        }
    }
    
    // Calculate checksum
    let checksum: i64 = c.iter().sum();
    println!("matrix {}x{} checksum: {}", n, n, checksum);
}
