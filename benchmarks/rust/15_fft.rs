// Fast Fourier Transform (FFT) Benchmark - Rust Implementation
// Cooley-Tukey radix-2 FFT algorithm

const N: usize = 256;
const PI: f64 = 3.14159265358979323846;

#[derive(Clone, Copy)]
struct Complex {
    real: f64,
    imag: f64,
}

impl Complex {
    fn new(real: f64, imag: f64) -> Self {
        Complex { real, imag }
    }
    
    fn add(self, other: Complex) -> Complex {
        Complex {
            real: self.real + other.real,
            imag: self.imag + other.imag,
        }
    }
    
    fn sub(self, other: Complex) -> Complex {
        Complex {
            real: self.real - other.real,
            imag: self.imag - other.imag,
        }
    }
    
    fn mul(self, other: Complex) -> Complex {
        Complex {
            real: self.real * other.real - self.imag * other.imag,
            imag: self.real * other.imag + self.imag * other.real,
        }
    }
}

fn bit_reverse(n: usize, bits: usize) -> usize {
    let mut reversed = 0;
    let mut num = n;
    for _ in 0..bits {
        reversed = (reversed << 1) | (num & 1);
        num >>= 1;
    }
    reversed
}

fn fft(x: &mut [Complex], n: usize, inverse: bool) {
    let bits = (n as f64).log2() as usize;
    
    // Bit-reversal permutation
    for i in 0..n {
        let rev = bit_reverse(i, bits);
        if i < rev {
            x.swap(i, rev);
        }
    }
    
    // Cooley-Tukey iterative FFT
    let mut len = 2;
    while len <= n {
        let angle = 2.0 * PI / len as f64 * if inverse { -1.0 } else { 1.0 };
        let wlen = Complex::new(angle.cos(), angle.sin());
        
        let mut i = 0;
        while i < n {
            let mut w = Complex::new(1.0, 0.0);
            for j in 0..len / 2 {
                let u = x[i + j];
                let v = x[i + j + len/2].mul(w);
                x[i + j] = u.add(v);
                x[i + j + len/2] = u.sub(v);
                w = w.mul(wlen);
            }
            i += len;
        }
        len <<= 1;
    }
    
    // Scale for inverse FFT
    if inverse {
        let n_inv = 1.0 / n as f64;
        for i in 0..n {
            x[i].real *= n_inv;
            x[i].imag *= n_inv;
        }
    }
}

fn generate_signal(signal: &mut [Complex], n: usize) {
    let freq1 = 2.0;
    let freq2 = 8.0;
    let sample_rate = 64.0;
    
    for i in 0..n {
        let t = i as f64 / sample_rate;
        let real = (2.0 * PI * freq1 * t).sin() + 0.5 * (2.0 * PI * freq2 * t).sin();
        signal[i] = Complex::new(real, 0.0);
    }
}

fn total_magnitude(spectrum: &[Complex], n: usize) -> f64 {
    let mut total = 0.0;
    for i in 0..n {
        total += (spectrum[i].real.powi(2) + spectrum[i].imag.powi(2)).sqrt();
    }
    total
}

fn main() {
    let mut signal = vec![Complex::new(0.0, 0.0); N];
    
    generate_signal(&mut signal, N);
    fft(&mut signal, N, false);
    
    let magnitude = total_magnitude(&signal, N);
    println!("{:.6}", magnitude);
    
    fft(&mut signal, N, true);
    println!("{:.6}", signal[0].real);
}
