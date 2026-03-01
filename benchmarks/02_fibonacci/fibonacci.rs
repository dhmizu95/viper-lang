// Benchmark 02: Fibonacci Numbers
// Category: Integer Arithmetic / Recursion
// Tests: Loop performance, variable assignment, arithmetic

use std::time::Instant;

const ITERATIONS: i64 = 10_000_000; // 10 million iterations

fn main() {
    let start = Instant::now();

    let mut a: i64 = 0;
    let mut b: i64 = 1;
    let mut count: i64 = 0;

    for _ in 0..ITERATIONS {
        let temp = a + b;
        a = b;
        b = temp;
        count += 1;
    }

    let elapsed = start.elapsed();

    println!("Fibonacci iterations: {}", count);
    println!("Final value (last 10 digits): {}", a % 10_000_000_000);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
