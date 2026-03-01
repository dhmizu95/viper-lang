// Benchmark 14: Champernowne Constant
// Category: Number Theory / String Processing
// Tests: Number to string conversion, string concatenation, digit counting

use std::time::Instant;

const MAX_N: usize = 1_000_000; // Concatenate numbers 1 to 1,000,000

fn main() {
    let start = Instant::now();

    // Build buffer using String
    let mut buffer = String::with_capacity(6_000_000);

    // Concatenate all numbers from 1 to MAX_N
    for i in 1..=MAX_N {
        buffer.push_str(&i.to_string());
    }

    // Count each digit 0-9
    let mut digit_counts = [0usize; 10];
    for c in buffer.chars() {
        digit_counts[c.to_digit(10).unwrap() as usize] += 1;
    }

    let elapsed = start.elapsed();

    println!("Numbers concatenated: 1 to {}", MAX_N);
    println!("Total length: {} characters", buffer.len());
    println!("Digit counts:");
    for d in 0..10 {
        println!("  {}: {}", d, digit_counts[d]);
    }
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
