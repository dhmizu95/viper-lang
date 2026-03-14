/**
 * Viper Async Runtime
 * Basic async/await support with event loop
 *
 * Supports unlimited tasks via dynamic allocation
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdbool.h>
#include "viper_stdlib.h"
#include "tagged_int.h"
#include "gmp_bridge.h"
#include "fiber.h"
#include "scheduler.h"

#define ASYNC_INITIAL_CAPACITY 256
#define ASYNC_STACK_SIZE 8192

/* ============================================ */
/* Future State                                  */
/* ============================================ */

typedef enum {
    ASYNC_PENDING = 0,
    ASYNC_READY = 1,
    ASYNC_RUNNING = 2,
    ASYNC_COMPLETED = 3,
    ASYNC_ERROR = 4,
} AsyncState;

typedef struct ViperFuture {
    int64_t ref_count;
    AsyncState state;
    int64_t result;
    void (*callback)(struct ViperFuture*);
    void* user_data;
    ViperFiber* waiting_fiber;    /* NEW: Fiber awaiting this future */
} ViperFuture;

/* ============================================ */
/* Task Control Block                            */
/* ============================================ */

typedef struct ViperTask {
    int64_t id;
    void (*func)(void*);
    void* arg;
    ViperFuture* future;
    int state;
    struct ViperTask* next;
} ViperTask;

/* ============================================ */
/* Event Loop                                    */
/* ============================================ */

typedef struct ViperEventLoop {
    ViperTask** tasks;           /* Dynamic array of task pointers */
    int64_t task_capacity;       /* Current capacity of task array */
    int64_t task_count;          /* Number of active tasks */
    int64_t current_task_id;
    ViperTask* running_task;
    ViperTask* completed_tasks;
} ViperEventLoop;

static ViperEventLoop* global_event_loop = NULL;

/* ============================================ */
/* Future Functions                              */
/* ============================================ */

ViperFuture* vp_future_create(void) {
    ViperFuture* future = (ViperFuture*)malloc(sizeof(ViperFuture));
    if (!future) return NULL;
    future->ref_count = 1;
    future->state = ASYNC_PENDING;
    future->result = 0;
    future->callback = NULL;
    future->user_data = NULL;
    future->waiting_fiber = NULL;
    return future;
}

void vp_future_free(ViperFuture* future) {
    if (!future) return;
    free(future);
}

void vp_future_retain(ViperFuture* future) {
    if (!future) return;
    future->ref_count += 1;
}

void vp_future_release(ViperFuture* future) {
    if (!future) return;
    future->ref_count -= 1;
    if (future->ref_count <= 0) {
        vp_future_free(future);
    }
}

void vp_future_set_result(ViperFuture* future, int64_t result) {
    if (!future) return;
    
    future->result = result;
    future->state = ASYNC_COMPLETED;

    /* Wake up waiting fiber (if any) */
    if (future->waiting_fiber) {
        future->waiting_fiber->state = FIBER_READY;
        vp_scheduler_add_ready(future->waiting_fiber);
    }

    /* Invoke callback if registered */
    if (future->callback) {
        future->callback(future);
    }
}

int64_t vp_future_await(ViperFuture* future) {
    if (!future) return 0;

    /* Fast path: future already ready */
    if (future->state == ASYNC_COMPLETED || future->state == ASYNC_ERROR) {
        return future->result;
    }

    /* Register current fiber as waiting on this future */
    ViperFiber* current = vp_fiber_current();
    if (current) {
        current->waiting_on = future;
        future->waiting_fiber = current;
    }

    /* Wait until future is ready, yielding fiber each iteration */
    while (future->state != ASYNC_COMPLETED && future->state != ASYNC_ERROR) {
        /* Yield to scheduler - this is the key change for async/await! */
        vp_fiber_yield();
    }

    /* Clear waiting references */
    if (current) {
        current->waiting_on = NULL;
    }
    future->waiting_fiber = NULL;

    return future->result;
}

int64_t vp_future_await_and_release(ViperFuture* future) {
    int64_t result = vp_future_await(future);
    vp_future_release(future);
    return result;
}

bool vp_future_is_ready(ViperFuture* future) {
    if (!future) return false;
    return future->state == ASYNC_COMPLETED || future->state == ASYNC_ERROR;
}

/* ============================================ */
/* Event Loop Functions                         */
/* ============================================ */

