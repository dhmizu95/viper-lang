/**
 * Viper Fiber Runtime
 * 
 * Stackful coroutines (goroutines) for supporting millions of concurrent tasks.
 * Features:
 * - M:N scheduling (M fibers on N threads)
 * - Dynamic stack growth (2KB -> 64KB)
 * - Context switching via setjmp/longjmp
 * - Work-stealing between threads
 */

#ifndef VIPER_FIBER_H
#define VIPER_FIBER_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Configuration                                */
/* ============================================ */

#define FIBER_INITIAL_STACK_SIZE 2048   /* 2KB initial stack */
#define FIBER_MAX_STACK_SIZE 65536      /* 64KB max stack */
#define FIBER_STACK_GROW_STEP 4096      /* Grow by 4KB */
#define FIBER_DEFAULT_STACK_SIZE 8192   /* 8KB default */

/* ============================================ */
/* Fiber States                                */
/* ============================================ */

typedef enum {
    FIBER_NEW = 0,         /* Created, not yet started */
    FIBER_READY = 1,       /* Ready to run */
    FIBER_RUNNING = 2,     /* Currently executing */
    FIBER_WAITING = 3,     /* Waiting on I/O or channel */
    FIBER_COMPLETED = 4,   /* Finished execution */
    FIBER_CANCELLED = 5    /* Cancelled */
} ViperFiberState;

/* ============================================ */
/* Fiber Control Block                          */
/* ============================================ */

typedef struct ViperFiber ViperFiber;

struct ViperFiber {
    /* Fiber ID */
    uint64_t id;
    
    /* State */
    ViperFiberState state;
    
    /* Stack */
    void* stack_base;          /* Bottom of stack (high address) */
    void* stack_ptr;            /* Current stack pointer */
    size_t stack_size;         /* Current stack size */
    size_t stack_capacity;      /* Allocated capacity */
    
    /* Function to execute */
    void (*func)(void*);
    void* arg;
    
    /* Return value */
    void* result;
    
    /* Parent fiber (who spawned this one) */
    ViperFiber* parent;
    
    /* Scheduler link */
    ViperFiber* next_ready;
    ViperFiber* prev_ready;
    
    /* Thread affinity (0 = any) */
    int32_t affinity;
    
    /* Debug info */
    const char* name;
};

/* ============================================ */
/* Fiber API                                   */
/* ============================================ */

/**
 * Create a new fiber
 * @param func Function to execute
 * @param arg Argument to pass to function
 * @param stack_size Initial stack size (0 = default)
 * @return New fiber, or NULL on failure
 */
ViperFiber* vp_fiber_create(void (*func)(void*), void* arg, size_t stack_size);

/**
 * Free a fiber
 * @param fiber Fiber to free
 */
void vp_fiber_free(ViperFiber* fiber);

/**
 * Start executing a fiber
 * @param fiber Fiber to start
 * @return 0 on success, -1 on failure
 */
int vp_fiber_start(ViperFiber* fiber);

/**
 * Yield execution to scheduler
 * Called automatically on I/O wait, channel ops, etc.
 */
void vp_fiber_yield(void);

/**
 * Resume a fiber
 * @param fiber Fiber to resume
 */
void vp_fiber_resume(ViperFiber* fiber);

/**
 * Get current running fiber
 * @return Current fiber, or NULL if on main thread
 */
ViperFiber* vp_fiber_current(void);

/**
 * Switch to another fiber
 * @param from Fiber to switch from
 * @param to Fiber to switch to
 */
void vp_fiber_switch(ViperFiber* from, ViperFiber* to);

/**
 * Grow fiber stack
 * @param fiber Fiber to grow stack for
 * @param new_size New stack size
 * @return 0 on success, -1 on failure
 */
int vp_fiber_grow_stack(ViperFiber* fiber, size_t new_size);

/**
 * Get fiber ID
 * @param fiber Fiber
 * @return Fiber ID
 */
uint64_t vp_fiber_id(ViperFiber* fiber);

/**
 * Get fiber state
 * @param fiber Fiber
 * @return Current state
 */
ViperFiberState vp_fiber_state(ViperFiber* fiber);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_FIBER_H */
