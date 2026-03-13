// Fixed-Width i64 Arithmetic Hot Loop Benchmark - Rust Implementation

fn main() {
    let n: i64 = 2_000_000;
    let mut acc: i64 = 1;

    for i in 1..=n {
        acc += i;
        acc -= i % 7;
        acc += (i * 3) % 11;
        if i % 5 == 0 {
            acc = acc / 2 + 17;
        }
    }

    println!("i64 hotloop checksum: {}", acc);
}
