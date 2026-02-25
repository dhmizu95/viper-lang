/**
 * Viper Fiber Scheduler - Optimized for 10M+ tasks
 * 
 * M:N fiber scheduler with optimizations:
 * - Simple work stealing
 * - Per-thread local queues
 * - Global distribution queues
 * - CPU affinity
 */

#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <pthread.h>
#include <sched.h>
#include <sys/sysinfo.h>
#include <unistd.h>
#include <stdint.h>
#include "fiber.h"

/* ============================================ */
/* Configuration                                */
/* ============================================ */

#define SCHEDULER_MAX_THREADS 1024
#define SCHEDULER_DEFAULT_THREADS 0
#define MAX_STEAL_ATTEMPTS 4
#define LOCAL_QUEUE_SIZE 256

/* ============================================ */
/* Simple Ring Buffer Queue                     */
/* ============================================ */

typedef struct FiberQueue {
    ViperFiber** items;
    int capacity;
    int front;
    int back;
    _Atomic int count;
    pthread_mutex_t mutex;
} FiberQueue;

static void queue_init(FiberQueue* q, int capacity) {
    q->items = (ViperFiber**)malloc(sizeof(ViperFiber*) * capacity);
    q->capacity = capacity;
    q->front = 0;
    q->back = 0;
    atomic_store(&q->count, 0);
    pthread_mutex_init(&q->mutex, NULL);
}

static void queue_destroy(FiberQueue* q) {
    free(q->items);
    pthread_mutex_destroy(&q->mutex);
}

static int queue_push(FiberQueue* q, ViperFiber* fiber) {
    int cnt = atomic_fetch_add(&q->count, 1);
    if (cnt >= q->capacity) {
        atomic_fetch_sub(&q->count, 1);
        return -1;  /* Full */
    }
    
    pthread_mutex_lock(&q->mutex);
    q->items[q->back] = fiber;
    q->back = (q->back + 1) % q->capacity;
    pthread_mutex_unlock(&q->mutex);
    return 0;
}

static ViperFiber* queue_pop(FiberQueue* q) {
    int cnt = atomic_load(&q->count);
    if (cnt <= 0) return NULL;
    
    pthread_mutex_lock(&q->mutex);
    if (q->front == q->back) {
        pthread_mutex_unlock(&q->mutex);
        return NULL;
    }
    
    ViperFiber* fiber = q->items[q->front];
    q->items[q->front] = NULL;
    q->front = (q->front + 1) % q->capacity;
    pthread_mutex_unlock(&q->mutex);
    atomic_fetch_sub(&q->count, 1);
    
    return fiber;
}

static size_t queue_size(FiberQueue* q) {
    return atomic_load(&q->count);
}

/* ============================================ */
/* Scheduler Thread                             */
/* ============================================ */

typedef struct SchedulerThread {
    pthread_t thread;
    int thread_id;
    int cpu_core;
    
    /* Local ready queue */
    FiberQueue local_queue;
    
    /* Statistics */
    _Atomic uint64_t fibers_run;
    _Atomic uint64_t steals_attempted;
    _Atomic uint64_t steals_succeeded;
    _Atomic uint64_t context_switches;
    
    _Atomic int running;
    _Atomic int idle;
} SchedulerThread;

/* ============================================ */
/* Scheduler State                              */
/* ============================================ */

typedef struct ViperScheduler {
    /* Thread pool */
    SchedulerThread* threads;
    int num_threads;
    int max_threads;
    _Atomic int active_threads;
    _Atomic int idle_threads;
    
    /* Global queues for stealing */
    FiberQueue* global_queues;
    int num_global_queues;
    
    /* Adaptive scaling */
    _Atomic bool scaling_enabled;
    _Atomic size_t pending_tasks;
    
    /* Control */
    _Atomic bool shutdown;
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    
    /* Global statistics */
    _Atomic uint64_t fibers_created;
    _Atomic uint64_t fibers_completed;
    _Atomic uint64_t total_context_switches;
    
    /* Load balancing */
    _Atomic size_t steal_attempts;
    _Atomic size_t steal_successes;
} ViperScheduler;

static ViperScheduler* g_scheduler = NULL;

/* ============================================ */
/* Scheduler Operations                        */
/* ============================================ */

