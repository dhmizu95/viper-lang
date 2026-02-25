/**
 * Viper Thread Pool Implementation
 * 
 * A work-stealing thread pool that schedules M tasks onto N worker threads.
 * Each worker has its own task queue - it processes from its own queue
 * and steals from others when idle.
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>
#include <stdatomic.h>
#include <unistd.h>
#include <sys/sysinfo.h>
#include "thread_pool.h"
#include "task_queue.h"

/* ============================================ */
/* Helper Functions                            */
/* ============================================ */

static size_t detect_cpu_count(void) {
    int cpus = get_nprocs();
    if (cpus <= 0) return 4;
    return (size_t)cpus;
}

/* ============================================ */
/* Constants                                    */
/* ============================================ */

#define VIPER_DEFAULT_THREAD_COUNT 0  /* 0 = auto-detect */
#define VIPER_MAX_THREAD_COUNT 512

/* ============================================ */
/* Internal Structures                          */
/* ============================================ */

typedef struct {
    ViperThreadPool* pool;
    size_t worker_id;
} WorkerContext;

/* ============================================ */
/* Worker Thread Function                       */
/* ============================================ */

static void* worker_thread(void* arg) {
    WorkerContext* ctx = (WorkerContext*)arg;
    ViperThreadPool* pool = ctx->pool;
    size_t worker_id = ctx->worker_id;
    
    while (atomic_load(&pool->running)) {
        ViperTask* task = NULL;
        
        /* Try to get task from own queue first */
        task = vp_taskqueue_pop(pool->queues[worker_id]);
        
        /* If no task, try to steal from others */
        if (!task) {
            for (size_t i = 0; i < pool->num_threads; i++) {
                if (i != worker_id) {
                    task = vp_taskqueue_steal(pool->queues[i]);
                    if (task) break;
                }
            }
        }
        
        /* Execute task if found */
        if (task) {
            vp_task_execute(task);
            if (task->cleanup) {
                task->cleanup(task);
            }
            free(task);
        } else {
            /* No work available - wait briefly */
            pthread_mutex_lock(&pool->mutex);
            
            if (!atomic_load(&pool->running)) {
                pthread_mutex_unlock(&pool->mutex);
                break;
            }
            
            /* Wait for new work or shutdown */
            struct timespec ts;
            clock_gettime(CLOCK_REALTIME, &ts);
            ts.tv_nsec += 10000000;  /* 10ms timeout */
            if (ts.tv_nsec >= 1000000000) {
                ts.tv_sec++;
                ts.tv_nsec -= 1000000000;
            }
            
            pthread_cond_timedwait(&pool->cond, &pool->mutex, &ts);
            pthread_mutex_unlock(&pool->mutex);
        }
    }
    
    return NULL;
}

/* ============================================ */
/* Thread Pool Functions                        */
/* ============================================ */

