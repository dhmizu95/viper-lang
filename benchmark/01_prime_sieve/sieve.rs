// Benchmark 01: Prime Sieve (Eratosthenes)
// Category: Integer Arithmetic
// Tests: Array operations, basic arithmetic, memory access

use std::time::Instant;

const LIMIT: usize = 10_000_000; // 10 million

fn main() {
    let start = Instant::now();

    // Allocate sieve array
    let mut is_prime = vec![true; LIMIT + 1];
    is_prime[0] = false;
    is_prime[1] = false;

    // Sieve of Eratosthenes
    let sqrt_limit = (LIMIT as f64).sqrt() as usize;
    for p in 2..=sqrt_limit {
        if is_prime[p] {
            for i in (p * p..=LIMIT).step_by(p) {
                is_prime[i] = false;
            }
        }
    }

    // Count primes
    let count = is_prime.iter().filter(|&&x| x).count();

    let elapsed = start.elapsed();

    println!("Primes up to {}: {}", LIMIT, count);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
