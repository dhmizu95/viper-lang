// Benchmark 12: Reverse Complement
// Category: String Processing / Bioinformatics
// Tests: String manipulation, I/O, character mapping

use std::time::Instant;

const SEQ_LEN: usize = 5_000_000; // 5 million bases

fn complement(c: u8) -> u8 {
    match c {
        b'A' | b'a' => b'T',
        b'T' | b't' => b'A',
        b'G' | b'g' => b'C',
        b'C' | b'c' => b'G',
        _ => b'N',
    }
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

    // Generate reverse complement
    let mut reverse_complement = vec![0u8; SEQ_LEN];
    for i in 0..SEQ_LEN {
        reverse_complement[i] = complement(sequence[SEQ_LEN - 1 - i]);
    }

    // Verify by checking a few positions
    let verified = (0..1000).all(|i| {
        let expected = complement(sequence[SEQ_LEN - 1 - i]);
        reverse_complement[i] == expected
    });

    let elapsed = start.elapsed();

    println!("Sequence length: {}", SEQ_LEN);
    println!("Verification: {}", if verified { "Passed" } else { "Failed" });
    println!(
        "First 50 bases of reverse complement: {}",
        std::str::from_utf8(&reverse_complement[..50]).unwrap()
    );
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
