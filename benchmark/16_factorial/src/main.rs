/*
 * Factorial(1,000,000) - Big Integer Benchmark
 * Tests arbitrary precision arithmetic and memory management
 */
use num_bigint::BigUint;
use num_traits::One;
use std::time::Instant;

fn main() {
    const N: u64 = 1_000_000;
    
    println!("Computing factorial({})...", N);
    
    let start = Instant::now();
    
    let mut result = BigUint::one();
    for i in 2u64..=N {
        result *= i;
    }
    
    let elapsed = start.elapsed();
    
    // Get number of digits for verification
    let str = result.to_string();
    let digits = str.len();
    println!("Factorial({}) has {} digits", N, digits);
    
    // Get first and last digits for verification
    println!("First 10 digits: {}...", &str[..10]);
    println!("Last 10 digits: {}...", &str[digits-10..]);
    
    println!("Time: {:?}", elapsed);
}
