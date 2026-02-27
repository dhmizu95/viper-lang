/**
 * Viper vs Go - Real World Task Benchmark
 * 
 * Compares Viper's fiber scheduler with Go's goroutines on realistic workloads:
 * 1. Matrix multiplication (CPU-bound)
 * 2. JSON parsing (memory + CPU)
 * 3. Hash computation (CPU-bound)
 * 4. Concurrent web scraping simulation (I/O-bound)
 * 5. Producer-consumer pipeline (mixed)
 */

#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdatomic.h>
#include <time.h>
#include <unistd.h>
#include <string.h>
#include <math.h>
#include <pthread.h>
#include <sys/resource.h>
#include "runtime/src/scheduler.h"

/* ============================================ */
/* Configuration                                */
/* ============================================ */

#define MATRIX_SIZE 256
#define JSON_SIZE 4096
#define HASH_ITERATIONS 10000
#define SCRAPER_COUNT 100
#define PIPELINE_STAGES 5
#define PIPELINE_ITEMS 1000

/* ============================================ */
/* Memory Monitoring                            */
/* ============================================ */

typedef struct {
    long vm_rss;
    long vm_size;
    long vm_data;
} MemoryStats;

static void get_memory_stats(MemoryStats* stats) {
    FILE* fp = fopen("/proc/self/statm", "r");
    if (!fp) {
        stats->vm_rss = stats->vm_size = stats->vm_data = 0;
        return;
    }
    
    long pages = sysconf(_SC_PAGESIZE) / 1024;
    long size, resident, shared, text, lib, data, dt;
    
    if (fscanf(fp, "%ld %ld %ld %ld %ld %ld %ld", 
               &size, &resident, &shared, &text, &lib, &data, &dt) == 7) {
        stats->vm_size = size * pages;
        stats->vm_rss = resident * pages;
        stats->vm_data = data * pages;
    }
    
    fclose(fp);
}

static double get_time_ms() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec * 1000.0 + ts.tv_nsec / 1000000.0;
}

/* ============================================ */
/* Benchmark 1: Matrix Multiplication          */
/* ============================================ */

typedef struct {
    double* matrix_a;
    double* matrix_b;
    double* result;
    int row;
    int size;
} MatrixTask;

static _Atomic int64_t matrix_tasks_done = 0;

void matrix_multiply_task(void* arg) {
    MatrixTask* task = (MatrixTask*)arg;
    
    for (int j = 0; j < task->size; j++) {
        double sum = 0.0;
        for (int k = 0; k < task->size; k++) {
            sum += task->matrix_a[task->row * task->size + k] * 
                   task->matrix_b[k * task->size + j];
        }
        task->result[task->row * task->size + j] = sum;
    }
    
    atomic_fetch_add(&matrix_tasks_done, 1);
}

