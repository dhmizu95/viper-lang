#ifndef VIPER_TASK_QUEUE_H
#define VIPER_TASK_QUEUE_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/**
 * Opaque Task type - represents a unit of work
 */
typedef struct ViperTask ViperTask;

/**
 * Task function signature
 */
typedef void (*ViperTaskFunc)(void* data);

/**
 * Task cleanup function signature
 */
typedef void (*ViperTaskCleanup)(ViperTask* task);

/**
 * Task structure
 */
struct ViperTask {
    ViperTaskFunc func;
    void* data;
    ViperTaskCleanup cleanup;
    void* next;  /* For internal use */
};

/**
 * Opaque TaskQueue type
 */
typedef struct ViperTaskQueue ViperTaskQueue;

/**
 * Create a new task queue
 * @return Pointer to TaskQueue, or NULL on failure
 */
ViperTaskQueue* vp_taskqueue_create(void);

/**
 * Destroy a task queue
 * @param queue TaskQueue to destroy
 */
void vp_taskqueue_destroy(ViperTaskQueue* queue);

/**
 * Push a task to the bottom of the queue (owner only)
 * @param queue TaskQueue
 * @param task Task to push
 */
void vp_taskqueue_push(ViperTaskQueue* queue, ViperTask* task);

/**
 * Pop a task from the bottom of the queue (owner only)
 * @param queue TaskQueue
 * @return Task, or NULL if empty
 */
ViperTask* vp_taskqueue_pop(ViperTaskQueue* queue);

/**
 * Steal a task from the top of the queue (other threads)
 * @param queue TaskQueue
 * @return Task, or NULL if empty or steal failed
 */
ViperTask* vp_taskqueue_steal(ViperTaskQueue* queue);

/**
 * Check if queue is empty
 * @param queue TaskQueue
 * @return true if empty
 */
bool vp_taskqueue_is_empty(ViperTaskQueue* queue);

/**
 * Get the number of tasks in the queue
 * @param queue TaskQueue
 * @return Number of tasks
 */
size_t vp_taskqueue_size(ViperTaskQueue* queue);

/**
 * Create a new task
 * @param func Task function
 * @param data Task data
 * @param cleanup Cleanup function (can be NULL)
 * @return Pointer to Task, or NULL on failure
 */
static inline ViperTask* vp_task_create(ViperTaskFunc func, void* data, ViperTaskCleanup cleanup) {
    ViperTask* task = (ViperTask*)malloc(sizeof(ViperTask));
    if (!task) return NULL;
    
    task->func = func;
    task->data = data;
    task->cleanup = cleanup;
    task->next = NULL;
    
    return task;
}

/**
 * Execute a task
 * @param task Task to execute
 */
static inline void vp_task_execute(ViperTask* task) {
    if (task && task->func) {
        task->func(task->data);
    }
}

#endif /* VIPER_TASK_QUEUE_H */
