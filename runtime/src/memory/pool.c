/**
 * Viper Object Pool Implementation
 * Phase 4: Memory Extensions
 */

#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include "pool.h"

VpObjectPool* vp_pool_create(size_t obj_size, int64_t capacity) {
    if (obj_size == 0 || capacity <= 0) {
        return NULL;
    }

    VpObjectPool* pool = (VpObjectPool*)malloc(sizeof(VpObjectPool));
    if (!pool) {
        return NULL;
    }

    pool->obj_size = obj_size;
    pool->capacity = capacity;
    pool->total_allocated = 0;
    pool->total_freed = 0;

    /* Pre-allocate the objects */
    size_t total_size = obj_size * capacity;
    pool->allocated = malloc(total_size);
    if (!pool->allocated) {
        free(pool);
        return NULL;
    }

    /* Initialize free list - each object points to the next */
    char* objects = (char*)pool->allocated;
    pool->free_list = objects;

    for (int64_t i = 0; i < capacity - 1; i++) {
        char* current = objects + (i * obj_size);
        char* next = objects + ((i + 1) * obj_size);
        *(void**)current = next;
    }

    /* Last object points to NULL */
    char* last = objects + ((capacity - 1) * obj_size);
    *(void**)last = NULL;

    return pool;
}

void vp_pool_destroy(VpObjectPool* pool) {
    if (!pool) return;
    free(pool->allocated);
    free(pool);
}

void* vp_pool_alloc(VpObjectPool* pool) {
    if (!pool || !pool->free_list) {
        return NULL;
    }

    /* Get the first free object */
    void* obj = pool->free_list;
    pool->free_list = *(void**)obj;

    pool->total_allocated++;

    return obj;
}

void vp_pool_free(VpObjectPool* pool, void* ptr) {
    if (!pool || !ptr) return;

    /* Add the object back to the free list */
    *(void**)ptr = pool->free_list;
    pool->free_list = ptr;

    pool->total_freed++;
}

int64_t vp_pool_available(VpObjectPool* pool) {
    if (!pool) return 0;

    /* Count free objects */
    int64_t count = 0;
    void* current = pool->free_list;
    while (current != NULL) {
        count++;
        current = *(void**)current;
    }

    return count;
}

int64_t vp_pool_allocated(VpObjectPool* pool) {
    if (!pool) return 0;
    return pool->total_allocated;
}

int64_t vp_pool_freed(VpObjectPool* pool) {
    if (!pool) return 0;
    return pool->total_freed;
}
