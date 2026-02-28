/**
 * Viper Runtime - Asyncio Module
 * Adapter over existing event_loop_epoll.c for async I/O
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <unistd.h>
#include <time.h>
#include "viper_stdlib.h"
#include "event_loop.h"

/* ============================================ */
/* Event Loop Wrapper                           */
/* ============================================ */

static ViperEventLoop* global_loop = NULL;

void vp_asyncio_init(void) {
    if (!global_loop) {
        global_loop = vp_event_loop_create();
    }
}

void vp_asyncio_cleanup(void) {
    if (global_loop) {
        vp_event_loop_destroy(global_loop);
        global_loop = NULL;
    }
}

/* ============================================ */
/* Sleep/Yield                                  */
/* ============================================ */

void vp_asyncio_sleep(double seconds) {
    if (seconds <= 0) return;
    
    struct timespec ts;
    ts.tv_sec = (time_t)seconds;
    ts.tv_nsec = (long)((seconds - (double)ts.tv_sec) * 1e9);
    nanosleep(&ts, NULL);
}

/* ============================================ */
/* Task/Future Management                       */
/* ============================================ */

typedef struct ViperTask {
    int64_t id;
    void* coroutine;  /* Opaque coroutine pointer */
    int64_t done;
    int64_t cancelled;
    void* result;
} ViperTask;

ViperTask* vp_asyncio_create_task(void* coro) {
    ViperTask* task = (ViperTask*)vp_arc_alloc(sizeof(ViperTask));
    if (!task) return NULL;
    
    static int64_t task_counter = 0;
    task->id = ++task_counter;
    task->coroutine = coro;
    task->done = 0;
    task->cancelled = 0;
    task->result = NULL;
    
    return task;
}

void vp_asyncio_task_free(ViperTask* task) {
    if (!task) return;
    vp_arc_release(task);
}

int64_t vp_asyncio_task_done(ViperTask* task) {
    return task ? task->done : 0;
}

int64_t vp_asyncio_task_cancelled(ViperTask* task) {
    return task ? task->cancelled : 0;
}

int64_t vp_asyncio_task_cancel(ViperTask* task) {
    if (!task) return -1;
    task->cancelled = 1;
    task->done = 1;
    return 0;
}

void* vp_asyncio_task_result(ViperTask* task) {
    return task ? task->result : NULL;
}

/* ============================================ */
/* Gather (run multiple coroutines)             */
/* ============================================ */

ViperList* vp_asyncio_gather(ViperList* coroutines) {
    ViperList* results = vp_list_create();
    
    if (!coroutines) {
        return results;
    }
    
    /* Simplified: just return empty list */
    /* Full implementation would run all coroutines concurrently */
    
    return results;
}

/* ============================================ */
/* Wait (wait for first completion)             */
/* ============================================ */

ViperTask* vp_asyncio_wait(ViperList* tasks, double timeout) {
    if (!tasks || vp_list_len(tasks) == 0) {
        return NULL;
    }
    
    /* Simplified: return first task */
    return NULL;
}

/* ============================================ */
/* Event Loop Run                               */
/* ============================================ */

int64_t vp_asyncio_run(void* main_coro) {
    vp_asyncio_init();
    
    if (!global_loop) {
        return -1;
    }
    
    /* Run the event loop with no timeout (run forever) */
    vp_event_loop_run(global_loop, -1);
    
    return 0;
}

void vp_asyncio_stop(void) {
    if (global_loop) {
        vp_event_loop_stop(global_loop);
    }
}

/* ============================================ */
/* Async Context Manager                        */
/* ============================================ */

typedef struct ViperAsyncContext {
    void* enter_result;
    int64_t exited;
} ViperAsyncContext;

ViperAsyncContext* vp_asyncio_enter_context(void* context) {
    ViperAsyncContext* ctx = (ViperAsyncContext*)vp_arc_alloc(sizeof(ViperAsyncContext));
    if (!ctx) return NULL;
    
    ctx->enter_result = NULL;  /* Would call __aenter__ */
    ctx->exited = 0;
    
    return ctx;
}

void vp_asyncio_exit_context(ViperAsyncContext* ctx) {
    if (!ctx) return;
    ctx->exited = 1;
    /* Would call __aexit__ */
}

void vp_asyncio_context_free(ViperAsyncContext* ctx) {
    if (!ctx) return;
    vp_arc_release(ctx);
}

/* ============================================ */
/* Async Iterator                               */
/* ============================================ */

typedef struct ViperAsyncIterator {
    void* iterator;
    int64_t done;
} ViperAsyncIterator;

