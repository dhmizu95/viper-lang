/**
 * Viper Task Queue Implementation
 * 
 * A work-stealing task queue based on the Chase-Lev deque algorithm.
 * Each worker thread has its own deque - it pushes/pops from the bottom,
 * while other threads can steal from the top.
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdatomic.h>
#include <stdbool.h>
#include <pthread.h>
#include "task_queue.h"

/* ============================================ */
/* Constants                                    */
/* ============================================ */

#define TASK_QUEUE_INITIAL_CAPACITY 64

/* ============================================ */
/* Task Queue Structure                         */
/* ============================================ */

struct ViperTaskQueue {
    ViperTask** tasks;
    size_t capacity;
    _Atomic size_t top;      /* Steal end */
    _Atomic size_t bottom;   /* Owner end */

    pthread_mutex_t mutex;   /* For rare contention cases */
};

/* ============================================ */
/* Task Queue Functions                         */
/* ============================================ */

ViperTaskQueue* vp_taskqueue_create(void) {
    ViperTaskQueue* queue = (ViperTaskQueue*)malloc(sizeof(ViperTaskQueue));
    if (!queue) {
        return NULL;
    }

    size_t initial_cap = TASK_QUEUE_INITIAL_CAPACITY;
    ViperTask** tasks = (ViperTask**)calloc(initial_cap, sizeof(ViperTask*));
    if (!tasks) {
        free(queue);
        return NULL;
    }

    queue->tasks = tasks;
    queue->capacity = initial_cap;
    atomic_store(&queue->top, 0);
    atomic_store(&queue->bottom, 0);
    pthread_mutex_init(&queue->mutex, NULL);

    return queue;
}

void vp_taskqueue_destroy(ViperTaskQueue* queue) {
    if (!queue) return;

    /* Free any remaining tasks */
    size_t top = atomic_load(&queue->top);
    size_t bottom = atomic_load(&queue->bottom);

    for (size_t i = top; i < bottom; i++) {
        ViperTask* task = queue->tasks[i % queue->capacity];
        if (task && task->cleanup) {
            task->cleanup(task);
        }
    }
    free(queue->tasks);

    pthread_mutex_destroy(&queue->mutex);
    free(queue);
}

void vp_taskqueue_push(ViperTaskQueue* queue, ViperTask* task) {
    if (!queue || !task) return;

    size_t bottom = atomic_load(&queue->bottom);
    size_t top = atomic_load(&queue->top);
    size_t capacity = queue->capacity;

    /* Check if queue is full */
    if (bottom - top >= capacity) {
        /* Grow the queue - this is the slow path */
        pthread_mutex_lock(&queue->mutex);

        /* Double-check after acquiring lock */
        bottom = atomic_load(&queue->bottom);
        top = atomic_load(&queue->top);
        capacity = queue->capacity;

        if (bottom - top >= capacity) {
            size_t new_capacity = capacity * 2;
            ViperTask** new_tasks = (ViperTask**)calloc(new_capacity, sizeof(ViperTask*));
            if (new_tasks) {
                ViperTask** old_tasks = queue->tasks;

                /* Copy tasks to new array */
                for (size_t i = top; i < bottom; i++) {
                    new_tasks[i % new_capacity] = old_tasks[i % capacity];
                }

                queue->tasks = new_tasks;
                queue->capacity = new_capacity;
                free(old_tasks);
            }
        }

        pthread_mutex_unlock(&queue->mutex);

        /* Reload values */
        bottom = atomic_load(&queue->bottom);
        capacity = queue->capacity;
    }

    /* Push task at bottom */
    queue->tasks[bottom % capacity] = task;
    atomic_thread_fence(memory_order_release);
    atomic_store(&queue->bottom, bottom + 1);
}

ViperTask* vp_taskqueue_pop(ViperTaskQueue* queue) {
    if (!queue) return NULL;

    size_t bottom = atomic_load(&queue->bottom);
    size_t top = atomic_load(&queue->top);

    if (top >= bottom) {
        return NULL;  /* Empty */
    }

    ViperTask* task = queue->tasks[(bottom - 1) % queue->capacity];
    atomic_store(&queue->bottom, bottom - 1);

    atomic_thread_fence(memory_order_seq_cst);
    top = atomic_load(&queue->top);

    if (top >= bottom - 1) {
        /* Race with stealer or empty */
        if (top > bottom - 1) {
            /* Stealer won - restore bottom */
            atomic_store(&queue->bottom, bottom);
            return NULL;
        }
        /* We won - set bottom to match top */
        atomic_store(&queue->bottom, top + 1);
    }

    return task;
}

ViperTask* vp_taskqueue_steal(ViperTaskQueue* queue) {
    if (!queue) return NULL;

    size_t top = atomic_load(&queue->top);
    atomic_thread_fence(memory_order_seq_cst);
    size_t bottom = atomic_load(&queue->bottom);

    if (top >= bottom) {
        return NULL;  /* Empty */
    }

    ViperTask* task = queue->tasks[top % queue->capacity];

    if (!atomic_compare_exchange_weak(&queue->top, &top, top + 1)) {
        /* CAS failed - another stealer won */
        return NULL;
    }

    return task;
}

bool vp_taskqueue_is_empty(ViperTaskQueue* queue) {
    if (!queue) return true;
    
    size_t top = atomic_load(&queue->top);
    size_t bottom = atomic_load(&queue->bottom);
    
    return top >= bottom;
}

size_t vp_taskqueue_size(ViperTaskQueue* queue) {
    if (!queue) return 0;
    
    size_t top = atomic_load(&queue->top);
    size_t bottom = atomic_load(&queue->bottom);
    
    if (top >= bottom) {
        return 0;
    }
    
    return bottom - top;
}