ViperEventLoop* vp_event_loop_create(void) {
    ViperEventLoop* loop = (ViperEventLoop*)malloc(sizeof(ViperEventLoop));
    if (!loop) return NULL;
    
    loop->task_capacity = ASYNC_INITIAL_CAPACITY;
    loop->task_count = 0;
    loop->current_task_id = 0;
    loop->running_task = NULL;
    loop->completed_tasks = NULL;
    
    loop->tasks = (ViperTask**)malloc(sizeof(ViperTask*) * ASYNC_INITIAL_CAPACITY);
    if (!loop->tasks) {
        free(loop);
        return NULL;
    }
    
    for (int64_t i = 0; i < ASYNC_INITIAL_CAPACITY; i++) {
        loop->tasks[i] = NULL;
    }
    
    return loop;
}

void vp_event_loop_free(ViperEventLoop* loop) {
    if (!loop) return;
    
    if (loop->tasks) {
        for (int64_t i = 0; i < loop->task_count; i++) {
            if (loop->tasks[i]) {
                free(loop->tasks[i]);
            }
        }
        free(loop->tasks);
    }
    
    free(loop);
}

ViperEventLoop* vp_event_loop_get(void) {
    if (!global_event_loop) {
        global_event_loop = vp_event_loop_create();
    }
    return global_event_loop;
}

int64_t vp_event_loop_spawn(void (*func)(void*), void* arg) {
    ViperEventLoop* loop = vp_event_loop_get();
    if (!loop) return -1;
    
    /* Grow task array if needed */
    if (loop->task_count >= loop->task_capacity) {
        int64_t new_capacity = loop->task_capacity * 2;
        ViperTask** new_tasks = (ViperTask**)realloc(loop->tasks, sizeof(ViperTask*) * new_capacity);
        if (!new_tasks) return -1;
        loop->tasks = new_tasks;
        loop->task_capacity = new_capacity;
    }
    
    ViperTask* task = (ViperTask*)malloc(sizeof(ViperTask));
    if (!task) return -1;
    
    task->id = loop->current_task_id++;
    task->func = func;
    task->arg = arg;
    task->future = vp_future_create();
    task->state = ASYNC_PENDING;
    task->next = NULL;
    
    loop->tasks[loop->task_count++] = task;
    
    return task->id;
}

void vp_event_loop_run(ViperEventLoop* loop) {
    if (!loop) return;
    
    /* Simple event loop - run all pending tasks */
    for (int64_t i = 0; i < loop->task_count; i++) {
        ViperTask* task = loop->tasks[i];
        if (task && task->state == ASYNC_PENDING) {
            loop->running_task = task;
            task->state = ASYNC_RUNNING;
            
            /* Execute the task */
            if (task->func) {
                task->func(task->arg);
            }
            
            task->state = ASYNC_COMPLETED;
        }
    }
}

void vp_event_loop_run_until_complete(ViperEventLoop* loop) {
    if (!loop) return;
    
    bool has_pending = true;
    while (has_pending) {
        has_pending = false;
        
        for (int64_t i = 0; i < loop->task_count; i++) {
            ViperTask* task = loop->tasks[i];
            if (task && task->state == ASYNC_PENDING) {
                has_pending = true;
                loop->running_task = task;
                task->state = ASYNC_RUNNING;
                
                if (task->func) {
                    task->func(task->arg);
                }
                
                task->state = ASYNC_COMPLETED;
            }
        }
    }
}

/* ============================================ */
/* Async/Await Builtins                         */
/* ============================================ */

/* Sleep for specified milliseconds (async) */
typedef struct SleepRequest {
    ViperFuture* future;
    int64_t wake_time_ms;
    bool completed;
} SleepRequest;

int64_t vp_async_sleep(int64_t milliseconds) {
    // Create a future for this sleep operation
    ViperFuture* future = vp_future_create();
    if (!future) return 0;
    
    // For JIT mode, we'll do a simple blocking sleep
    // In a full implementation, this would register with event loop and yield
    struct timespec ts;
    ts.tv_sec = milliseconds / 1000;
    ts.tv_nsec = (milliseconds % 1000) * 1000000;
    nanosleep(&ts, NULL);
    
    // Set result and return
    vp_future_set_result(future, 0);
    
    // Return future pointer as i64 (caller will await it)
    return (int64_t)(uintptr_t)future;
}

/* Schedule a task to run asynchronously */
/* Spawn async work on the fiber scheduler */
extern void vp_scheduler_submit_task(void (*func)(void*), void* arg);

int64_t vp_async_spawn(void (*func)(void*), void* arg) {
    if (!func) return -1;
    vp_scheduler_submit_task(func, arg);
    return 0;
}

