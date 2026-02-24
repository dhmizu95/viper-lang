// Benchmark 06: Simple Ray Tracer
// Category: Simulation / Floating Point
// Tests: 3D math, vector operations, recursion

#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>

#define WIDTH 400
#define HEIGHT 400
#define MAX_DEPTH 4

typedef struct {
    double x, y, z;
} Vec3;

typedef struct {
    Vec3 center;
    double radius;
} Sphere;

Vec3 vec3(double x, double y, double z) {
    Vec3 v = {x, y, z};
    return v;
}

Vec3 vec3_add(Vec3 a, Vec3 b) {
    return vec3(a.x + b.x, a.y + b.y, a.z + b.z);
}

Vec3 vec3_sub(Vec3 a, Vec3 b) {
    return vec3(a.x - b.x, a.y - b.y, a.z - b.z);
}

Vec3 vec3_mul(Vec3 a, double t) {
    return vec3(a.x * t, a.y * t, a.z * t);
}

double vec3_dot(Vec3 a, Vec3 b) {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}

double vec3_length(Vec3 a) {
    return sqrt(vec3_dot(a, a));
}

Vec3 vec3_normalize(Vec3 a) {
    double len = vec3_length(a);
    return vec3_mul(a, 1.0 / len);
}

// Ray-sphere intersection
double hit_sphere(Sphere s, Vec3 origin, Vec3 dir) {
    Vec3 oc = vec3_sub(origin, s.center);
    double a = vec3_dot(dir, dir);
    double b = 2.0 * vec3_dot(oc, dir);
    double c = vec3_dot(oc, oc) - s.radius * s.radius;
    double discriminant = b * b - 4 * a * c;
    
    if (discriminant < 0) return -1.0;
    return (-b - sqrt(discriminant)) / (2.0 * a);
}

// Trace ray and return color intensity
double trace(Vec3 origin, Vec3 dir, Sphere* spheres, int sphere_count, int depth) {
    if (depth <= 0) return 0.0;
    
    double closest = 1e10;
    int hit_idx = -1;
    
    for (int i = 0; i < sphere_count; i++) {
        double t = hit_sphere(spheres[i], origin, dir);
        if (t > 0.001 && t < closest) {
            closest = t;
            hit_idx = i;
        }
    }
    
    if (hit_idx < 0) return 0.0;
    
    Vec3 hit_point = vec3_add(origin, vec3_mul(dir, closest));
    Vec3 normal = vec3_normalize(vec3_sub(hit_point, spheres[hit_idx].center));
    Vec3 reflected = vec3_sub(dir, vec3_mul(normal, 2.0 * vec3_dot(dir, normal)));
    
    return 0.5 + 0.5 * trace(hit_point, reflected, spheres, sphere_count, depth - 1);
}

int main() {
    clock_t start = clock();
    
    // Scene setup
    Sphere spheres[3];
    spheres[0] = (Sphere){vec3(0, 0, -5), 1.0};
    spheres[1] = (Sphere){vec3(2, 0.5, -4), 0.5};
    spheres[2] = (Sphere){vec3(-2, 0, -6), 0.7};
    
    double total_intensity = 0.0;
    
    // Render
    for (int y = 0; y < HEIGHT; y++) {
        for (int x = 0; x < WIDTH; x++) {
            double u = (2.0 * x - WIDTH) / HEIGHT;
            double v = (HEIGHT - 2.0 * y) / HEIGHT;
            Vec3 dir = vec3_normalize(vec3(u, v, -1.0));
            Vec3 origin = vec3(0, 0, 0);
            
            double intensity = trace(origin, dir, spheres, 3, MAX_DEPTH);
            total_intensity += intensity;
        }
    }
    
    clock_t end = clock();
    double time_spent = (double)(end - start) / CLOCKS_PER_SEC;
    
    printf("Image size: %dx%d\n", WIDTH, HEIGHT);
    printf("Average intensity: %.6f\n", total_intensity / (WIDTH * HEIGHT));
    printf("Time: %.4f seconds\n", time_spent);
    
    return 0;
}
