/**
 * Viper Fiber Pool - Slab Allocator Header
 */

#ifndef VIPER_FIBER_POOL_H
#define VIPER_FIBER_POOL_H

#include <stddef.h>
#include <stdint.h>
#include <pthread.h>
#include "fiber.h"

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Fiber Pool Structure                        */
/* ============================================ */

typedef struct ViperFiberPool {
    ViperFiber* fibers;           /* Array of fiber control blocks */
    ViperFiber** free_list;       /* Free list for O(1) allocation */
    size_t capacity;              /* Total capacity */
    size_t available;             /* Available fibers */
    size_t allocated;             /* Currently allocated */
    pthread_mutex_t mutex;        /* Thread safety */
} ViperFiberPool;

/* ============================================ */
/* Pool Lifecycle                              */
/* ============================================ */

/**
 * Create a new fiber pool
 * @param initial_size Initial number of fibers (0 = default)
 * @return New pool, or NULL on failure
 */
ViperFiberPool* vp_fiber_pool_create(size_t initial_size);

/**
 * Destroy a fiber pool
 * @param pool Pool to destroy
 */
void vp_fiber_pool_destroy(ViperFiberPool* pool);

/* ============================================ */
/* Allocation                                  */
/* ============================================ */

/**
 * Allocate a fiber from the pool
 * @param pool Pool to allocate from
 * @return Fiber, or NULL if pool exhausted
 */
ViperFiber* vp_fiber_pool_alloc(ViperFiberPool* pool);

/**
 * Free a fiber back to the pool
 * @param pool Pool
 * @param fiber Fiber to free
 */
void vp_fiber_pool_free(ViperFiberPool* pool, ViperFiber* fiber);

/* ============================================ */
/* Statistics                                  */
/* ============================================ */

size_t vp_fiber_pool_available(ViperFiberPool* pool);
size_t vp_fiber_pool_allocated(ViperFiberPool* pool);
size_t vp_fiber_pool_capacity(ViperFiberPool* pool);

/* ============================================ */
/* Global Pool (convenience)                   */
/* ============================================ */

ViperFiberPool* vp_fiber_pool_get_global(void);
ViperFiber* vp_fiber_alloc_global(void);
void vp_fiber_free_global(ViperFiber* fiber);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_FIBER_POOL_H */
