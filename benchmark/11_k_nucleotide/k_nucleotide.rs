// Benchmark 11: K-Nucleotide
// Category: String Processing / Bioinformatics
// Tests: String manipulation, hash tables, pattern matching

use std::collections::HashMap;
use std::time::Instant;

const SEQ_LEN: usize = 100_000;
const K: usize = 4;

fn main() {
    let start = Instant::now();

    // Generate random DNA sequence
    let bases = [b'A', b'C', b'G', b'T'];
    let mut sequence = Vec::with_capacity(SEQ_LEN);

    let mut rng = rand_xoshiro::Xoroshiro64Star::seed_from_u64(42);
    for _ in 0..SEQ_LEN {
        sequence.push(bases[(rng.next_u64() % 4) as usize]);
    }

    // Count all k-mers using HashMap
    let mut kmer_counts: HashMap<String, usize> = HashMap::new();

    for i in 0..=SEQ_LEN - K {
        let kmer = std::str::from_utf8(&sequence[i..i + K]).unwrap();
        *kmer_counts.entry(kmer.to_string()).or_insert(0) += 1;
    }

    // Find most and least frequent
    let mut max_count = 0;
    let mut min_count = SEQ_LEN + 1;
    let mut max_kmer = String::new();
    let mut min_kmer = String::new();
    let total_unique = kmer_counts.len();

    for (kmer, &count) in &kmer_counts {
        if count > max_count {
            max_count = count;
            max_kmer = kmer.clone();
        }
        if count < min_count {
            min_count = count;
            min_kmer = kmer.clone();
        }
    }

    let elapsed = start.elapsed();

    println!("Sequence length: {}, K: {}", SEQ_LEN, K);
    println!("Unique {}-mers: {}", K, total_unique);
    println!("Most frequent: {} ({} times)", max_kmer, max_count);
    println!("Least frequent: {} ({} times)", min_kmer, min_count);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}

// Simple PRNG for reproducibility without external dependency
mod rand_xoshiro {
    pub struct Xoroshiro64Star {
        state: [u64; 2],
    }

    impl Xoroshiro64Star {
        pub fn seed_from_u64(seed: u64) -> Self {
            Self {
                state: [seed, seed.wrapping_add(0x1234567890abcdef)],
            }
        }

        pub fn next_u64(&mut self) -> u64 {
            let mut s0 = self.state[0];
            let mut s1 = self.state[1];
            let result = s0.wrapping_add(s1).rotate_left(18).wrapping_mul(5);

            s1 ^= s0;
            self.state[0] = s0.rotate_left(24) ^ s1 ^ (s1 << 16);
            self.state[1] = s1.rotate_left(37);

            result
        }
    }
}
