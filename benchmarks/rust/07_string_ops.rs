// String Operations Benchmark - Rust Implementation
// String concatenation and character scanning

fn main() {
    let text = "alpha,beta,gamma,delta;".repeat(400);

    let mut count_a = 0;
    let mut count_comma = 0;
    let mut count_semicolon = 0;

    for ch in text.chars() {
        if ch == 'a' {
            count_a += 1;
        } else if ch == ',' {
            count_comma += 1;
        } else if ch == ';' {
            count_semicolon += 1;
        }
    }

    let checksum = text.len() as i64 + count_a + count_comma + count_semicolon;
    println!("string operations checksum: {}", checksum);
}
