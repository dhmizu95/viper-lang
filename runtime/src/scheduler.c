/**
 * Viper Fiber Scheduler
 * 
 * M:N fiber scheduler - runs fibers on thread pool.
 * Features:
 * - Work-stealing ready queue
 * - Per-thread run queues
 * - Fiber affinity support
 * - NUMA-aware scheduling
 */

#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <pthread.h>
#include <sched.h>
#include "fiber.h"

/* ============================================ */
/* Configuration                                */
/* ============================================ */

#define SCHEDULER_MAX_THREADS 512
#define SCHEDULER_DEFAULT_THREADS 4
#define FIBER_BATCH_SIZE 16

/* ============================================ */
/* Scheduler State                              */
/* ============================================ */

typedef struct SchedulerThread {
    pthread_t thread;
    int thread_id;
    ViperFiber* ready_queue[256];
    int ready_count;
    int ready_front;
    int ready_back;
    _Atomic bool running;
} SchedulerThread;

typedef struct ViperScheduler {
    /* Thread pool */
    SchedulerThread* threads;
    int num_threads;
    _Atomic int next_thread;
    
    /* Global ready queue (for work stealing) */
    ViperFiber* global_ready;
    _Atomic size_t global_ready_count;
    
    /* Sleeping fibers */
    ViperFiber* sleeping_fibers;
    pthread_mutex_t sleep_mutex;
    pthread_cond_t sleep_cond;
    
    /* Statistics */
    _Atomic uint64_t fibers_created;
    _Atomic uint64_t fibers_completed;
    _Atomic uint64_t context_switches;
    
    /* Control */
    _Atomic bool shutdown;
    pthread_mutex_t mutex;
    pthread_cond_t cond;
} ViperScheduler;

static ViperScheduler* g_scheduler = NULL;

/* ============================================ */
/* Queue Operations                            */
/* ============================================ */

static void queue_push(SchedulerThread* sched, ViperFiber* fiber) {
    if (!sched || !fiber) return;
    
    int idx = (sched->ready_back) % 256;
    sched->ready_queue[idx] = fiber;
    sched->ready_back++;
    sched->ready_count++;
}

static ViperFiber* queue_pop(SchedulerThread* sched) {
    if (!sched || sched->ready_count == 0) return NULL;
    
    int idx = (sched->ready_front) % 256;
    ViperFiber* fiber = sched->ready_queue[idx];
    sched->ready_queue[idx] = NULL;
    sched->ready_front++;
    sched->ready_count--;
    
    return fiber;
}

static ViperFiber* queue_steal(SchedulerThread* sched) {
    if (!sched) return NULL;
    
    /* Try to steal from front (oldest) */
    if (sched->ready_count > 0) {
        int idx = (sched->ready_front) % 256;
        ViperFiber* fiber = sched->ready_queue[idx];
        if (fiber) {
            sched->ready_queue[idx] = NULL;
            sched->ready_front++;
            sched->ready_count--;
            return fiber;
        }
    }
    
    return NULL;
}

/* ============================================ */
/* Scheduler Operations                        */
/* ============================================ */

void vp_scheduler_add_ready(ViperFiber* fiber) {
    if (!g_scheduler || !fiber) return;
    
    /* Simple: add to global queue */
    fiber->next_ready = g_scheduler->global_ready;
    g_scheduler->global_ready = fiber;
    atomic_fetch_add(&g_scheduler->global_ready_count, 1);
    
    /* Wake up a thread if needed */
    pthread_cond_signal(&g_scheduler->cond);
}

ViperFiber* vp_scheduler_get_ready(void) {
    if (!g_scheduler) return NULL;
    
    /* Try local queue first */
    int thread_id = sched_getcpu();
    if (thread_id >= 0 && thread_id < g_scheduler->num_threads) {
        ViperFiber* fiber = queue_pop(&g_scheduler->threads[thread_id]);
        if (fiber) return fiber;
    }
    
    /* Try global queue */
    ViperFiber* fiber = g_scheduler->global_ready;
    if (fiber) {
        ViperFiber* next = fiber->next_ready;
        if (atomic_compare_exchange_strong(
            (atomic_uintptr_t*)&g_scheduler->global_ready,
            (uintptr_t*)&fiber,
            (uintptr_t)next
        )) {
            atomic_fetch_sub(&g_scheduler->global_ready_count, 1);
            return fiber;
        }
    }
    
    /* Try to steal from other threads */
    for (int i = 0; i < g_scheduler->num_threads; i++) {
        if (i == thread_id) continue;
        fiber = queue_steal(&g_scheduler->threads[i]);
        if (fiber) return fiber;
    }
    
    return NULL;
}

void vp_scheduler_put_to_sleep(ViperFiber* fiber) {
    if (!g_scheduler || !fiber) return;
    
    /* Add to sleep queue */
    pthread_mutex_lock(&g_scheduler->sleep_mutex);
    fiber->next_ready = g_scheduler->sleeping_fibers;
    g_scheduler->sleeping_fibers = fiber;
    pthread_mutex_unlock(&g_scheduler->sleep_mutex);
}

/* ============================================ */
/* Worker Thread                               */
/* ============================================ */