ViperThreadPool* vp_threadpool_create(size_t num_threads) {
    if (num_threads == 0) {
        num_threads = detect_cpu_count();
    }
    if (num_threads > VIPER_MAX_THREAD_COUNT) {
        num_threads = VIPER_MAX_THREAD_COUNT;
    }
    
    ViperThreadPool* pool = (ViperThreadPool*)malloc(sizeof(ViperThreadPool));
    if (!pool) {
        return NULL;
    }
    
    pool->num_threads = num_threads;
    atomic_store(&pool->running, true);
    atomic_store(&pool->task_count, 0);
    
    pthread_mutex_init(&pool->mutex, NULL);
    pthread_cond_init(&pool->cond, NULL);
    
    /* Create worker queues */
    pool->queues = (ViperTaskQueue**)calloc(num_threads, sizeof(ViperTaskQueue*));
    if (!pool->queues) {
        free(pool);
        return NULL;
    }
    
    for (size_t i = 0; i < num_threads; i++) {
        pool->queues[i] = vp_taskqueue_create();
        if (!pool->queues[i]) {
            /* Cleanup on failure */
            for (size_t j = 0; j < i; j++) {
                vp_taskqueue_destroy(pool->queues[j]);
            }
            free(pool->queues);
            free(pool);
            return NULL;
        }
    }
    
    /* Create worker threads */
    pool->threads = (pthread_t*)calloc(num_threads, sizeof(pthread_t));
    if (!pool->threads) {
        for (size_t i = 0; i < num_threads; i++) {
            vp_taskqueue_destroy(pool->queues[i]);
        }
        free(pool->queues);
        free(pool);
        return NULL;
    }
    
    WorkerContext* contexts = (WorkerContext*)malloc(num_threads * sizeof(WorkerContext));
    if (!contexts) {
        free(pool->threads);
        for (size_t i = 0; i < num_threads; i++) {
            vp_taskqueue_destroy(pool->queues[i]);
        }
        free(pool->queues);
        free(pool);
        return NULL;
    }
    
    for (size_t i = 0; i < num_threads; i++) {
        contexts[i].pool = pool;
        contexts[i].worker_id = i;
        
        if (pthread_create(&pool->threads[i], NULL, worker_thread, &contexts[i]) != 0) {
            /* Cleanup on failure */
            atomic_store(&pool->running, false);
            pthread_cond_broadcast(&pool->cond);
            
            for (size_t j = 0; j <= i; j++) {
                pthread_join(pool->threads[j], NULL);
            }
            
            free(contexts);
            free(pool->threads);
            for (size_t j = 0; j < num_threads; j++) {
                vp_taskqueue_destroy(pool->queues[j]);
            }
            free(pool->queues);
            free(pool);
            return NULL;
        }
    }
    
    pool->contexts = contexts;
    return pool;
}

void vp_threadpool_destroy(ViperThreadPool* pool) {
    if (!pool) return;
    
    /* Signal shutdown */
    atomic_store(&pool->running, false);
    pthread_cond_broadcast(&pool->cond);
    
    /* Wait for all workers to finish */
    for (size_t i = 0; i < pool->num_threads; i++) {
        pthread_join(pool->threads[i], NULL);
    }
    
    /* Cleanup */
    free(pool->contexts);
    free(pool->threads);
    
    for (size_t i = 0; i < pool->num_threads; i++) {
        vp_taskqueue_destroy(pool->queues[i]);
    }
    free(pool->queues);
    
    pthread_mutex_destroy(&pool->mutex);
    pthread_cond_destroy(&pool->cond);
    free(pool);
}

void vp_threadpool_submit(ViperThreadPool* pool, ViperTask* task) {
    if (!pool || !task) return;
    
    /* Simple round-robin distribution */
    static _Atomic size_t next_queue = 0;
    size_t queue_idx = atomic_fetch_add(&next_queue, 1) % pool->num_threads;
    
    vp_taskqueue_push(pool->queues[queue_idx], task);
    atomic_fetch_add(&pool->task_count, 1);
    
    /* Wake up a worker */
    pthread_cond_signal(&pool->cond);
}

void vp_threadpool_submit_to(ViperThreadPool* pool, size_t queue_idx, ViperTask* task) {
    if (!pool || !task) return;
    
    if (queue_idx >= pool->num_threads) {
        queue_idx = 0;
    }
    
    vp_taskqueue_push(pool->queues[queue_idx], task);
    atomic_fetch_add(&pool->task_count, 1);
    
    pthread_cond_signal(&pool->cond);
}

void vp_threadpool_wait(ViperThreadPool* pool) {
    if (!pool) return;
    
    pthread_mutex_lock(&pool->mutex);
    
    while (atomic_load(&pool->task_count) > 0) {
        /* Check if all queues are empty */
        bool all_empty = true;
        for (size_t i = 0; i < pool->num_threads; i++) {
            if (!vp_taskqueue_is_empty(pool->queues[i])) {
                all_empty = false;
                break;
            }
        }
        
        if (all_empty) {
            break;
        }
        
        pthread_cond_wait(&pool->cond, &pool->mutex);
    }
    
    pthread_mutex_unlock(&pool->mutex);
}

size_t vp_threadpool_num_threads(ViperThreadPool* pool) {
    return pool ? pool->num_threads : 0;
}

int64_t vp_threadpool_pending_tasks(ViperThreadPool* pool) {
    if (!pool) return 0;
    return atomic_load(&pool->task_count);
}