/* Await a future result */
int64_t vp_async_await(ViperFuture* future) {
    return vp_future_await(future);
}

/* Run event loop */
void vp_async_run_loop(void) {
    ViperEventLoop* loop = vp_event_loop_get();
    vp_event_loop_run_until_complete(loop);
}

/* Create a completed future with a value */
ViperFuture* vp_async_ready(int64_t value) {
    ViperFuture* future = vp_future_create();
    future->state = ASYNC_COMPLETED;
    future->result = value;
    return future;
}

/* Gather multiple futures and return array of results */
typedef struct GatherResult {
    int64_t* results;
    int64_t count;
} GatherResult;

int64_t vp_future_gather(int64_t* futures_ptr, int64_t count) {
    if (!futures_ptr || count <= 0) return 0;
    
    // Allocate result array
    int64_t* results = (int64_t*)malloc(sizeof(int64_t) * count);
    if (!results) return 0;
    
    // Wait for all futures and collect results
    for (int64_t i = 0; i < count; i++) {
        ViperFuture* future = (ViperFuture*)(uintptr_t)futures_ptr[i];
        results[i] = vp_future_await_and_release(future);
    }
    
    // Return pointer to results as i64
    return (int64_t)(uintptr_t)results;
}

void vp_future_gather_free(int64_t results_ptr, int64_t count) {
    if (results_ptr) {
        free((void*)(uintptr_t)results_ptr);
    }
}

/* ============================================ */
/* Async Iteration                              */
/* ============================================ */

/* Async generator for range-like iteration */
typedef struct ViperAsyncRange {
    uint64_t magic;
    int64_t current;      /* Current index */
    int64_t end;          /* End value */
    int64_t step;         /* Step value */
} ViperAsyncRange;

/* Magic tag to validate async range pointers */
#define VIPER_ASYNC_RANGE_MAGIC 0x5650525F41524E47ULL  /* "VPR_ARNG" */

static int64_t tagged_to_i64(TaggedInt value) {
    if (tagged_int_is_small(value)) {
        return tagged_int_get_small(value);
    }
    return vp_bigint_to_i64(tagged_int_get_bigint(value));
}

/* Create an async range iterator */
ViperAsyncRange* vp_async_range_create(int64_t start, int64_t end, int64_t step) {
    ViperAsyncRange* range = (ViperAsyncRange*)malloc(sizeof(ViperAsyncRange));
    if (!range) return NULL;
    range->magic = VIPER_ASYNC_RANGE_MAGIC;
    range->current = tagged_to_i64((TaggedInt)start);
    range->end = tagged_to_i64((TaggedInt)end);
    range->step = tagged_to_i64((TaggedInt)step);
    if (range->step == 0) {
        range->step = 1;
    }
    return range;
}

/* Get next value from async range */
/* Returns next value, or -1 to signal StopAsyncIteration */
int64_t vp_async_range_next(ViperAsyncRange* range) {
    if (!range || range->magic != VIPER_ASYNC_RANGE_MAGIC) return -1;
    
    /* Check if we've already reached the end before this call */
    if (range->step > 0) {
        if (range->current >= range->end) return -1;
    } else if (range->step < 0) {
        if (range->current <= range->end) return -1;
    } else {
        return -1;  /* step of 0 - infinite loop prevention */
    }
    
    int64_t result = range->current;
    range->current += range->step;

    return tagged_int_from_i64(result);
}

/* Free async range */
void vp_async_range_free(ViperAsyncRange* range) {
    if (range) free(range);
}

/* Get async iterator from async iterable */
/* Currently handles async range objects (ViperAsyncRange*) */
/* Returns pointer to the async iterator state, or NULL on error */
void* vp_async_iter(void* obj) {
    if (!obj) return NULL;

    /* Only accept known async range objects for now */
    ViperAsyncRange* range = (ViperAsyncRange*)obj;
    if (range->magic == VIPER_ASYNC_RANGE_MAGIC) {
        return obj;
    }

    /* Unknown async iterator type - return NULL to avoid segfaults */
    return NULL;
}

/* Get next item from async iterator */
/* For async range, this calls vp_async_range_next */
/* Returns next value, or -1 for StopAsyncIteration */
int64_t vp_async_next(void* iterator) {
    if (!iterator) return -1;
    
    /* Assume it's an async range */
    ViperAsyncRange* range = (ViperAsyncRange*)iterator;
    return vp_async_range_next(range);
}
