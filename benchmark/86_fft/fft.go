/*
 * FFT (Fast Fourier Transform) - Signal Processing Benchmark
 * Tests recursion, floating-point math, and array operations
 * Uses 1M samples
 */
package main

import (
	"fmt"
	"math"
	"time"
)

const N = 1048576 // 2^20 = 1M samples

type Complex struct {
	real, imag float64
}

func precomputeBitReverse() []int {
	bits := 20 // log2(N)
	bitRev := make([]int, N)
	for i := 0; i < N; i++ {
		rev := 0
		for j := 0; j < bits; j++ {
			rev = (rev << 1) | ((i >> j) & 1)
		}
		bitRev[i] = rev
	}
	return bitRev
}

func fft(x []Complex, bitRev []int) {
	n := len(x)
	
	// Bit-reversal permutation
	for i := 0; i < n; i++ {
		if i < bitRev[i] {
			x[i], x[bitRev[i]] = x[bitRev[i]], x[i]
		}
	}
	
	// Cooley-Tukey FFT
	for len := 2; len <= n; len *= 2 {
		angle := -2.0 * math.Pi / float64(len)
		wlen := Complex{math.Cos(angle), math.Sin(angle)}
		
		for i := 0; i < n; i += len {
			w := Complex{1.0, 0.0}
			for j := 0; j < len/2; j++ {
				u := x[i+j]
				v := Complex{
					w.real*x[i+j+len/2].real - w.imag*x[i+j+len/2].imag,
					w.real*x[i+j+len/2].imag + w.imag*x[i+j+len/2].real,
				}
				x[i+j] = Complex{u.real + v.real, u.imag + v.imag}
				x[i+j+len/2] = Complex{u.real - v.real, u.imag - v.imag}
				
				w = Complex{
					w.real*wlen.real - w.imag*wlen.imag,
					w.real*wlen.imag + w.imag*wlen.real,
				}
			}
		}
	}
}

func main() {
	fmt.Printf("Computing FFT with %d samples...\n", N)
	
	bitRev := precomputeBitReverse()
	
	// Initialize with test signal
	data := make([]Complex, N)
	for i := 0; i < N; i++ {
		t := float64(i) / float64(N)
		data[i] = Complex{
			math.Sin(2.0 * math.Pi * 10.0 * t) + 0.5*math.Sin(2.0*math.Pi*25.0*t),
			0.0,
		}
	}
	
	start := time.Now()
	
	fft(data, bitRev)
	
	elapsed := time.Since(start)
	
	// Compute magnitude spectrum checksum
	var checksum float64
	for i := 0; i < N; i++ {
		checksum += math.Sqrt(data[i].real*data[i].real + data[i].imag*data[i].imag)
	}
	
	fmt.Printf("Magnitude checksum: %.6f\n", checksum)
	fmt.Printf("Time: %v\n", elapsed)
	fmt.Printf("Samples/sec: %.0f\n", float64(N)/elapsed.Seconds())
}
