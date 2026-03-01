/*
 * Monte Carlo Pi - Floating Point Benchmark
 * Tests floating-point performance and random number generation
 * Uses 1 billion samples
 */
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::time::Instant;

fn main() {
    const SAMPLES: i64 = 1_000_000_000; // 1 billion
    let mut inside: i64 = 0;
    
    println!("Computing Pi using Monte Carlo with {} samples...", SAMPLES);
    
    let start = Instant::now();
    
    let mut rng = StdRng::seed_from_u64(42);
    
    for _ in 0..SAMPLES {
        let x: f64 = rng.random();
        let y: f64 = rng.random();
        if x * x + y * y <= 1.0 {
            inside += 1;
        }
    }
    
    let elapsed = start.elapsed();
    
    let pi = 4.0 * inside as f64 / SAMPLES as f64;
    
    println!("Estimated Pi: {:.15}", pi);
    println!("Actual Pi:    3.141592653589793");
    println!("Error:        {:.15}", (pi - 3.141592653589793).abs());
    println!("Time:         {:?}", elapsed);
    println!("Samples/sec:  {:.0}", SAMPLES as f64 / elapsed.as_secs_f64());
}
