#ifndef VIPER_THREAD_POOL_H
#define VIPER_THREAD_POOL_H

#include <stdint.h>
#include <stddef.h>
#include <pthread.h>
#include <stdatomic.h>
#include "task_queue.h"

/**
 * Opaque ThreadPool type
 */
typedef struct ViperThreadPool ViperThreadPool;

/**
 * ThreadPool structure (internal details hidden)
 */
struct ViperThreadPool {
    pthread_t* threads;
    ViperTaskQueue** queues;
    void* contexts;  /* WorkerContext array */
    size_t num_threads;
    _Atomic bool running;
    _Atomic int64_t task_count;
    pthread_mutex_t mutex;
    pthread_cond_t cond;
};

/**
 * Create a new thread pool
 * @param num_threads Number of worker threads (0 for default)
 * @return Pointer to ThreadPool, or NULL on failure
 */
ViperThreadPool* vp_threadpool_create(size_t num_threads);

/**
 * Destroy a thread pool (waits for all tasks to complete)
 * @param pool ThreadPool to destroy
 */
void vp_threadpool_destroy(ViperThreadPool* pool);

/**
 * Submit a task to the thread pool
 * @param pool ThreadPool
 * @param task Task to submit
 */
void vp_threadpool_submit(ViperThreadPool* pool, ViperTask* task);

/**
 * Submit a task to a specific worker queue
 * @param pool ThreadPool
 * @param queue_idx Worker queue index
 * @param task Task to submit
 */
void vp_threadpool_submit_to(ViperThreadPool* pool, size_t queue_idx, ViperTask* task);

/**
 * Wait for all pending tasks to complete
 * @param pool ThreadPool
 */
void vp_threadpool_wait(ViperThreadPool* pool);

/**
 * Get the number of threads in the pool
 * @param pool ThreadPool
 * @return Number of threads
 */
size_t vp_threadpool_num_threads(ViperThreadPool* pool);

/**
 * Get the number of pending tasks
 * @param pool ThreadPool
 * @return Number of pending tasks
 */
int64_t vp_threadpool_pending_tasks(ViperThreadPool* pool);

#endif /* VIPER_THREAD_POOL_H */
