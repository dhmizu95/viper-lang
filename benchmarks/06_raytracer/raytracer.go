// Benchmark 06: Simple Ray Tracer
// Category: Simulation / Floating Point
// Tests: 3D math, vector operations, recursion

package main

import (
	"fmt"
	"math"
	"time"
)

const (
	WIDTH     = 400
	HEIGHT    = 400
	MAX_DEPTH = 4
)

type Vec3 struct {
	X, Y, Z float64
}

type Sphere struct {
	Center Vec3
	Radius float64
}

func vec3Add(a, b Vec3) Vec3 {
	return Vec3{a.X + b.X, a.Y + b.Y, a.Z + b.Z}
}

func vec3Sub(a, b Vec3) Vec3 {
	return Vec3{a.X - b.X, a.Y - b.Y, a.Z - b.Z}
}

func vec3Mul(a Vec3, t float64) Vec3 {
	return Vec3{a.X * t, a.Y * t, a.Z * t}
}

func vec3Dot(a, b Vec3) float64 {
	return a.X*b.X + a.Y*b.Y + a.Z*b.Z
}

func vec3Length(a Vec3) float64 {
	return math.Sqrt(vec3Dot(a, a))
}

func vec3Normalize(a Vec3) Vec3 {
	len := vec3Length(a)
	return vec3Mul(a, 1.0/len)
}

// Ray-sphere intersection
func hitSphere(s Sphere, origin, dir Vec3) float64 {
	oc := vec3Sub(origin, s.Center)
	a := vec3Dot(dir, dir)
	b := 2.0 * vec3Dot(oc, dir)
	c := vec3Dot(oc, oc) - s.Radius*s.Radius
	discriminant := b*b - 4*a*c

	if discriminant < 0 {
		return -1.0
	}
	return (-b - math.Sqrt(discriminant)) / (2.0 * a)
}

// Trace ray and return color intensity
func trace(origin, dir Vec3, spheres []Sphere, depth int) float64 {
	if depth <= 0 {
		return 0.0
	}

	closest := 1e10
	hitIdx := -1

	for i, s := range spheres {
		t := hitSphere(s, origin, dir)
		if t > 0.001 && t < closest {
			closest = t
			hitIdx = i
		}
	}

	if hitIdx < 0 {
		return 0.0
	}

	hitPoint := vec3Add(origin, vec3Mul(dir, closest))
	normal := vec3Normalize(vec3Sub(hitPoint, spheres[hitIdx].Center))
	reflected := vec3Sub(dir, vec3Mul(normal, 2.0*vec3Dot(dir, normal)))

	return 0.5 + 0.5*trace(hitPoint, reflected, spheres, depth-1)
}

func main() {
	start := time.Now()

	// Scene setup
	spheres := []Sphere{
		{Center: Vec3{0, 0, -5}, Radius: 1.0},
		{Center: Vec3{2, 0.5, -4}, Radius: 0.5},
		{Center: Vec3{-2, 0, -6}, Radius: 0.7},
	}

	totalIntensity := 0.0

	// Render
	for y := 0; y < HEIGHT; y++ {
		for x := 0; x < WIDTH; x++ {
			u := (2.0*float64(x) - float64(WIDTH)) / float64(HEIGHT)
			v := (float64(HEIGHT) - 2.0*float64(y)) / float64(HEIGHT)
			dir := vec3Normalize(Vec3{u, v, -1.0})
			origin := Vec3{0, 0, 0}

			intensity := trace(origin, dir, spheres, MAX_DEPTH)
			totalIntensity += intensity
		}
	}

	elapsed := time.Since(start)

	fmt.Printf("Image size: %dx%d\n", WIDTH, HEIGHT)
	fmt.Printf("Average intensity: %.6f\n", totalIntensity/float64(WIDTH*HEIGHT))
	fmt.Printf("Time: %.4f seconds\n", elapsed.Seconds())
}