ViperAsyncIterator* vp_asyncio_aiter(void* iterable) {
    ViperAsyncIterator* it = (ViperAsyncIterator*)vp_arc_alloc(sizeof(ViperAsyncIterator));
    if (!it) return NULL;
    
    it->iterator = iterable;
    it->done = 0;
    
    return it;
}

void* vp_asyncio_anext(ViperAsyncIterator* it) {
    if (!it || it->done) {
        return NULL;
    }
    
    /* Would call __anext__ */
    return NULL;
}

void vp_asyncio_iterator_free(ViperAsyncIterator* it) {
    if (!it) return;
    vp_arc_release(it);
}

/* ============================================ */
/* Lock (async mutex)                           */
/* ============================================ */

typedef struct ViperAsyncLock {
    int64_t locked;
    ViperList* waiters;
} ViperAsyncLock;

ViperAsyncLock* vp_asyncio_lock_create(void) {
    ViperAsyncLock* lock = (ViperAsyncLock*)vp_arc_alloc(sizeof(ViperAsyncLock));
    if (!lock) return NULL;
    
    lock->locked = 0;
    lock->waiters = vp_list_create();
    
    return lock;
}

void vp_asyncio_lock_free(ViperAsyncLock* lock) {
    if (!lock) return;
    
    if (lock->waiters) {
        vp_list_free(lock->waiters);
    }
    vp_arc_release(lock);
}

int64_t vp_asyncio_lock_acquire(ViperAsyncLock* lock) {
    if (!lock) return -1;
    
    if (!lock->locked) {
        lock->locked = 1;
        return 1;
    }
    
    /* Would add to waiters and yield */
    return 0;
}

void vp_asyncio_lock_release(ViperAsyncLock* lock) {
    if (!lock) return;
    lock->locked = 0;
    /* Would wake up a waiter */
}

/* ============================================ */
/* Event (async signaling)                      */
/* ============================================ */

typedef struct ViperAsyncEvent {
    int64_t set;
    ViperList* waiters;
} ViperAsyncEvent;

ViperAsyncEvent* vp_asyncio_event_create(void) {
    ViperAsyncEvent* event = (ViperAsyncEvent*)vp_arc_alloc(sizeof(ViperAsyncEvent));
    if (!event) return NULL;
    
    event->set = 0;
    event->waiters = vp_list_create();
    
    return event;
}

void vp_asyncio_event_free(ViperAsyncEvent* event) {
    if (!event) return;
    
    if (event->waiters) {
        vp_list_free(event->waiters);
    }
    vp_arc_release(event);
}

int64_t vp_asyncio_event_is_set(ViperAsyncEvent* event) {
    return event ? event->set : 0;
}

void vp_asyncio_event_set(ViperAsyncEvent* event) {
    if (!event) return;
    event->set = 1;
    /* Would wake up all waiters */
}

void vp_asyncio_event_clear(ViperAsyncEvent* event) {
    if (!event) return;
    event->set = 0;
}

int64_t vp_asyncio_event_wait(ViperAsyncEvent* event, double timeout) {
    if (!event) return 0;
    
    if (event->set) {
        return 1;
    }
    
    /* Would add to waiters and yield */
    return 0;
}

/* ============================================ */
/* Queue (async producer-consumer)              */
/* ============================================ */

typedef struct ViperAsyncQueue {
    ViperList* items;
    int64_t maxsize;
} ViperAsyncQueue;

ViperAsyncQueue* vp_asyncio_queue_create(int64_t maxsize) {
    ViperAsyncQueue* queue = (ViperAsyncQueue*)vp_arc_alloc(sizeof(ViperAsyncQueue));
    if (!queue) return NULL;
    
    queue->items = vp_list_create();
    queue->maxsize = maxsize;
    
    return queue;
}

void vp_asyncio_queue_free(ViperAsyncQueue* queue) {
    if (!queue) return;
    
    if (queue->items) {
        vp_list_free(queue->items);
    }
    vp_arc_release(queue);
}

int64_t vp_asyncio_queue_size(ViperAsyncQueue* queue) {
    return queue ? vp_list_len(queue->items) : 0;
}

int64_t vp_asyncio_queue_empty(ViperAsyncQueue* queue) {
    return vp_asyncio_queue_size(queue) == 0;
}

int64_t vp_asyncio_queue_full(ViperAsyncQueue* queue) {
    if (!queue || queue->maxsize <= 0) return 0;
    return vp_asyncio_queue_size(queue) >= queue->maxsize;
}

void vp_asyncio_queue_put(ViperAsyncQueue* queue, int64_t item) {
    if (!queue) return;
    vp_list_append(queue->items, item);
}

int64_t vp_asyncio_queue_get(ViperAsyncQueue* queue) {
    if (!queue || vp_list_len(queue->items) == 0) {
        return 0;
    }
    return vp_list_pop(queue->items);
}

