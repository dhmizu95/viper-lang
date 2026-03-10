// Matrix Multiplication Benchmark - Rust Implementation
// Multiply two NxN matrices (matching Viper algorithm)

fn main() {
    let n = 30;
    let mut checksum: i64 = 0;
    
    for row in 0..n {
        for col in 0..n {
            let mut sum = 0;
            for k in 0..n {
                let a_val = (row * n + k) % 10;
                let b_val = (k * n + col + 1) % 10;
                sum += a_val * b_val;
            }
            checksum += sum as i64;
        }
    }
    
    println!("matrix {}x{} checksum: {}", n, n, checksum);
}
