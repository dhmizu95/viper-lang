// Prime Count Benchmark - Rust Implementation
// Count primes using trial division (matches Viper algorithm)

fn is_prime(n: i64) -> i64 {
    if n < 2 { return 0; }
    if n == 2 { return 1; }
    if n % 2 == 0 { return 0; }
    
    let mut i = 3;
    while i * i <= n {
        if n % i == 0 { return 0; }
        i = i + 2;
    }
    return 1;
}

fn count_primes(n: i64) -> i64 {
    let mut count = 0;
    let mut i = 2;
    while i <= n {
        if is_prime(i) == 1 { count += 1; }
        i = i + 1;
    }
    count
}

fn main() {
    let n = 5000;
    let result = count_primes(n);
    println!("primes up to {}: {}", n, result);
}