void vp_asyncio_queue_task_done(ViperAsyncQueue* queue) {
    /* Would decrement unfinished tasks counter */
    (void)queue;
}

void vp_asyncio_queue_join(ViperAsyncQueue* queue) {
    /* Would wait for all tasks to be done */
    (void)queue;
}

/* ============================================ */
/* Semaphore (async counting lock)              */
/* ============================================ */

typedef struct ViperAsyncSemaphore {
    int64_t value;
    ViperList* waiters;
} ViperAsyncSemaphore;

ViperAsyncSemaphore* vp_asyncio_semaphore_create(int64_t value) {
    ViperAsyncSemaphore* sem = (ViperAsyncSemaphore*)vp_arc_alloc(sizeof(ViperAsyncSemaphore));
    if (!sem) return NULL;
    
    sem->value = value;
    sem->waiters = vp_list_create();
    
    return sem;
}

void vp_asyncio_semaphore_free(ViperAsyncSemaphore* sem) {
    if (!sem) return;
    
    if (sem->waiters) {
        vp_list_free(sem->waiters);
    }
    vp_arc_release(sem);
}

int64_t vp_asyncio_semaphore_acquire(ViperAsyncSemaphore* sem) {
    if (!sem) return -1;
    
    if (sem->value > 0) {
        sem->value--;
        return 1;
    }
    
    /* Would add to waiters and yield */
    return 0;
}

void vp_asyncio_semaphore_release(ViperAsyncSemaphore* sem) {
    if (!sem) return;
    sem->value++;
    /* Would wake up a waiter */
}

/* ============================================ */
/* Timeout                                      */
/* ============================================ */

typedef struct ViperAsyncTimeout {
    double deadline;
    int64_t expired;
} ViperAsyncTimeout;

ViperAsyncTimeout* vp_asyncio_timeout_create(double seconds) {
    ViperAsyncTimeout* timeout = (ViperAsyncTimeout*)vp_arc_alloc(sizeof(ViperAsyncTimeout));
    if (!timeout) return NULL;
    
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    timeout->deadline = (double)ts.tv_sec + (double)ts.tv_nsec / 1e9 + seconds;
    timeout->expired = 0;
    
    return timeout;
}

void vp_asyncio_timeout_free(ViperAsyncTimeout* timeout) {
    if (!timeout) return;
    vp_arc_release(timeout);
}

int64_t vp_asyncio_timeout_expired(ViperAsyncTimeout* timeout) {
    if (!timeout) return 0;
    
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    double now = (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
    
    if (now >= timeout->deadline) {
        timeout->expired = 1;
    }

    return timeout->expired;
}

/* ============================================ */
/* Async Context Manager Support                */
/* ============================================ */

/* 
 * Async context manager protocol:
 * - vp_async_context_enter(context) -> result (or -1 for error)
 * - vp_async_context_exit(context, exc_type, exc_val, exc_tb) -> cleanup result
 * 
 * These functions wrap the existing ViperAsyncContext type for use
 * with the async with statement codegen.
 */

/* Enter an async context - calls __aenter__ and returns the result */
int64_t vp_async_context_enter(void* context) {
    if (!context) return -1;
    
    /* For now, just return the context pointer as-is
     * In a full implementation, this would:
     * 1. Look up __aenter__ method on the context object
     * 2. Call it as an async function
     * 3. Await the result
     * 4. Return the result
     */
    return (int64_t)context;
}

/* Exit an async context - calls __aexit__ with exception info */
int64_t vp_async_context_exit(void* context, int64_t exc_type, int64_t exc_val, int64_t exc_tb) {
    if (!context) return -1;
    
    /* For now, just release the context
     * In a full implementation, this would:
     * 1. Look up __aexit__ method on the context object
     * 2. Pass exception info (exc_type, exc_val, exc_tb) - all 0 if no exception
     * 3. Call it as an async function
     * 4. Await the result (True to suppress exception, False/None to propagate)
     * 5. Return whether exception was suppressed
     */
    vp_arc_release(context);
    return 0; /* Don't suppress exceptions */
}

/* Create an async context wrapper using existing ViperAsyncContext */
ViperAsyncContext* vp_async_context_create(void* inner) {
    ViperAsyncContext* ctx = (ViperAsyncContext*)vp_arc_alloc(sizeof(ViperAsyncContext));
    if (!ctx) return NULL;
    
    ctx->enter_result = inner;
    ctx->exited = 0;
    
    return ctx;
}

void vp_async_context_free(ViperAsyncContext* ctx) {
    if (!ctx) return;
    if (ctx->enter_result) {
        vp_arc_release(ctx->enter_result);
    }
    vp_arc_release(ctx);
}

