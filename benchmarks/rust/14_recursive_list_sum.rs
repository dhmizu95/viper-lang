// Recursive List Sum Benchmark - Rust Implementation
// Recursive sum of a vector

fn sum_vec(vec: &Vec<i64>, idx: usize) -> i64 {
    if idx >= vec.len() {
        return 0;
    }
    vec[idx] + sum_vec(vec, idx + 1)
}

fn main() {
    let vec: Vec<i64> = (1..=1000).collect();
    
    let result = sum_vec(&vec, 0);
    println!("{}", result);
}