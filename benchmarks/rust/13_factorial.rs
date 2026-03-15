// Factorial Benchmark - Rust Implementation
// Recursive Factorial calculation

fn fact(n: i64) -> i64 {
    if n <= 1 {
        return 1;
    }
    n * fact(n - 1)
}

fn main() {
    let n: i64 = 15;
    let result = fact(n);
    println!("fact({}) = {}", n, result);
}