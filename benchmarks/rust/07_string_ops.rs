// String Operations Benchmark - Rust Implementation
// Character counting in byte array

fn main() {
    // Benchmark parameter
    let n = 9000;
    
    // Create array with repeating pattern
    let mut s = vec![0i64; n];
    for i in 0..n {
        s[i] = ((i as i64 * 7 + 3) % 128) as i64;
    }
    
    // Count occurrences of specific values
    let mut count1 = 0;
    let mut count2 = 0;
    let mut count3 = 0;
    let mut count4 = 0;
    
    for i in 0..n {
        if s[i] == 65 {
            count1 += 1;
        } else if s[i] == 66 {
            count2 += 1;
        } else if s[i] == 67 {
            count3 += 1;
        } else if s[i] == 68 {
            count4 += 1;
        }
    }
    
    // Calculate checksum
    let checksum = n as i64 + count1 + count2 + count3 + count4;
    println!("string operations checksum: {}", checksum);
}