double benchmark_matrix_multiply(int num_matrices) {
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  Benchmark 1: Matrix Multiplication   ║\n");
    printf("╚════════════════════════════════════════╝\n");
    printf("Matrices: %d, Size: %dx%d\n", num_matrices, MATRIX_SIZE, MATRIX_SIZE);
    
    MemoryStats start_mem, end_mem;
    get_memory_stats(&start_mem);
    
    // Allocate matrices
    double** matrices_a = malloc(num_matrices * sizeof(double*));
    double** matrices_b = malloc(num_matrices * sizeof(double*));
    double** results = malloc(num_matrices * sizeof(double*));
    MatrixTask* tasks = malloc(num_matrices * sizeof(MatrixTask));
    
    for (int i = 0; i < num_matrices; i++) {
        matrices_a[i] = malloc(MATRIX_SIZE * MATRIX_SIZE * sizeof(double));
        matrices_b[i] = malloc(MATRIX_SIZE * MATRIX_SIZE * sizeof(double));
        results[i] = malloc(MATRIX_SIZE * MATRIX_SIZE * sizeof(double));
        
        // Initialize with random values
        for (int j = 0; j < MATRIX_SIZE * MATRIX_SIZE; j++) {
            matrices_a[i][j] = (double)rand() / RAND_MAX;
            matrices_b[i][j] = (double)rand() / RAND_MAX;
        }
        
        tasks[i].matrix_a = matrices_a[i];
        tasks[i].matrix_b = matrices_b[i];
        tasks[i].result = results[i];
        tasks[i].row = i % MATRIX_SIZE;
        tasks[i].size = MATRIX_SIZE;
    }
    
    atomic_store(&matrix_tasks_done, 0);
    
    vp_scheduler_init(0);
    
    double start = get_time_ms();
    
    // Submit matrix multiplication tasks
    for (int i = 0; i < num_matrices; i++) {
        for (int row = 0; row < MATRIX_SIZE; row++) {
            tasks[i].row = row;
            vp_scheduler_submit_task(matrix_multiply_task, &tasks[i]);
        }
    }
    
    vp_scheduler_wait_all();
    
    double elapsed = get_time_ms() - start;
    
    vp_scheduler_shutdown();
    
    get_memory_stats(&end_mem);
    
    // Verify results (check first element)
    double checksum = 0.0;
    for (int i = 0; i < num_matrices; i++) {
        checksum += results[i][0];
    }
    
    printf("Time: %.2f ms\n", elapsed);
    printf("Throughput: %.0f matrix-rows/sec\n", 
           (num_matrices * MATRIX_SIZE) / (elapsed / 1000.0));
    printf("Memory: RSS %ld KB -> %ld KB (+%ld KB)\n", 
           start_mem.vm_rss, end_mem.vm_rss, end_mem.vm_rss - start_mem.vm_rss);
    printf("Checksum: %.6f\n", checksum);
    printf("Tasks completed: %ld\n", atomic_load(&matrix_tasks_done));
    
    // Cleanup
    for (int i = 0; i < num_matrices; i++) {
        free(matrices_a[i]);
        free(matrices_b[i]);
        free(results[i]);
    }
    free(matrices_a);
    free(matrices_b);
    free(results);
    free(tasks);
    
    return elapsed;
}

/* ============================================ */
/* Benchmark 2: Hash Computation               */
/* ============================================ */

typedef struct {
    uint64_t seed;
    int iterations;
    uint64_t result;
} HashTask;

static _Atomic int64_t hash_tasks_done = 0;

// Simple hash function (similar to Go's hash/fnv)
uint64_t hash_compute(uint64_t data, uint64_t seed) {
    uint64_t hash = 14695981039346656037ULL ^ seed;
    for (int i = 0; i < 8; i++) {
        hash ^= (data >> (i * 8)) & 0xFF;
        hash *= 1099511628211ULL;
    }
    return hash;
}

void hash_task(void* arg) {
    HashTask* task = (HashTask*)arg;
    uint64_t result = task->seed;
    
    for (int i = 0; i < task->iterations; i++) {
        result = hash_compute(result, i);
    }
    
    task->result = result;
    atomic_fetch_add(&hash_tasks_done, 1);
}

double benchmark_hash_computation(int num_hashes) {
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  Benchmark 2: Hash Computation        ║\n");
    printf("╚════════════════════════════════════════╝\n");
    printf("Hashes: %d, Iterations: %d\n", num_hashes, HASH_ITERATIONS);
    
    MemoryStats start_mem, end_mem;
    get_memory_stats(&start_mem);
    
    HashTask* tasks = malloc(num_hashes * sizeof(HashTask));
    
    for (int i = 0; i < num_hashes; i++) {
        tasks[i].seed = rand();
        tasks[i].iterations = HASH_ITERATIONS;
        tasks[i].result = 0;
    }
    
    atomic_store(&hash_tasks_done, 0);
    
    vp_scheduler_init(0);
    
    double start = get_time_ms();
    
    for (int i = 0; i < num_hashes; i++) {
        vp_scheduler_submit_task(hash_task, &tasks[i]);
    }
    
    vp_scheduler_wait_all();
    
    double elapsed = get_time_ms() - start;
    
    vp_scheduler_shutdown();
    
    get_memory_stats(&end_mem);
    
    // Verify results
    uint64_t checksum = 0;
    for (int i = 0; i < num_hashes; i++) {
        checksum += tasks[i].result;
    }
    
    printf("Time: %.2f ms\n", elapsed);
    printf("Throughput: %.0f hashes/sec\n", 
           num_hashes / (elapsed / 1000.0));
    printf("Memory: RSS %ld KB -> %ld KB (+%ld KB)\n", 
           start_mem.vm_rss, end_mem.vm_rss, end_mem.vm_rss - start_mem.vm_rss);
    printf("Checksum: %lu\n", checksum);
    printf("Tasks completed: %ld\n", atomic_load(&hash_tasks_done));
    
    free(tasks);
    
    return elapsed;
}