static void* scheduler_worker(void* arg) {
    SchedulerThread* sched = (SchedulerThread*)arg;
    ViperFiber* current = NULL;
    
    while (!atomic_load(&g_scheduler->shutdown)) {
        /* Try to get a fiber to run */
        
        /* 1. Try local queue */
        current = queue_pop(sched);
        if (current) {
            vp_fiber_switch(NULL, current);
            atomic_fetch_add(&g_scheduler->context_switches, 1);
            
            if (current->state == FIBER_COMPLETED) {
                atomic_fetch_add(&g_scheduler->fibers_completed, 1);
                
                /* Resume parent if any */
                if (current->parent) {
                    vp_scheduler_add_ready(current->parent);
                }
            }
            continue;
        }
        
        /* 2. Try global queue */
        current = g_scheduler->global_ready;
        if (current) {
            ViperFiber* next = current->next_ready;
            if (atomic_compare_exchange_strong(
                (atomic_uintptr_t*)&g_scheduler->global_ready,
                (uintptr_t*)&current,
                (uintptr_t)next
            )) {
                atomic_fetch_sub(&g_scheduler->global_ready_count, 1);
                vp_fiber_switch(NULL, current);
                atomic_fetch_add(&g_scheduler->context_switches, 1);
                continue;
            }
        }
        
        /* 3. Try stealing from other threads */
        for (int i = 0; i < g_scheduler->num_threads; i++) {
            if (i == sched->thread_id) continue;
            current = queue_steal(&g_scheduler->threads[i]);
            if (current) {
                vp_fiber_switch(NULL, current);
                atomic_fetch_add(&g_scheduler->context_switches, 1);
                continue;
            }
        }
        
        /* 4. Nothing to do, wait */
        pthread_mutex_lock(&g_scheduler->mutex);
        if (atomic_load(&g_scheduler->global_ready_count) > 0) {
            pthread_mutex_unlock(&g_scheduler->mutex);
            continue;
        }
        
        /* Wait for work */
        struct timespec ts;
        clock_gettime(CLOCK_REALTIME, &ts);
        ts.tv_nsec += 1000000;  /* 1ms timeout */
        if (ts.tv_nsec >= 1000000000) {
            ts.tv_sec++;
            ts.tv_nsec -= 1000000000;
        }
        
        pthread_cond_timedwait(&g_scheduler->cond, &g_scheduler->mutex, &ts);
        pthread_mutex_unlock(&g_scheduler->mutex);
    }
    
    return NULL;
}

/* ============================================ */
/* Scheduler Lifecycle                         */
/* ============================================ */

ViperScheduler* vp_scheduler_create(int num_threads) {
    if (num_threads <= 0) {
        num_threads = SCHEDULER_DEFAULT_THREADS;
    }
    if (num_threads > SCHEDULER_MAX_THREADS) {
        num_threads = SCHEDULER_MAX_THREADS;
    }
    
    ViperScheduler* sched = (ViperScheduler*)malloc(sizeof(ViperScheduler));
    if (!sched) return NULL;
    
    memset(sched, 0, sizeof(ViperScheduler));
    
    sched->threads = (SchedulerThread*)malloc(sizeof(SchedulerThread) * num_threads);
    if (!sched->threads) {
        free(sched);
        return NULL;
    }
    
    memset(sched->threads, 0, sizeof(SchedulerThread) * num_threads);
    
    pthread_mutex_init(&sched->mutex, NULL);
    pthread_cond_init(&sched->cond, NULL);
    pthread_mutex_init(&sched->sleep_mutex, NULL);
    
    sched->num_threads = num_threads;
    
    /* Start worker threads */
    for (int i = 0; i < num_threads; i++) {
        sched->threads[i].thread_id = i;
        pthread_create(&sched->threads[i].thread, NULL, scheduler_worker, &sched->threads[i]);
    }
    
    g_scheduler = sched;
    return sched;
}

void vp_scheduler_destroy(ViperScheduler* sched) {
    if (!sched) return;
    
    /* Signal shutdown */
    atomic_store(&sched->shutdown, true);
    pthread_cond_broadcast(&sched->cond);
    
    /* Wait for threads */
    for (int i = 0; i < sched->num_threads; i++) {
        pthread_join(sched->threads[i].thread, NULL);
    }
    
    /* Clean up */
    pthread_mutex_destroy(&sched->mutex);
    pthread_cond_destroy(&sched->cond);
    pthread_mutex_destroy(&sched->sleep_mutex);
    
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

/**
 * Spawn a new fiber on the scheduler
 * @param func Function to run
 * @param arg Argument to pass
 * @return New fiber, or NULL on failure
 */
ViperFiber* vp_scheduler_spawn(void (*func)(void*), void* arg) {
    ViperFiber* fiber = vp_fiber_create(func, arg, 0);
    if (!fiber) return NULL;
    
    atomic_fetch_add(&g_scheduler->fibers_created, 1);
    
    vp_fiber_start(fiber);
    
    return fiber;
}

/**
 * Get scheduler statistics
 */
void vp_scheduler_stats(uint64_t* created, uint64_t* completed, uint64_t* switches) {
    if (g_scheduler) {
        if (created) *created = atomic_load(&g_scheduler->fibers_created);
        if (completed) *completed = atomic_load(&g_scheduler->fibers_completed);
        if (switches) *switches = atomic_load(&g_scheduler->context_switches);
    } else {
        if (created) *created = 0;
        if (completed) *completed = 0;
        if (switches) *switches = 0;
    }
}
