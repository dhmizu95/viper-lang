/*
 * Fibonacci(1,000,000) - Big Integer Benchmark
 * Tests arbitrary precision arithmetic and iterative computation
 */
use num_bigint::BigUint;
use std::time::Instant;

fn main() {
    const N: usize = 1_000_000;
    
    println!("Computing Fibonacci({})...", N);
    
    let start = Instant::now();
    
    let mut a = BigUint::from(0u32);
    let mut b = BigUint::from(1u32);
    
    for _ in 0..N {
        let temp = a.clone();
        a = &a + &b;
        b = temp;
    }
    
    let elapsed = start.elapsed();
    
    // Get number of digits for verification
    let str = a.to_string();
    let digits = str.len();
    println!("Fibonacci({}) has {} digits", N, digits);
    
    // Get first and last digits for verification
    println!("First 10 digits: {}...", &str[..10]);
    println!("Last 10 digits: {}...", &str[digits-10..]);
    
    println!("Time: {:?}", elapsed);
}
