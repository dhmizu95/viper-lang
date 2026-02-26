/**
 * Viper Concurrency Runtime
 * 
 * High-level concurrency runtime that provides the interface
 * for Viper's M:N threading system.
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdatomic.h>
#include "viper_stdlib.h"
#include "channel.h"
#include "wait_group.h"
#include "thread_pool.h"

/* ============================================ */
/* Global State                                 */
/* ============================================ */

static ViperThreadPool* _Atomic g_thread_pool = NULL;

/* ============================================ */
/* Channel Operations                           */
/* ============================================ */

ViperChannel* vp_chan_create(int64_t capacity) {
    return vp_channel_create((size_t)capacity);
}

void vp_chan_destroy(ViperChannel* chan) {
    vp_channel_destroy(chan);
}

void vp_chan_send(ViperChannel* chan, int64_t value) {
    vp_channel_send(chan, value);
}

int64_t vp_chan_recv(ViperChannel* chan) {
    return vp_channel_recv(chan);
}

/* ============================================ */
/* WaitGroup Operations                         */
/* ============================================ */

ViperWaitGroup* vp_waitgroup_create(void) {
    return vp_waitgroup_create_impl();
}

void vp_waitgroup_destroy(ViperWaitGroup* wg) {
    vp_waitgroup_destroy_impl(wg);
}

void vp_waitgroup_add(ViperWaitGroup* wg, int64_t n) {
    vp_waitgroup_add_impl(wg, n);
}

void vp_waitgroup_done(ViperWaitGroup* wg) {
    vp_waitgroup_done_impl(wg);
}

void vp_waitgroup_wait(ViperWaitGroup* wg) {
    vp_waitgroup_wait_impl(wg);
}

/* ============================================ */
/* Thread Pool Operations                       */
/* ============================================ */

void vp_init_threadpool(size_t num_threads) {
    ViperThreadPool* pool = vp_threadpool_create(num_threads);
    atomic_store(&g_thread_pool, pool);
}

void vp_shutdown_threadpool(void) {
    ViperThreadPool* pool = atomic_exchange(&g_thread_pool, NULL);
    if (pool) {
        vp_threadpool_destroy(pool);
    }
}

void vp_submit_task(void (*func)(void*), void* data) {
    ViperThreadPool* pool = atomic_load(&g_thread_pool);
    if (!pool) {
        /* Auto-initialize with default thread count */
        vp_init_threadpool(0);
        pool = atomic_load(&g_thread_pool);
    }
    
    if (pool) {
        ViperTask* task = vp_task_create(func, data, NULL);
        vp_threadpool_submit(pool, task);
    }
}

void vp_wait_all_tasks(void) {
    ViperThreadPool* pool = atomic_load(&g_thread_pool);
    if (pool) {
        vp_threadpool_wait(pool);
    }
}

/* ============================================ */
/* Task Execution Helper                        */
/* ============================================ */

typedef struct {
    void (*func)(void);
    void* context;
} TaskClosure;

static void task_wrapper(void* data) {
    TaskClosure* closure = (TaskClosure*)data;
    if (closure && closure->func) {
        closure->func();
    }
    free(closure);
}

void vp_spawn_task(void (*func)(void)) {
    TaskClosure* closure = (TaskClosure*)malloc(sizeof(TaskClosure));
    if (closure) {
        closure->func = func;
        closure->context = NULL;
        vp_submit_task(task_wrapper, closure);
    }
}

/* ============================================ */
/* Thread Spawning (true parallelism)           */
/* ============================================ */

#include <pthread.h>

static _Atomic int64_t g_thread_counter = 0;

typedef struct {
    void (*func)(void);
    void* arg;
} TaskData;

static void* thread_start(void* arg) {
    TaskData* data = (TaskData*)arg;
    if (data && data->func) {
        data->func();
    }
    free(data);
    return NULL;
}

int64_t vp_spawn_thread(void (*func)(void)) {
    TaskData* data = (TaskData*)malloc(sizeof(TaskData));
    if (!data) return -1;
    
    data->func = func;
    data->arg = NULL;
    
    pthread_t thread;
    if (pthread_create(&thread, NULL, thread_start, data) != 0) {
        free(data);
        return -1;
    }
    
    pthread_detach(thread);
    
    return atomic_fetch_add(&g_thread_counter, 1);
}
