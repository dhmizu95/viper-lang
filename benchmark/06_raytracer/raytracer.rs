// Benchmark 06: Simple Ray Tracer
// Category: Simulation / Floating Point
// Tests: 3D math, vector operations, recursion

use std::time::Instant;

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const MAX_DEPTH: usize = 4;

#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

struct Sphere {
    center: Vec3,
    radius: f64,
}

impl Vec3 {
    fn new(x: f64, y: f64, z: f64) -> Self {
        Vec3 { x, y, z }
    }

    fn add(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn mul(self, t: f64) -> Vec3 {
        Vec3::new(self.x * t, self.y * t, self.z * t)
    }

    fn dot(self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn length(self) -> f64 {
        (self.dot(self)).sqrt()
    }

    fn normalize(self) -> Vec3 {
        self.mul(1.0 / self.length())
    }
}

// Ray-sphere intersection
fn hit_sphere(s: &Sphere, origin: Vec3, dir: Vec3) -> f64 {
    let oc = origin.sub(s.center);
    let a = dir.dot(dir);
    let b = 2.0 * oc.dot(dir);
    let c = oc.dot(oc) - s.radius * s.radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return -1.0;
    }
    (-b - discriminant.sqrt()) / (2.0 * a)
}

// Trace ray and return color intensity
fn trace(origin: Vec3, dir: Vec3, spheres: &[Sphere], depth: usize) -> f64 {
    if depth == 0 {
        return 0.0;
    }

    let mut closest = 1e10;
    let mut hit_idx = None;

    for (i, s) in spheres.iter().enumerate() {
        let t = hit_sphere(s, origin, dir);
        if t > 0.001 && t < closest {
            closest = t;
            hit_idx = Some(i);
        }
    }

    if let Some(idx) = hit_idx {
        let hit_point = origin.add(dir.mul(closest));
        let normal = hit_point.sub(spheres[idx].center).normalize();
        let reflected = dir.sub(normal.mul(2.0 * dir.dot(normal)));

        0.5 + 0.5 * trace(hit_point, reflected, spheres, depth - 1)
    } else {
        0.0
    }
}

fn main() {
    let start = Instant::now();

    // Scene setup
    let spheres = [
        Sphere {
            center: Vec3::new(0.0, 0.0, -5.0),
            radius: 1.0,
        },
        Sphere {
            center: Vec3::new(2.0, 0.5, -4.0),
            radius: 0.5,
        },
        Sphere {
            center: Vec3::new(-2.0, 0.0, -6.0),
            radius: 0.7,
        },
    ];

    let mut total_intensity = 0.0;

    // Render
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let u = (2.0 * x as f64 - WIDTH as f64) / HEIGHT as f64;
            let v = (HEIGHT as f64 - 2.0 * y as f64) / HEIGHT as f64;
            let dir = Vec3::new(u, v, -1.0).normalize();
            let origin = Vec3::new(0.0, 0.0, 0.0);

            let intensity = trace(origin, dir, &spheres, MAX_DEPTH);
            total_intensity += intensity;
        }
    }

    let elapsed = start.elapsed();

    println!("Image size: {}x{}", WIDTH, HEIGHT);
    println!("Average intensity: {:.6}", total_intensity / (WIDTH * HEIGHT) as f64);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
