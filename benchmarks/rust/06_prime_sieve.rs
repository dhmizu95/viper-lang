// Prime Sieve Benchmark - Rust Implementation
// Sieve of Eratosthenes algorithm

fn main() {
    // Benchmark parameter - find primes up to n
    let n = 10000;
    
    // Initialize sieve (true = potentially prime)
    let mut sieve = vec![true; n + 1];
    sieve[0] = false;
    sieve[1] = false;
    
    // Sieve of Eratosthenes
    let mut p = 2;
    while p * p <= n {
        if sieve[p] {
            // Mark all multiples of p as not prime
            let mut i = p * p;
            while i <= n {
                sieve[i] = false;
                i += p;
            }
        }
        p += 1;
    }
    
    // Count primes
    let count = sieve.iter().filter(|&&is_prime| is_prime).count();
    println!("primes up to {}: {}", n, count);
}
