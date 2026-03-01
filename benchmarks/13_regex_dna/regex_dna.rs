// Benchmark 13: Regex DNA Matching
// Category: String Processing / Pattern Matching
// Tests: Pattern matching, string scanning

use std::time::Instant;

const SEQ_LEN: usize = 1_000_000; // 1 million bases

// Pattern: Find all occurrences of "ATG" (start codon) followed by any 3 bases, then "TAA", "TAG", or "TGA" (stop codons)
fn match_pattern(seq: &[u8], pos: usize) -> bool {
    if pos + 9 > seq.len() {
        return false;
    }

    // Check for start codon ATG
    if seq[pos] != b'A' || seq[pos + 1] != b'T' || seq[pos + 2] != b'G' {
        return false;
    }

    // Check for stop codon at position + 6 to + 8 (after 3 any bases)
    let stop = &seq[pos + 6..pos + 9];
    stop == b"TAA" || stop == b"TAG" || stop == b"TGA"
}

fn main() {
    let start = Instant::now();

    // Generate random DNA sequence
    let bases = [b'A', b'T', b'G', b'C'];
    let mut sequence = Vec::with_capacity(SEQ_LEN);

    let mut rng = Xoroshiro64Star::seed_from_u64(42);
    for _ in 0..SEQ_LEN {
        sequence.push(bases[(rng.next_u64() % 4) as usize]);
    }

    // Find pattern matches
    let match_count = (0..SEQ_LEN - 9)
        .filter(|&i| match_pattern(&sequence, i))
        .count();

    let elapsed = start.elapsed();

    println!("Sequence length: {}", SEQ_LEN);
    println!("Pattern matches found: {}", match_count);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}

// Simple PRNG for reproducibility
struct Xoroshiro64Star {
    state: [u64; 2],
}

impl Xoroshiro64Star {
    fn seed_from_u64(seed: u64) -> Self {
        Self {
            state: [seed, seed.wrapping_add(0x1234567890abcdef)],
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut s0 = self.state[0];
        let mut s1 = self.state[1];
        let result = s0.wrapping_add(s1).rotate_left(18).wrapping_mul(5);

        s1 ^= s0;
        self.state[0] = s0.rotate_left(24) ^ s1 ^ (s1 << 16);
        self.state[1] = s1.rotate_left(37);

        result
    }
}
