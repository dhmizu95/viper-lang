// Benchmark 07: N-Body Simulation
// Category: Simulation
// Tests: Physics calculations, nested loops, floating-point math

use std::time::Instant;

const N_BODIES: usize = 500;
const TIME_STEPS: usize = 1000;
const G: f64 = 6.67430e-11;
const DT: f64 = 0.01;

struct Body {
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    mass: f64,
}

fn main() {
    let start = Instant::now();

    // Initialize bodies in a simple configuration
    let mut bodies: Vec<Body> = (0..N_BODIES)
        .map(|i| Body {
            x: (i % 10) as f64 * 1e6,
            y: ((i / 10) % 10) as f64 * 1e6,
            z: (i / 100) as f64 * 1e6,
            vx: (i % 7) as f64 * 100.0,
            vy: (i % 5) as f64 * 100.0,
            vz: (i % 3) as f64 * 100.0,
            mass: 1e10 + (i % 100) as f64 * 1e8,
        })
        .collect();

    // Simulation loop
    for _ in 0..TIME_STEPS {
        // Calculate forces and update velocities
        for i in 0..N_BODIES {
            let mut fx = 0.0;
            let mut fy = 0.0;
            let mut fz = 0.0;

            for j in 0..N_BODIES {
                if i == j {
                    continue;
                }

                let dx = bodies[j].x - bodies[i].x;
                let dy = bodies[j].y - bodies[i].y;
                let dz = bodies[j].z - bodies[i].z;
                let dist_sq = dx * dx + dy * dy + dz * dz + 1e10; // Softening
                let dist = dist_sq.sqrt();
                let force = G * bodies[i].mass * bodies[j].mass / dist_sq;

                fx += force * dx / dist;
                fy += force * dy / dist;
                fz += force * dz / dist;
            }

            bodies[i].vx += fx / bodies[i].mass * DT;
            bodies[i].vy += fy / bodies[i].mass * DT;
            bodies[i].vz += fz / bodies[i].mass * DT;
        }

        // Update positions
        for i in 0..N_BODIES {
            bodies[i].x += bodies[i].vx * DT;
            bodies[i].y += bodies[i].vy * DT;
            bodies[i].z += bodies[i].vz * DT;
        }
    }

    let elapsed = start.elapsed();

    // Calculate total kinetic energy for verification
    let total_energy: f64 = bodies
        .iter()
        .map(|b| {
            let v_sq = b.vx * b.vx + b.vy * b.vy + b.vz * b.vz;
            0.5 * b.mass * v_sq
        })
        .sum();

    println!("N-Bodies: {}, Time steps: {}", N_BODIES, TIME_STEPS);
    println!("Total kinetic energy: {:.6e}", total_energy);
    println!("Time: {:.4} seconds", elapsed.as_secs_f64());
}
