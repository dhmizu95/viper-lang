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
    return future;
}

void vp_future_free(ViperFuture* future) {
    if (!future) return;
    free(future);
}

void vp_future_set_result(ViperFuture* future, int64_t result) {
    if (!future) return;
    future->result = result;
    future->state = ASYNC_COMPLETED;
    
    if (future->callback) {
        future->callback(future);
    }
}

int64_t vp_future_await(ViperFuture* future) {
    if (!future) return 0;
    
    /* For simple implementation, just spin until ready */
    while (future->state != ASYNC_COMPLETED && future->state != ASYNC_ERROR) {
        /* Yield to scheduler */
    }
    
    return future->result;
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

/* Schedule a task to run asynchronously */
int64_t vp_async_spawn(void (*func)(void*), void* arg) {
    return vp_event_loop_spawn(func, arg);
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