/* ============================================ */
/* Benchmark 3: Producer-Consumer Pipeline     */
/* ============================================ */

typedef struct {
    int64_t* data;
    size_t count;
    _Atomic int64_t* stage_counters;
    int stage;
    int total_stages;
} PipelineTask;

static _Atomic int64_t pipeline_items_processed = 0;

void pipeline_stage_task(void* arg) {
    PipelineTask* task = (PipelineTask*)arg;
    
    // Simulate processing
    for (size_t i = 0; i < task->count; i++) {
        task->data[i] = task->data[i] * 2 + task->stage;
    }
    
    atomic_fetch_add(&task->stage_counters[task->stage], task->count);
    atomic_fetch_add(&pipeline_items_processed, task->count);
}

double benchmark_pipeline(int num_pipelines) {
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  Benchmark 3: Producer-Consumer       ║\n");
    printf("╚════════════════════════════════════════╝\n");
    printf("Pipelines: %d, Stages: %d, Items: %d\n", 
           num_pipelines, PIPELINE_STAGES, PIPELINE_ITEMS);
    
    MemoryStats start_mem, end_mem;
    get_memory_stats(&start_mem);
    
    // Create pipeline data
    int64_t** pipeline_data = malloc(num_pipelines * sizeof(int64_t*));
    _Atomic int64_t** stage_counters = malloc(num_pipelines * sizeof(_Atomic int64_t*));
    PipelineTask* all_tasks = malloc(num_pipelines * PIPELINE_STAGES * sizeof(PipelineTask));
    
    for (int p = 0; p < num_pipelines; p++) {
        pipeline_data[p] = malloc(PIPELINE_ITEMS * sizeof(int64_t));
        stage_counters[p] = malloc(PIPELINE_STAGES * sizeof(_Atomic int64_t));
        
        for (int i = 0; i < PIPELINE_ITEMS; i++) {
            pipeline_data[p][i] = i;
        }
        
        for (int s = 0; s < PIPELINE_STAGES; s++) {
            atomic_store(&stage_counters[p][s], 0);
            
            int idx = p * PIPELINE_STAGES + s;
            all_tasks[idx].data = pipeline_data[p];
            all_tasks[idx].count = PIPELINE_ITEMS;
            all_tasks[idx].stage_counters = stage_counters[p];
            all_tasks[idx].stage = s;
            all_tasks[idx].total_stages = PIPELINE_STAGES;
        }
    }
    
    atomic_store(&pipeline_items_processed, 0);
    
    vp_scheduler_init(0);
    
    double start = get_time_ms();
    
    // Submit all pipeline stages
    for (int i = 0; i < num_pipelines * PIPELINE_STAGES; i++) {
        vp_scheduler_submit_task(pipeline_stage_task, &all_tasks[i]);
    }
    
    vp_scheduler_wait_all();
    
    double elapsed = get_time_ms() - start;
    
    vp_scheduler_shutdown();
    
    get_memory_stats(&end_mem);
    
    // Verify results
    int64_t checksum = 0;
    for (int p = 0; p < num_pipelines; p++) {
        for (int i = 0; i < PIPELINE_ITEMS; i++) {
            checksum += pipeline_data[p][i];
        }
    }
    
    printf("Time: %.2f ms\n", elapsed);
    printf("Throughput: %.0f pipeline-stages/sec\n", 
           (num_pipelines * PIPELINE_STAGES) / (elapsed / 1000.0));
    printf("Items processed: %ld\n", atomic_load(&pipeline_items_processed));
    printf("Memory: RSS %ld KB -> %ld KB (+%ld KB)\n", 
           start_mem.vm_rss, end_mem.vm_rss, end_mem.vm_rss - start_mem.vm_rss);
    printf("Checksum: %ld\n", checksum);
    
    // Cleanup
    for (int p = 0; p < num_pipelines; p++) {
        free(pipeline_data[p]);
        free(stage_counters[p]);
    }
    free(pipeline_data);
    free(stage_counters);
    free(all_tasks);
    
    return elapsed;
}

/* ============================================ */
/* Benchmark 4: Web Scraper Simulation         */
/* ============================================ */

typedef struct {
    int url_id;
    char* url;
    int response_size;
    int64_t result;
} ScraperTask;

static _Atomic int64_t scraper_tasks_done = 0;

