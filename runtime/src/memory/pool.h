/**
 * Viper Object Pool Header
 * Phase 4: Memory Extensions
 */

#ifndef VIPER_POOL_H
#define VIPER_POOL_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Object Pool Type                              */
/* ============================================ */

typedef struct VpObjectPool {
    size_t obj_size;
    int64_t capacity;
    void* free_list;
    void* allocated;
    int64_t total_allocated;
    int64_t total_freed;
} VpObjectPool;

/* ============================================ */
/* Object Pool Functions                         */
/* ============================================ */

VpObjectPool* vp_pool_create(size_t obj_size, int64_t capacity);
void vp_pool_destroy(VpObjectPool* pool);
void* vp_pool_alloc(VpObjectPool* pool);
void vp_pool_free(VpObjectPool* pool, void* ptr);
int64_t vp_pool_available(VpObjectPool* pool);
int64_t vp_pool_allocated(VpObjectPool* pool);
int64_t vp_pool_freed(VpObjectPool* pool);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_POOL_H */
