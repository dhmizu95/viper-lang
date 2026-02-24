// Benchmark 07: N-Body Simulation
// Category: Simulation
// Tests: Physics calculations, nested loops, floating-point math

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>

#define N_BODIES 500
#define TIME_STEPS 1000
#define G 6.67430e-11
#define DT 0.01

typedef struct {
    double x, y, z;
    double vx, vy, vz;
    double mass;
} Body;

int main() {
    clock_t start = clock();
    
    // Initialize bodies in a simple configuration
    Body* bodies = (Body*)malloc(N_BODIES * sizeof(Body));
    if (!bodies) {
        fprintf(stderr, "Memory allocation failed\n");
        return 1;
    }
    
    for (int i = 0; i < N_BODIES; i++) {
        bodies[i].x = (i % 10) * 1e6;
        bodies[i].y = ((i / 10) % 10) * 1e6;
        bodies[i].z = (i / 100) * 1e6;
        bodies[i].vx = (i % 7) * 100.0;
        bodies[i].vy = (i % 5) * 100.0;
        bodies[i].vz = (i % 3) * 100.0;
        bodies[i].mass = 1e10 + (i % 100) * 1e8;
    }
    
    // Simulation loop
    for (int step = 0; step < TIME_STEPS; step++) {
        // Calculate forces and update velocities
        for (int i = 0; i < N_BODIES; i++) {
            double fx = 0, fy = 0, fz = 0;
            
            for (int j = 0; j < N_BODIES; j++) {
                if (i == j) continue;
                
                double dx = bodies[j].x - bodies[i].x;
                double dy = bodies[j].y - bodies[i].y;
                double dz = bodies[j].z - bodies[i].z;
                double dist_sq = dx * dx + dy * dy + dz * dz + 1e10; // Softening
                double dist = sqrt(dist_sq);
                double force = G * bodies[i].mass * bodies[j].mass / dist_sq;
                
                fx += force * dx / dist;
                fy += force * dy / dist;
                fz += force * dz / dist;
            }
            
            bodies[i].vx += fx / bodies[i].mass * DT;
            bodies[i].vy += fy / bodies[i].mass * DT;
            bodies[i].vz += fz / bodies[i].mass * DT;
        }
        
        // Update positions
        for (int i = 0; i < N_BODIES; i++) {
            bodies[i].x += bodies[i].vx * DT;
            bodies[i].y += bodies[i].vy * DT;
            bodies[i].z += bodies[i].vz * DT;
        }
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    // Calculate total kinetic energy for verification
    double total_energy = 0;
    for (int i = 0; i < N_BODIES; i++) {
        double v_sq = bodies[i].vx * bodies[i].vx + 
                      bodies[i].vy * bodies[i].vy + 
                      bodies[i].vz * bodies[i].vz;
        total_energy += 0.5 * bodies[i].mass * v_sq;
    }
    
    printf("N-Bodies: %d, Time steps: %d\n", N_BODIES, TIME_STEPS);
    printf("Total kinetic energy: %.6e\n", total_energy);
    printf("Time: %.4f seconds\n", time_spent);
    
    free(bodies);
    return 0;
}
