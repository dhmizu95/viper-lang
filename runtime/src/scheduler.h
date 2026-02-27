/**
 * Viper Fiber Scheduler
 *
 * M:N fiber scheduler for supporting 10M+ concurrent tasks.
 * Features:
 * - Work-stealing between threads
 * - Per-thread local queues
 * - Global distribution queues
 * - Dynamic thread scaling
 */

#ifndef VIPER_SCHEDULER_H
#define VIPER_SCHEDULER_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "fiber.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Configuration                                */
/* ============================================ */

#define SCHEDULER_MAX_THREADS 1024
#define SCHEDULER_DEFAULT_THREADS 0  /* Auto-detect CPU count */
#define SCHEDULER_MIN_THREADS 1
#define MAX_STEAL_ATTEMPTS 4
#define LOCAL_QUEUE_SIZE 256

/* ============================================ */
/* Opaque Types                                 */
/* ============================================ */

typedef struct ViperScheduler ViperScheduler;

/* ============================================ */
/* Scheduler Lifecycle                         */
/* ============================================ */

/**
 * Initialize the global scheduler
 * @param num_threads Number of worker threads (0 = auto-detect)
 * @return 0 on success, -1 on failure
 */
int vp_scheduler_init(int num_threads);

/**
 * Shutdown the global scheduler
 */
void vp_scheduler_shutdown(void);

/**
 * Get the global scheduler instance
 * @return Global scheduler, or NULL if not initialized
 */
ViperScheduler* vp_scheduler_get_global(void);

/* ============================================ */
/* Fiber Scheduling                            */
/* ============================================ */

/**
 * Spawn a new fiber and schedule it for execution
 * @param func Function to execute
 * @param arg Argument to pass to function
 * @return Fiber handle, or NULL on failure
 */
ViperFiber* vp_scheduler_spawn(void (*func)(void*), void* arg);

/**
 * Add a ready fiber to the scheduler queue
 * @param fiber Fiber to schedule
 */
void vp_scheduler_add_ready(ViperFiber* fiber);

/**
 * Get next ready fiber (called by worker threads)
 * @return Next fiber to run, or NULL if none available
 */
ViperFiber* vp_scheduler_get_ready(void);

/**
 * Put a fiber to sleep (yield)
 * @param fiber Fiber to suspend
 */
void vp_scheduler_put_to_sleep(ViperFiber* fiber);

/* ============================================ */
/* Statistics                                  */
/* ============================================ */

/**
 * Get global scheduler statistics
 * @param created Output: fibers created
 * @param completed Output: fibers completed
 * @param switches Output: context switches
 */
void vp_scheduler_stats(uint64_t* created, uint64_t* completed, uint64_t* switches);

/**
 * Get per-thread statistics
 * @param thread_id Thread ID
 * @param fibers_run Output: fibers executed by this thread
 * @param steals_attempted Output: steal attempts
 * @param steals_succeeded Output: successful steals
 */
void vp_scheduler_thread_stats(int thread_id, uint64_t* fibers_run,
                               uint64_t* steals_attempted, uint64_t* steals_succeeded);

/**
 * Enable/disable dynamic thread scaling
 * @param enabled true to enable, false to disable
 */
void vp_scheduler_set_scaling(bool enabled);

/* ============================================ */
/* Integration with Thread Pool                */
/* ============================================ */

/**
 * Submit a task to run on the fiber scheduler
 * This is the main entry point for 'task' keyword codegen
 * @param func Function to execute
 * @param arg Argument (packed struct with function args)
 */
void vp_scheduler_submit_task(void (*func)(void*), void* arg);

/**
 * Wait for all pending fibers to complete
 */
void vp_scheduler_wait_all(void);

/**
 * Get number of pending tasks
 * @return Number of pending fibers
 */
int64_t vp_scheduler_pending_tasks(void);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_SCHEDULER_H */
