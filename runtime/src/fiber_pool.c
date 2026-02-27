/**
 * Viper Fiber Pool - Slab Allocator for Fibers
 * 
 * Pre-allocates fiber control blocks to reduce allocation overhead.
 * Provides O(1) allocation and deallocation.
 */

#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>
#include <pthread.h>
#include "fiber.h"
#include "fiber_pool.h"

/* ============================================ */
/* Configuration                               */
/* ============================================ */

#define FIBER_POOL_INITIAL_SIZE 1024
#define FIBER_POOL_GROWTH_FACTOR 2
#define FIBER_POOL_MAX_SIZE (1024 * 1024)  /* 1M fibers */

/* ============================================ */
/* Fiber Pool Implementation                   */
/* ============================================ */

ViperFiberPool* vp_fiber_pool_create(size_t initial_size) {
    if (initial_size == 0) {
        initial_size = FIBER_POOL_INITIAL_SIZE;
    }
    if (initial_size > FIBER_POOL_MAX_SIZE) {
        initial_size = FIBER_POOL_MAX_SIZE;
    }
    
    ViperFiberPool* pool = calloc(1, sizeof(ViperFiberPool));
    if (!pool) return NULL;
    
    pool->capacity = initial_size;
    pool->fibers = calloc(pool->capacity, sizeof(ViperFiber));
    if (!pool->fibers) {
        free(pool);
        return NULL;
    }
    
    /* Initialize free list */
    pool->free_list = calloc(pool->capacity, sizeof(ViperFiber*));
    if (!pool->free_list) {
        free(pool->fibers);
        free(pool);
        return NULL;
    }
    
    /* All fibers start as available */
    for (size_t i = 0; i < pool->capacity; i++) {
        pool->free_list[i] = &pool->fibers[i];
        pool->fibers[i].id = i;
        pool->fibers[i].pool = pool;
    }
    
    pool->available = pool->capacity;
    pool->allocated = 0;
    
    pthread_mutex_init(&pool->mutex, NULL);
    
    return pool;
}

void vp_fiber_pool_destroy(ViperFiberPool* pool) {
    if (!pool) return;
    
    pthread_mutex_destroy(&pool->mutex);
    free(pool->free_list);
    free(pool->fibers);
    free(pool);
}

ViperFiber* vp_fiber_pool_alloc(ViperFiberPool* pool) {
    if (!pool) return NULL;
    
    pthread_mutex_lock(&pool->mutex);
    
    if (pool->available == 0) {
        /* Grow the pool */
        size_t new_size = pool->capacity * FIBER_POOL_GROWTH_FACTOR;
        if (new_size > FIBER_POOL_MAX_SIZE) {
            new_size = FIBER_POOL_MAX_SIZE;
        }
        if (new_size == pool->capacity) {
            /* At max capacity */
            pthread_mutex_unlock(&pool->mutex);
            return NULL;
        }
        
        /* Reallocate */
        ViperFiber* new_fibers = realloc(pool->fibers, new_size * sizeof(ViperFiber));
        if (!new_fibers) {
            pthread_mutex_unlock(&pool->mutex);
            return NULL;
        }
        pool->fibers = new_fibers;
        
        ViperFiber** new_free_list = realloc(pool->free_list, new_size * sizeof(ViperFiber*));
        if (!new_free_list) {
            pthread_mutex_unlock(&pool->mutex);
            return NULL;
        }
        pool->free_list = new_free_list;
        
        /* Initialize new fibers */
        for (size_t i = pool->capacity; i < new_size; i++) {
            pool->free_list[pool->available + (i - pool->capacity)] = &pool->fibers[i];
            pool->fibers[i].id = i;
            pool->fibers[i].pool = pool;
        }
        
        pool->available += (new_size - pool->capacity);
        pool->capacity = new_size;
    }
    
    /* Allocate from free list */
    pool->available--;
    ViperFiber* fiber = pool->free_list[pool->available];
    pool->allocated++;
    
    /* Reset fiber state */
    memset(fiber, 0, sizeof(ViperFiber));
    fiber->id = pool->allocated;  /* Unique ID */
    fiber->pool = pool;
    fiber->state = FIBER_NEW;
    
    pthread_mutex_unlock(&pool->mutex);
    
    return fiber;
}

void vp_fiber_pool_free(ViperFiberPool* pool, ViperFiber* fiber) {
    if (!pool || !fiber) return;
    
    pthread_mutex_lock(&pool->mutex);
    
    /* Return to free list */
    pool->free_list[pool->available] = fiber;
    pool->available++;
    pool->allocated--;
    
    pthread_mutex_unlock(&pool->mutex);
}

size_t vp_fiber_pool_available(ViperFiberPool* pool) {
    if (!pool) return 0;
    return pool->available;
}

size_t vp_fiber_pool_allocated(ViperFiberPool* pool) {
    if (!pool) return 0;
    return pool->allocated;
}

size_t vp_fiber_pool_capacity(ViperFiberPool* pool) {
    if (!pool) return 0;
    return pool->capacity;
}

/* ============================================ */
/* Global Fiber Pool (optional)                */
/* ============================================ */

static ViperFiberPool* g_global_fiber_pool = NULL;

ViperFiberPool* vp_fiber_pool_get_global(void) {
    if (!g_global_fiber_pool) {
        g_global_fiber_pool = vp_fiber_pool_create(0);
    }
    return g_global_fiber_pool;
}

ViperFiber* vp_fiber_alloc_global(void) {
    ViperFiberPool* pool = vp_fiber_pool_get_global();
    return vp_fiber_pool_alloc(pool);
}

void vp_fiber_free_global(ViperFiber* fiber) {
    if (fiber && fiber->pool) {
        vp_fiber_pool_free(fiber->pool, fiber);
    }
}
