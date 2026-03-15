// Recursive List Sum Benchmark - Rust Implementation
// Recursive sum of a range

fn sum_range(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    n + sum_range(n - 1)
}

fn main() {
    let n = 200;
    let result = sum_range(n);
    println!("{}", result);
}
