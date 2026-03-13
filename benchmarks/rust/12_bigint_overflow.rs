// BigInt Overflow Path Benchmark - Rust Implementation

fn main() {
    let mut value: i128 = 1i128 << 100;
    let mut checksum: i64 = 0;

    for _ in 0..200_000 {
        value += 123_456_789;
        value -= 98_765_432;
        checksum += (value % 97) as i64;
    }

    println!("bigint overflow checksum: {}", checksum);
}
