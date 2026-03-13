/**
 * Viper Async Runtime - Minimal Implementation
 * 
 * Provides async/await support with fiber-based execution.
 * This is a simplified implementation that works with the fiber scheduler.
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdatomic.h>
#include "viper_stdlib.h"
#include "tagged_int.h"
#include "gmp_bridge.h"

/* ============================================ */
/* Async Range (for async for loops)           */
/* ============================================ */

typedef struct ViperAsyncRange {
    uint64_t magic;
    int64_t current;
    int64_t end;
    int64_t step;
} ViperAsyncRange;

#define VIPER_ASYNC_RANGE_MAGIC 0x5650525F41524E47ULL  /* "VPR_ARNG" */

static int64_t tagged_to_i64(TaggedInt value) {
    if (tagged_int_is_small(value)) {
        return tagged_int_get_small(value);
    }
    return vp_bigint_to_i64(tagged_int_get_bigint(value));
}

ViperAsyncRange* vp_async_range_create(int64_t start, int64_t end, int64_t step) {
    int64_t raw_start = tagged_to_i64((TaggedInt)start);
    int64_t raw_end = tagged_to_i64((TaggedInt)end);
    int64_t raw_step = tagged_to_i64((TaggedInt)step);
    if (raw_step == 0) raw_step = 1;
    
    ViperAsyncRange* range = (ViperAsyncRange*)malloc(sizeof(ViperAsyncRange));
    if (!range) return NULL;
    
    range->magic = VIPER_ASYNC_RANGE_MAGIC;
    range->current = raw_start;
    range->end = raw_end;
    range->step = raw_step;
    
    return range;
}

int64_t vp_async_range_next(ViperAsyncRange* range) {
    if (!range || range->magic != VIPER_ASYNC_RANGE_MAGIC) return -1;
    
    if (range->current >= range->end) {
        return -1;  /* StopAsyncIteration */
    }
    
    int64_t value = range->current;
    range->current += range->step;

    return tagged_int_from_i64(value);
}

void vp_async_range_free(ViperAsyncRange* range) {
    if (range) free(range);
}

/* ============================================ */
/* Async Iteration                             */
/* ============================================ */

void* vp_async_iter(void* obj) {
    if (!obj) return NULL;

    ViperAsyncRange* range = (ViperAsyncRange*)obj;
    if (range->magic == VIPER_ASYNC_RANGE_MAGIC) {
        return obj;
    }

    return NULL;
}

/* For async range, this calls vp_async_range_next */
int64_t vp_async_next(void* iterator) {
    if (!iterator) return -1;
    
    ViperAsyncRange* range = (ViperAsyncRange*)iterator;
    return vp_async_range_next(range);
}

/* ============================================ */
/* Future/Await (Simplified)                   */
/* ============================================ */

/* Simple future that just returns the value immediately */
/* A full implementation would have pending/ready states */
int64_t vp_future_await(int64_t future_value) {
    /* For now, just return the value */
    /* This makes async/await work synchronously */
    return future_value;
}

/* ============================================ */
/* Async Task Spawning                         */
/* ============================================ */

/* Spawn an async task (uses fiber scheduler) */
extern void vp_scheduler_submit_task(void (*func)(void*), void* arg);

int64_t vp_async_spawn(void (*func)(void*), void* arg) {
    vp_scheduler_submit_task(func, arg);
    return 0;
}

void vp_async_run_loop(void) {
    /* No-op for now - scheduler handles everything */
}
