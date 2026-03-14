/**
 * Viper Concurrency Runtime
 *
 * High-level concurrency runtime that provides the interface
 * for Viper's M:N threading system with fiber-based scheduling.
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdatomic.h>
#include "viper_stdlib.h"
#include "channel.h"
#include "wait_group.h"
#include "thread_pool.h"
#include "scheduler.h"
#include "fiber.h"
#include "event_loop.h"

/* ============================================ */
/* Forward Declarations                        */
/* ============================================ */

static void vp_concurrency_ensure_init(void);
void vp_init_threadpool(size_t num_threads);

/* ============================================ */
/* Global State                                 */
/* ============================================ */

static ViperThreadPool* _Atomic g_thread_pool = NULL;
static ViperScheduler* _Atomic g_scheduler = NULL;
static ViperEventLoop* _Atomic g_event_loop = NULL;
static _Atomic int64_t g_initialized = 0;

/* ============================================ */
/* Initialization                               */
/* ============================================ */

static void vp_concurrency_ensure_init(void) {
    if (atomic_load(&g_initialized)) return;

    /* Initialize scheduler first */
    vp_scheduler_init(0);  /* Auto-detect CPU count */
    atomic_store(&g_scheduler, vp_scheduler_get_global());

    /* Initialize thread pool for blocking operations */
    vp_init_threadpool(0);

    /* Initialize event loop for async I/O */
    atomic_store(&g_event_loop, vp_event_loop_get());

    atomic_store(&g_initialized, 1);
}

/* ============================================ */
/* Channel Operations                           */
/* ============================================ */

ViperChannel* vp_chan_create(int64_t capacity) {
    vp_concurrency_ensure_init();
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
    vp_concurrency_ensure_init();
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
/* Fiber-based Task Submission                 */
/* ============================================ */

void vp_submit_task(void (*func)(void*), void* data) {
    vp_concurrency_ensure_init();

    /* Use fiber scheduler for M:N scheduling */
    ViperScheduler* sched = atomic_load(&g_scheduler);
    if (sched) {
        vp_scheduler_submit_task(func, data);
    } else {
        /* Fallback to direct execution */
        if (func) func(data);
    }
}

void vp_wait_all_tasks(void) {
    vp_concurrency_ensure_init();

    ViperScheduler* sched = atomic_load(&g_scheduler);
    if (sched) {
        vp_scheduler_wait_all();
    }
}

int64_t vp_pending_tasks(void) {
    ViperScheduler* sched = atomic_load(&g_scheduler);
    if (sched) {
        return vp_scheduler_pending_tasks();
    }
    return 0;
}

/* ============================================ */
/* Thread Pool Operations (for blocking I/O)   */
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

/* ============================================ */
/* Event Loop Operations (async I/O)           */
/* ============================================ */

ViperEventLoop* vp_get_event_loop(void) {
    vp_concurrency_ensure_init();
    return atomic_load(&g_event_loop);
}

/* ============================================ */
/* Scheduler Shutdown                          */
/* ============================================ */

void vp_shutdown_scheduler(void) {
    ViperScheduler* sched = atomic_exchange(&g_scheduler, NULL);
    if (sched) {
        vp_scheduler_shutdown();
    }

    ViperEventLoop* loop = atomic_exchange(&g_event_loop, NULL);
    if (loop) {
        vp_event_loop_destroy(loop);
    }

    atomic_store(&g_initialized, 0);
}

/* ============================================ */
/* Statistics                                  */
/* ============================================ */

void vp_concurrency_stats(uint64_t* fibers_created, uint64_t* fibers_completed,
                          uint64_t* context_switches) {
    vp_scheduler_stats(fibers_created, fibers_completed, context_switches);
}
