// Prime Sieve of Eratosthenes - Rust Implementation
use std::time::Instant;

fn sieve(n: i64) -> i64 {
    // Create sieve vector (true = potentially prime)
    let mut is_prime = vec![true; (n + 1) as usize];
    is_prime[0] = false;
    is_prime[1] = false;
    
    // Sieve of Eratosthenes
    let sqrt_n = (n as f64).sqrt() as i64;
    for i in 2..=sqrt_n {
        if is_prime[i as usize] {
            // Mark multiples as composite
            let mut j = i * i;
            while j <= n {
                is_prime[j as usize] = false;
                j += i;
            }
        }
    }
    
    // Count primes
    is_prime.iter().filter(|&&p| p).count() as i64
}

fn main() {
    println!("Prime Sieve Benchmark");
    println!("=====================");
    
    let sizes = [100000, 500000, 1000000, 5000000, 10000000];
    
    for &n in &sizes {
        println!("Sieving up to: {}", n);
        
        let start = Instant::now();
        let count = sieve(n);
        let elapsed = start.elapsed();
        
        println!("Primes found: {}", count);
        println!("Time: {:.2} ms", elapsed.as_secs_f64() * 1000.0);
        println!();
    }
    
    println!("Benchmark complete!");
}
