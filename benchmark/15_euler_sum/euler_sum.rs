// Benchmark 15: Euler Sum (Harmonic Series)
// Category: Floating Point / Numerical Analysis
// Tests: Floating-point summation, precision, Kahan summation

use std::time::Instant;

const N: usize = 100_000_000; // 100 million terms

// Naive summation
fn naive_sum(n: usize) -> f64 {
    let mut sum = 0.0;
    for i in 1..=n {
        sum += 1.0 / i as f64;
    }
    sum
}

// Kahan summation (compensated summation)
fn kahan_sum(n: usize) -> f64 {
    let mut sum = 0.0;
    let mut c = 0.0; // Running compensation for lost low-order bits

    for i in 1..=n {
        let y = 1.0 / i as f64 - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }
    sum
}

fn main() {
    let start = Instant::now();

    let naive_result = naive_sum(N);
    let kahan_result = kahan_sum(N);

    // Theoretical approximation: H(n) ≈ ln(n) + γ + 1/(2n)
    // where γ (Euler-Mascheroni constant) ≈ 0.5772156649015328606
    let euler_mascheroni: f64 = 0.5772156649015328606;
    let theoretical = (N as f64).ln() + euler_mascheroni + 1.0 / (2.0 * N as f64);

    let elapsed = start.elapsed();

    println!("Number of terms: {}", N);
    println!("Naive summation:     {:.15}", naive_result);
    println!("Kahan summation:     {:.15}", kahan_result);
    println!("Theoretical value:   {:.15}", theoretical);
    println!("Naive error:         {:.15}", (naive_result - theoretical).abs());
    println!("Kahan error:         {:.15}", (kahan_result - theoretical).abs());
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
