/**
 * Viper Custom Allocator Implementation
 * Phase 4: Memory Extensions
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdatomic.h>
#include "allocator.h"

/* ============================================ */
/* Global Allocator State                       */
/* ============================================ */

static vp_alloc_fn g_alloc_fn = vp_default_alloc;
static vp_free_fn g_free_fn = vp_default_free;
static _Atomic bool g_is_custom = false;

/* ============================================ */
/* Statistics                                   */
/* ============================================ */

static _Atomic int64_t g_total_allocated = 0;
static _Atomic int64_t g_total_freed = 0;
static _Atomic int64_t g_allocation_count = 0;
static _Atomic int64_t g_free_count = 0;

/* ============================================ */
/* Default Allocator Implementation            */
/* ============================================ */

void* vp_default_alloc(size_t size) {
    return malloc(size);
}

void vp_default_free(void* ptr) {
    free(ptr);
}

/* ============================================ */
/* Allocator Management                         */
/* ============================================ */

void vp_set_allocator(vp_alloc_fn alloc_fn, vp_free_fn free_fn) {
    if (alloc_fn && free_fn) {
        g_alloc_fn = alloc_fn;
        g_free_fn = free_fn;
        atomic_store(&g_is_custom, true);
    }
}

vp_alloc_fn vp_get_allocator(void) {
    return g_alloc_fn;
}

vp_free_fn vp_get_free_fn(void) {
    return g_free_fn;
}

void vp_reset_allocator(void) {
    g_alloc_fn = vp_default_alloc;
    g_free_fn = vp_default_free;
    atomic_store(&g_is_custom, false);
}

/* ============================================ */
/* Statistics                                   */
/* ============================================ */

VpAllocatorStats vp_get_allocator_stats(void) {
    VpAllocatorStats stats;
    stats.total_allocated = atomic_load(&g_total_allocated);
    stats.total_freed = atomic_load(&g_total_freed);
    stats.current_allocated = stats.total_allocated - stats.total_freed;
    stats.allocation_count = atomic_load(&g_allocation_count);
    stats.free_count = atomic_load(&g_free_count);
    return stats;
}

void vp_reset_allocator_stats(void) {
    atomic_store(&g_total_allocated, 0);
    atomic_store(&g_total_freed, 0);
    atomic_store(&g_allocation_count, 0);
    atomic_store(&g_free_count, 0);
}

bool vp_allocator_is_custom(void) {
    return atomic_load(&g_is_custom);
}

/* ============================================ */
/* Tracked Allocation Functions                */
/* ============================================ */

void* vp_tracked_alloc(size_t size) {
    void* ptr = g_alloc_fn(size);
    if (ptr) {
        atomic_fetch_add(&g_total_allocated, size);
        atomic_fetch_add(&g_allocation_count, 1);
    }
    return ptr;
}

void vp_tracked_free(void* ptr) {
    if (ptr) {
        atomic_fetch_add(&g_free_count, 1);
    }
    g_free_fn(ptr);
}
