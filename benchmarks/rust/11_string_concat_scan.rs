// String Concat and Scan Benchmark - Rust Implementation

use std::fmt::Write;

fn main() {
    let n = 250;
    let mut text = String::with_capacity(4096);

    for i in 0..n {
        let _ = write!(&mut text, "item={};", i);
    }

    let mut digits = 0i64;
    let mut equals = 0i64;
    let mut semicolons = 0i64;
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits += 1;
        } else if ch == '=' {
            equals += 1;
        } else if ch == ';' {
            semicolons += 1;
        }
    }

    println!(
        "string concat checksum: {}",
        text.len() as i64 + digits + equals + semicolons
    );
}
