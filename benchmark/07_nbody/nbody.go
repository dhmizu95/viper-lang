// Benchmark 07: N-Body Simulation
// Category: Simulation
// Tests: Physics calculations, nested loops, floating-point math

package main

import (
	"fmt"
	"math"
	"time"
)

const (
	N_BODIES   = 500
	TIME_STEPS = 1000
	G          = 6.67430e-11
	DT         = 0.01
)

type Body struct {
	X, Y, Z   float64
	Vx, Vy, Vz float64
	Mass      float64
}

func main() {
	start := time.Now()

	// Initialize bodies in a simple configuration
	bodies := make([]Body, N_BODIES)
	for i := 0; i < N_BODIES; i++ {
		bodies[i] = Body{
			X:   float64(i%10) * 1e6,
			Y:   float64((i/10)%10) * 1e6,
			Z:   float64(i/100) * 1e6,
			Vx:  float64(i%7) * 100.0,
			Vy:  float64(i%5) * 100.0,
			Vz:  float64(i%3) * 100.0,
			Mass: 1e10 + float64(i%100)*1e8,
		}
	}

	// Simulation loop
	for step := 0; step < TIME_STEPS; step++ {
		// Calculate forces and update velocities
		for i := 0; i < N_BODIES; i++ {
			fx, fy, fz := 0.0, 0.0, 0.0

			for j := 0; j < N_BODIES; j++ {
				if i == j {
					continue
				}

				dx := bodies[j].X - bodies[i].X
				dy := bodies[j].Y - bodies[i].Y
				dz := bodies[j].Z - bodies[i].Z
				distSq := dx*dx + dy*dy + dz*dz + 1e10 // Softening
				dist := math.Sqrt(distSq)
				force := G * bodies[i].Mass * bodies[j].Mass / distSq

				fx += force * dx / dist
				fy += force * dy / dist
				fz += force * dz / dist
			}

			bodies[i].Vx += fx / bodies[i].Mass * DT
			bodies[i].Vy += fy / bodies[i].Mass * DT
			bodies[i].Vz += fz / bodies[i].Mass * DT
		}

		// Update positions
		for i := 0; i < N_BODIES; i++ {
			bodies[i].X += bodies[i].Vx * DT
			bodies[i].Y += bodies[i].Vy * DT
			bodies[i].Z += bodies[i].Vz * DT
		}
	}

	elapsed := time.Since(start)

	// Calculate total kinetic energy for verification
	totalEnergy := 0.0
	for i := 0; i < N_BODIES; i++ {
		vSq := bodies[i].Vx*bodies[i].Vx + bodies[i].Vy*bodies[i].Vy + bodies[i].Vz*bodies[i].Vz
		totalEnergy += 0.5 * bodies[i].Mass * vSq
	}

	fmt.Printf("N-Bodies: %d, Time steps: %d\n", N_BODIES, TIME_STEPS)
	fmt.Printf("Total kinetic energy: %.6e\n", totalEnergy)
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
