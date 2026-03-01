/*
 * FFT (Fast Fourier Transform) - Signal Processing Benchmark
 * Tests recursion, floating-point math, and array operations
 * Uses 1M samples
 */
use std::time::Instant;

const N: usize = 1_048_576; // 2^20 = 1M samples

#[derive(Clone, Copy)]
struct Complex {
    real: f64,
    imag: f64,
}

fn precompute_bit_reverse() -> Vec<usize> {
    let bits = 20; // log2(N)
    let mut bit_rev = vec![0; N];
    for i in 0..N {
        let mut rev = 0;
        for j in 0..bits {
            rev = (rev << 1) | ((i >> j) & 1);
        }
        bit_rev[i] = rev;
    }
    bit_rev
}

fn fft(x: &mut [Complex], bit_rev: &[usize]) {
    let n = x.len();
    
    // Bit-reversal permutation
    for i in 0..n {
        if i < bit_rev[i] {
            x.swap(i, bit_rev[i]);
        }
    }
    
    // Cooley-Tukey FFT
    let mut len = 2;
    while len <= n {
        let angle = -2.0 * std::f64::consts::PI / len as f64;
        let wlen = Complex {
            real: angle.cos(),
            imag: angle.sin(),
        };
        
        for i in (0..n).step_by(len) {
            let mut w = Complex { real: 1.0, imag: 0.0 };
            for j in 0..len / 2 {
                let u = x[i + j];
                let v = Complex {
                    real: w.real * x[i + j + len / 2].real - w.imag * x[i + j + len / 2].imag,
                    imag: w.real * x[i + j + len / 2].imag + w.imag * x[i + j + len / 2].real,
                };
                x[i + j] = Complex {
                    real: u.real + v.real,
                    imag: u.imag + v.imag,
                };
                x[i + j + len / 2] = Complex {
                    real: u.real - v.real,
                    imag: u.imag - v.imag,
                };
                
                w = Complex {
                    real: w.real * wlen.real - w.imag * wlen.imag,
                    imag: w.real * wlen.imag + w.imag * wlen.real,
                };
            }
        }
        len *= 2;
    }
}

fn main() {
    println!("Computing FFT with {} samples...", N);
    
    let bit_rev = precompute_bit_reverse();
    
    // Initialize with test signal
    let mut data: Vec<Complex> = Vec::with_capacity(N);
    for i in 0..N {
        let t = i as f64 / N as f64;
        data.push(Complex {
            real: (2.0 * std::f64::consts::PI * 10.0 * t).sin() 
                + 0.5 * (2.0 * std::f64::consts::PI * 25.0 * t).sin(),
            imag: 0.0,
        });
    }
    
    let start = Instant::now();
    
    fft(&mut data, &bit_rev);
    
    let elapsed = start.elapsed();
    
    // Compute magnitude spectrum checksum
    let checksum: f64 = data.iter()
        .map(|c| (c.real * c.real + c.imag * c.imag).sqrt())
        .sum();
    
    println!("Magnitude checksum: {:.6}", checksum);
    println!("Time: {:?}", elapsed);
    println!("Samples/sec: {:.0}", N as f64 / elapsed.as_secs_f64());
}
