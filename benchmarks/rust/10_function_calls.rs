// Function Call Overhead Benchmark - Rust Implementation

#[inline(always)]
fn mix(x: i64, y: i64) -> i64 {
    ((x * 3) + (y * 5) - (x % 7) + (y % 11)) % 1_000_003
}

fn main() {
    let n = 1_500_000i64;
    let mut acc = 0i64;

    for i in 0..n {
        acc += mix(i, acc + i);
    }

    println!("function call checksum: {}", acc);
}