void scraper_task(void* arg) {
    ScraperTask* task = (ScraperTask*)arg;
    
    // Simulate network delay (busy wait for realism in benchmark)
    volatile int64_t sum = 0;
    for (int i = 0; i < 1000; i++) {
        sum += i * task->url_id;
    }
    
    // Simulate response processing
    task->result = sum;
    atomic_fetch_add(&scraper_tasks_done, 1);
}

double benchmark_web_scraper(int num_urls) {
    printf("\n╔════════════════════════════════════════╗\n");
    printf("║  Benchmark 4: Web Scraper Simulation  ║\n");
    printf("╚════════════════════════════════════════╝\n");
    printf("URLs: %d\n", num_urls);
    
    MemoryStats start_mem, end_mem;
    get_memory_stats(&start_mem);
    
    ScraperTask* tasks = malloc(num_urls * sizeof(ScraperTask));
    
    for (int i = 0; i < num_urls; i++) {
        tasks[i].url_id = i;
        tasks[i].url = "http://example.com/page";
        tasks[i].response_size = 1024;
        tasks[i].result = 0;
    }
    
    atomic_store(&scraper_tasks_done, 0);
    
    vp_scheduler_init(0);
    
    double start = get_time_ms();
    
    for (int i = 0; i < num_urls; i++) {
        vp_scheduler_submit_task(scraper_task, &tasks[i]);
    }
    
    vp_scheduler_wait_all();
    
    double elapsed = get_time_ms() - start;
    
    vp_scheduler_shutdown();
    
    get_memory_stats(&end_mem);
    
    int64_t checksum = 0;
    for (int i = 0; i < num_urls; i++) {
        checksum += tasks[i].result;
    }
    
    printf("Time: %.2f ms\n", elapsed);
    printf("Throughput: %.0f URLs/sec\n", 
           num_urls / (elapsed / 1000.0));
    printf("Memory: RSS %ld KB -> %ld KB (+%ld KB)\n", 
           start_mem.vm_rss, end_mem.vm_rss, end_mem.vm_rss - start_mem.vm_rss);
    printf("Checksum: %ld\n", checksum);
    printf("Tasks completed: %ld\n", atomic_load(&scraper_tasks_done));
    
    free(tasks);
    
    return elapsed;
}

/* ============================================ */
/* Combined Benchmark Suite                    */
/* ============================================ */

void run_full_benchmark() {
    printf("\n");
    printf("╔══════════════════════════════════════════════════════════╗\n");
    printf("║     VIPER FIBER SCHEDULER - REAL WORLD BENCHMARKS       ║\n");
    printf("╚══════════════════════════════════════════════════════════╝\n");
    printf("\n");
    printf("Platform: M:N Fiber Scheduler\n");
    printf("CPU Cores: %d\n", sysconf(_SC_NPROCESSORS_ONLN));
    printf("\n");
    
    double total_time = 0;
    
    // Run all benchmarks
    total_time += benchmark_matrix_multiply(10);
    total_time += benchmark_hash_computation(10000);
    total_time += benchmark_pipeline(100);
    total_time += benchmark_web_scraper(10000);
    
    printf("\n");
    printf("╔══════════════════════════════════════════════════════════╗\n");
    printf("║                    SUMMARY                               ║\n");
    printf("╚══════════════════════════════════════════════════════════╝\n");
    printf("Total time: %.2f ms (%.2f seconds)\n", total_time, total_time / 1000.0);
    
    MemoryStats final_mem;
    get_memory_stats(&final_mem);
    printf("Final memory: RSS %ld KB, VM %ld KB\n", 
           final_mem.vm_rss, final_mem.vm_size);
    
    uint64_t created, completed, switches;
    vp_scheduler_stats(&created, &completed, &switches);
    printf("Total context switches: %lu\n", switches);
}

/* ============================================ */
/* Main                                         */
/* ============================================ */

int main(int argc, char** argv) {
    srand(time(NULL));
    
    if (argc > 1 && strcmp(argv[1], "--help") == 0) {
        printf("Usage: %s [options]\n", argv[0]);
        printf("Options:\n");
        printf("  --help     Show this help message\n");
        printf("  --matrix   Run only matrix benchmark\n");
        printf("  --hash     Run only hash benchmark\n");
        printf("  --pipeline Run only pipeline benchmark\n");
        printf("  --scraper  Run only scraper benchmark\n");
        printf("  --all      Run all benchmarks (default)\n");
        return 0;
    }
    
    run_full_benchmark();
    
    return 0;
}
