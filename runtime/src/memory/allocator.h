/**
 * Viper Custom Allocator Header
 * Phase 4: Memory Extensions
 */

#ifndef VIPER_ALLOCATOR_H
#define VIPER_ALLOCATOR_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Allocator Function Types                     */
/* ============================================ */

typedef void* (*vp_alloc_fn)(size_t size);
typedef void (*vp_free_fn)(void* ptr);

/* ============================================ */
/* Allocator Statistics                         */
/* ============================================ */

typedef struct VpAllocatorStats {
    int64_t total_allocated;
    int64_t total_freed;
    int64_t current_allocated;
    int64_t allocation_count;
    int64_t free_count;
} VpAllocatorStats;

/* ============================================ */
/* Allocator Functions                           */
/* ============================================ */

/* Simple malloc wrapper for JIT (vp_free is in arc.c) */
void* vp_malloc(size_t size);

void vp_set_allocator(vp_alloc_fn alloc_fn, vp_free_fn free_fn);
vp_alloc_fn vp_get_allocator(void);
vp_free_fn vp_get_free_fn(void);
void vp_reset_allocator(void);

void* vp_default_alloc(size_t size);
void vp_default_free(void* ptr);

VpAllocatorStats vp_get_allocator_stats(void);
void vp_reset_allocator_stats(void);

bool vp_allocator_is_custom(void);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_ALLOCATOR_H */