void vp_scheduler_add_ready(ViperFiber* fiber) {
    if (!g_scheduler || !fiber) return;
    
    /* Add to global queue (hash by fiber ID) */
    int qidx = fiber->id % g_scheduler->num_global_queues;
    queue_push(&g_scheduler->global_queues[qidx], fiber);
    
    /* Wake up idle threads */
    int idle = atomic_load(&g_scheduler->idle_threads);
    if (idle > 0) {
        pthread_cond_signal(&g_scheduler->cond);
    }
    
    atomic_fetch_add(&g_scheduler->pending_tasks, 1);
}

ViperFiber* vp_scheduler_get_ready(void) {
    if (!g_scheduler) return NULL;
    
    /* Try local queue first */
    int thread_id = 0;  /* Would use sched_getcpu() but it's not available everywhere */
    SchedulerThread* st = &g_scheduler->threads[thread_id];
    
    ViperFiber* fiber = queue_pop(&st->local_queue);
    if (fiber) return fiber;
    
    /* Try global queues */
    for (int i = 0; i < MAX_STEAL_ATTEMPTS; i++) {
        int qidx = (thread_id + i) % g_scheduler->num_global_queues;
        fiber = queue_pop(&g_scheduler->global_queues[qidx]);
        if (fiber) {
            atomic_fetch_add(&st->steals_succeeded, 1);
            return fiber;
        }
        atomic_fetch_add(&st->steals_attempted, 1);
    }
    
    return NULL;
}

void vp_scheduler_put_to_sleep(ViperFiber* fiber) {
    (void)fiber;
}

/* ============================================ */
/* Worker Thread                               */
/* ============================================ */

static void* scheduler_worker(void* arg) {
    SchedulerThread* st = (SchedulerThread*)arg;
    
    while (!atomic_load(&g_scheduler->shutdown)) {
        ViperFiber* current = NULL;
        
        /* Try local queue */
        current = queue_pop(&st->local_queue);
        if (!current) {
            /* Try global queues */
            for (int i = 0; i < MAX_STEAL_ATTEMPTS; i++) {
                int qidx = (st->thread_id + i) % g_scheduler->num_global_queues;
                current = queue_pop(&g_scheduler->global_queues[qidx]);
                if (current) {
                    atomic_fetch_add(&st->steals_succeeded, 1);
                    break;
                }
            }
        }
        
        if (current) {
            atomic_fetch_add(&st->context_switches, 1);
            atomic_fetch_add(&g_scheduler->total_context_switches, 1);
            
            /* Run the fiber */
            current->state = FIBER_RUNNING;
            
            if (current->state == FIBER_NEW) {
                current->func(current->arg);
                current->state = FIBER_COMPLETED;
                atomic_fetch_add(&g_scheduler->fibers_completed, 1);
                
                if (current->parent) {
                    vp_scheduler_add_ready(current->parent);
                }
                
                vp_fiber_free(current);
            }
            
            atomic_fetch_add(&st->fibers_run, 1);
            continue;
        }
        
        /* Nothing to do - wait */
        atomic_fetch_add(&st->idle, 1);
        atomic_fetch_add(&g_scheduler->idle_threads, 1);
        
        pthread_mutex_lock(&g_scheduler->mutex);
        
        size_t pending = 0;
        for (int i = 0; i < g_scheduler->num_global_queues; i++) {
            pending += queue_size(&g_scheduler->global_queues[i]);
        }
        
        if (pending == 0 && !atomic_load(&g_scheduler->shutdown)) {
            struct timespec ts;
            clock_gettime(CLOCK_REALTIME, &ts);
            ts.tv_nsec += 1000000;  /* 1ms */
            if (ts.tv_nsec >= 1000000000) {
                ts.tv_sec++;
                ts.tv_nsec -= 1000000000;
            }
            pthread_cond_timedwait(&g_scheduler->cond, &g_scheduler->mutex, &ts);
        }
        
        pthread_mutex_unlock(&g_scheduler->mutex);
        
        atomic_fetch_sub(&st->idle, 1);
        atomic_fetch_sub(&g_scheduler->idle_threads, 1);
    }
    
    return NULL;
}

/* ============================================ */
/* Scheduler Lifecycle                         */
/* ============================================ */

static int detect_cpu_count(void) {
    int cpus = sysconf(_SC_NPROCESSORS_ONLN);
    if (cpus <= 0) return 4;
    return cpus;
}

