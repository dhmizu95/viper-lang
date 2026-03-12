/**
 * Viper Task Memory Monitor - C Runtime Test
 * 
 * Monitors memory usage while running concurrent fiber tasks.
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdatomic.h>
#include <time.h>
#include <unistd.h>
#include <sys/resource.h>
#include "runtime/src/scheduler.h"

/* ============================================ */
/* Memory Monitoring                            */
/* ============================================ */

typedef struct {
    long vm_size;      /* Virtual memory size (KB) */
    long vm_rss;       /* Resident set size (KB) */
    long vm_shared;    /* Shared memory (KB) */
    long vm_data;      /* Data segment (KB) */
} MemoryStats;

static void get_memory_stats(MemoryStats* stats) {
    FILE* fp = fopen("/proc/self/statm", "r");
    if (!fp) {
        stats->vm_size = 0;
        stats->vm_rss = 0;
        stats->vm_shared = 0;
        stats->vm_data = 0;
        return;
    }
    
    long pages = sysconf(_SC_PAGESIZE) / 1024;  /* KB per page */
    long size, resident, shared, text, lib, data, dt;
    
    if (fscanf(fp, "%ld %ld %ld %ld %ld %ld %ld", 
               &size, &resident, &shared, &text, &lib, &data, &dt) == 7) {
        stats->vm_size = size * pages;
        stats->vm_rss = resident * pages;
        stats->vm_shared = shared * pages;
        stats->vm_data = data * pages;
    }
    
    fclose(fp);
}

static void print_memory_stats(const char* label, MemoryStats* stats) {
    printf("%-20s: VM=%ld KB, RSS=%ld KB, Shared=%ld KB, Data=%ld KB\n",
           label, stats->vm_size, stats->vm_rss, 
           stats->vm_shared, stats->vm_data);
}

/* ============================================ */
/* Task Definitions                             */
/* ============================================ */

static _Atomic int64_t tasks_completed = 0;
static _Atomic int64_t total_allocations = 0;

typedef struct {
    int64_t id;
    int64_t* data;
    size_t data_size;
} TaskContext;

void simple_task(void* arg) {
    (void)arg;
    atomic_fetch_add(&tasks_completed, 1);
}

void memory_task(void* arg) {
    TaskContext* ctx = (TaskContext*)arg;
    
    /* Allocate some memory to simulate work */
    ctx->data = (int64_t*)malloc(ctx->data_size * sizeof(int64_t));
    if (ctx->data) {
        atomic_fetch_add(&total_allocations, ctx->data_size);
        
        /* Initialize data */
        for (size_t i = 0; i < ctx->data_size; i++) {
            ctx->data[i] = ctx->id * 1000 + i;
        }
        
        /* Simulate some work */
        volatile int64_t sum = 0;
        for (size_t i = 0; i < ctx->data_size; i++) {
            sum += ctx->data[i];
        }
        
        /* Free memory */
        free(ctx->data);
    }
    
    atomic_fetch_add(&tasks_completed, 1);
}

/* ============================================ */
/* Benchmark Functions                          */
/* ============================================ */

void run_simple_benchmark(int64_t num_tasks) {
    printf("\n=== Simple Task Benchmark ===\n");
    printf("Tasks: %ld\n", num_tasks);
    
    MemoryStats start_mem, end_mem;
    get_memory_stats(&start_mem);
    print_memory_stats("Before", &start_mem);
    
    atomic_store(&tasks_completed, 0);
    
    double start_time = clock();
    
    vp_scheduler_init(0);
    
    for (int64_t i = 0; i < num_tasks; i++) {
        vp_scheduler_submit_task(simple_task, NULL);
    }
    
    vp_scheduler_wait_all();
    vp_scheduler_shutdown();
    
    double end_time = clock();
    double elapsed = (end_time - start_time) / CLOCKS_PER_SEC;
    
    get_memory_stats(&end_mem);
    print_memory_stats("After", &end_mem);
    
    printf("Completed: %ld tasks in %.3f seconds\n", 
           atomic_load(&tasks_completed), elapsed);
    printf("Throughput: %.0f tasks/sec\n", 
           num_tasks / elapsed);
    
    uint64_t created, completed, switches;
    vp_scheduler_stats(&created, &completed, &switches);
    printf("Context switches: %lu\n", switches);
}

void run_memory_benchmark(int64_t num_tasks, size_t alloc_per_task) {
    printf("\n=== Memory Allocation Benchmark ===\n");
    printf("Tasks: %ld, Allocation per task: %zu bytes\n", 
           num_tasks, alloc_per_task * sizeof(int64_t));
    
    MemoryStats start_mem, peak_mem, end_mem;
    get_memory_stats(&start_mem);
    print_memory_stats("Before", &start_mem);
    
    atomic_store(&tasks_completed, 0);
    atomic_store(&total_allocations, 0);
    
    TaskContext* contexts = (TaskContext*)malloc(num_tasks * sizeof(TaskContext));
    if (!contexts) {
        printf("Failed to allocate contexts\n");
        return;
    }
    
    for (int64_t i = 0; i < num_tasks; i++) {
        contexts[i].id = i;
        contexts[i].data = NULL;
        contexts[i].data_size = alloc_per_task;
    }
    
    vp_scheduler_init(0);
    
    for (int64_t i = 0; i < num_tasks; i++) {
        vp_scheduler_submit_task(memory_task, &contexts[i]);
        
        /* Check memory periodically */
        if (i % (num_tasks / 10 + 1) == 0) {
            get_memory_stats(&peak_mem);
            printf("Progress: %ld/%ld tasks, RSS: %ld KB\n", 
                   i, num_tasks, peak_mem.vm_rss);
        }
    }
    
    vp_scheduler_wait_all();
    vp_scheduler_shutdown();
    
    get_memory_stats(&end_mem);
    print_memory_stats("After", &end_mem);
    
    printf("Completed: %ld tasks\n", atomic_load(&tasks_completed));
    printf("Total allocations: %ld elements (%.2f MB)\n", 
           atomic_load(&total_allocations),
           atomic_load(&total_allocations) * sizeof(int64_t) / (1024.0 * 1024.0));
    
    free(contexts);
}

/* ============================================ */
/* Main                                         */
/* ============================================ */

int main(int argc, char** argv) {
    printf("╔════════════════════════════════════════════╗\n");
    printf("║   Viper Fiber Task Memory Monitor         ║\n");
    printf("╚════════════════════════════════════════════╝\n");
    
    int64_t simple_tasks = 100000;
    int64_t memory_tasks = 10000;
    size_t alloc_size = 100;  /* 100 int64_t per task = 800 bytes */
    
    if (argc > 1) {
        simple_tasks = atoll(argv[1]);
    }
    if (argc > 2) {
        memory_tasks = atoll(argv[2]);
    }
    if (argc > 3) {
        alloc_size = atoi(argv[3]);
    }
    
    /* Run benchmarks */
    run_simple_benchmark(simple_tasks);
    run_memory_benchmark(memory_tasks, alloc_size);
    
    printf("\n=== Summary ===\n");
    MemoryStats final_mem;
    get_memory_stats(&final_mem);
    print_memory_stats("Final memory", &final_mem);
    
    return 0;
}
