// Integer Arithmetic Hot Loop Benchmark - Rust Implementation

fn main() {
    let n = 2_000_000i64;
    let mut acc = 1i64;

    for i in 1..=n {
        acc += i;
        acc -= i % 7;
        acc += (i * 3) % 11;
        if i % 5 == 0 {
            acc = acc / 2 + 17;
        }
    }

    println!("int hotloop checksum: {}", acc);
}