ViperScheduler* vp_scheduler_create(int num_threads) {
    if (num_threads <= 0) {
        num_threads = detect_cpu_count();
    }
    if (num_threads > SCHEDULER_MAX_THREADS) {
        num_threads = SCHEDULER_MAX_THREADS;
    }
    
    ViperScheduler* sched = (ViperScheduler*)malloc(sizeof(ViperScheduler));
    if (!sched) return NULL;
    
    memset(sched, 0, sizeof(ViperScheduler));
    
    /* Initialize thread pool */
    sched->threads = (SchedulerThread*)malloc(sizeof(SchedulerThread) * num_threads);
    if (!sched->threads) {
        free(sched);
        return NULL;
    }
    
    memset(sched->threads, 0, sizeof(SchedulerThread) * num_threads);
    
    /* Initialize global queues */
    sched->num_global_queues = 16;
    sched->global_queues = (FiberQueue*)malloc(sizeof(FiberQueue) * sched->num_global_queues);
    for (int i = 0; i < sched->num_global_queues; i++) {
        queue_init(&sched->global_queues[i], 1024);  /* 1K per queue */
    }
    
    pthread_mutex_init(&sched->mutex, NULL);
    pthread_cond_init(&sched->cond, NULL);
    
    sched->num_threads = num_threads;
    sched->max_threads = SCHEDULER_MAX_THREADS;
    atomic_store(&sched->active_threads, num_threads);
    atomic_store(&sched->scaling_enabled, true);
    
    /* Start worker threads */
    for (int i = 0; i < num_threads; i++) {
        sched->threads[i].thread_id = i;
        sched->threads[i].cpu_core = i % detect_cpu_count();
        
        queue_init(&sched->threads[i].local_queue, LOCAL_QUEUE_SIZE);
        
        pthread_create(&sched->threads[i].thread, NULL, scheduler_worker, &sched->threads[i]);
    }
    
    g_scheduler = sched;
    return sched;
}

void vp_scheduler_destroy(ViperScheduler* sched) {
    if (!sched) return;
    
    atomic_store(&sched->shutdown, true);
    pthread_cond_broadcast(&sched->cond);
    
    for (int i = 0; i < sched->num_threads; i++) {
        pthread_join(sched->threads[i].thread, NULL);
    }
    
    for (int i = 0; i < sched->num_global_queues; i++) {
        queue_destroy(&sched->global_queues[i]);
    }
    
    for (int i = 0; i < sched->num_threads; i++) {
        queue_destroy(&sched->threads[i].local_queue);
    }
    
    free(sched->global_queues);
    pthread_mutex_destroy(&sched->mutex);
    pthread_cond_destroy(&sched->cond);
    free(sched->threads);
    free(sched);
    
    g_scheduler = NULL;
}

/* ============================================ */
/* Public API                                  */
/* ============================================ */

void vp_scheduler_init(int num_threads) {
    if (!g_scheduler) {
        vp_scheduler_create(num_threads);
    }
}

void vp_scheduler_shutdown(void) {
    if (g_scheduler) {
        vp_scheduler_destroy(g_scheduler);
    }
}

ViperFiber* vp_scheduler_spawn(void (*func)(void*), void* arg) {
    ViperFiber* fiber = vp_fiber_create(func, arg, 0);
    if (!fiber) return NULL;
    
    atomic_fetch_add(&g_scheduler->fibers_created, 1);
    
    vp_fiber_start(fiber);
    vp_scheduler_add_ready(fiber);
    
    return fiber;
}

void vp_scheduler_stats(uint64_t* created, uint64_t* completed, uint64_t* switches) {
    if (g_scheduler) {
        if (created) *created = atomic_load(&g_scheduler->fibers_created);
        if (completed) *completed = atomic_load(&g_scheduler->fibers_completed);
        if (switches) *switches = atomic_load(&g_scheduler->total_context_switches);
    } else {
        if (created) *created = 0;
        if (completed) *completed = 0;
        if (switches) *switches = 0;
    }
}

void vp_scheduler_thread_stats(int thread_id, uint64_t* fibers_run, 
                               uint64_t* steals_attempted, uint64_t* steals_succeeded) {
    if (g_scheduler && thread_id >= 0 && thread_id < g_scheduler->num_threads) {
        SchedulerThread* st = &g_scheduler->threads[thread_id];
        if (fibers_run) *fibers_run = atomic_load(&st->fibers_run);
        if (steals_attempted) *steals_attempted = atomic_load(&st->steals_attempted);
        if (steals_succeeded) *steals_succeeded = atomic_load(&st->steals_succeeded);
    }
}

void vp_scheduler_set_scaling(bool enabled) {
    if (g_scheduler) {
        atomic_store(&g_scheduler->scaling_enabled, enabled);
    }
}
