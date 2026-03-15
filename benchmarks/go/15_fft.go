// Fast Fourier Transform (FFT) Benchmark - Go Implementation
// Cooley-Tukey radix-2 FFT algorithm

package main

import (
	"fmt"
	"math"
)

const (
	N  = 256
	PI = math.Pi
)

type Complex struct {
	Real, Imag float64
}

func complexMake(real, imag float64) Complex {
	return Complex{real, imag}
}

func (a Complex) add(b Complex) Complex {
	return complexMake(a.Real+b.Real, a.Imag+b.Imag)
}

func (a Complex) sub(b Complex) Complex {
	return complexMake(a.Real-b.Real, a.Imag-b.Imag)
}

func (a Complex) mul(b Complex) Complex {
	return complexMake(
		a.Real*b.Real-a.Imag*b.Imag,
		a.Real*b.Imag+a.Imag*b.Real,
	)
}

func bitReverse(n, bits int) int {
	reversed := 0
	for i := 0; i < bits; i++ {
		reversed = (reversed << 1) | (n & 1)
		n >>= 1
	}
	return reversed
}

func fft(x []Complex, inverse bool) {
	n := len(x)
	bits := 0
	for temp := n; temp > 1; temp >>= 1 {
		bits++
	}

	// Bit-reversal permutation
	for i := 0; i < n; i++ {
		rev := bitReverse(i, bits)
		if i < rev {
			x[i], x[rev] = x[rev], x[i]
		}
	}

	// Cooley-Tukey iterative FFT
	for length := 2; length <= n; length <<= 1 {
		angle := 2 * PI / float64(length)
		if inverse {
			angle = -angle
		}
		wlen := complexMake(math.Cos(angle), math.Sin(angle))

		for i := 0; i < n; i += length {
			w := complexMake(1.0, 0.0)
			for j := 0; j < length/2; j++ {
				u := x[i+j]
				v := x[i+j+length/2].mul(w)
				x[i+j] = u.add(v)
				x[i+j+length/2] = u.sub(v)
				w = w.mul(wlen)
			}
		}
	}

	// Scale for inverse FFT
	if inverse {
		for i := 0; i < n; i++ {
			x[i].Real /= float64(n)
			x[i].Imag /= float64(n)
		}
	}
}

func generateSignal(n int) []Complex {
	freq1 := 2.0
	freq2 := 8.0
	sampleRate := 64.0

	signal := make([]Complex, n)
	for i := 0; i < n; i++ {
		t := float64(i) / sampleRate
		real := math.Sin(2*PI*freq1*t) + 0.5*math.Sin(2*PI*freq2*t)
		signal[i] = complexMake(real, 0.0)
	}
	return signal
}

func totalMagnitude(spectrum []Complex) float64 {
	total := 0.0
	for _, c := range spectrum {
		total += math.Sqrt(c.Real*c.Real + c.Imag*c.Imag)
	}
	return total
}

func main() {
	// Generate test signal
	signal := generateSignal(N)

	// Perform FFT
	fft(signal, false)

	// Calculate and print result
	magnitude := totalMagnitude(signal)
	fmt.Printf("%.6f\n", magnitude)

	// Perform inverse FFT to verify
	fft(signal, true)

	// Print first sample of reconstructed signal (verification)
	fmt.Printf("%.6f\n", signal[0].Real)
}
