/**
 * Viper Barrier Header
 * Phase 4: Synchronization Primitives
 */

#ifndef VIPER_BARRIER_H
#define VIPER_BARRIER_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Barrier Type                                  */
/* ============================================ */

typedef struct VpBarrier {
    int64_t count;
    int64_t waiting;
    void* mutex;
    void* cond;
} VpBarrier;

/* ============================================ */
/* Barrier Functions                            */
/* ============================================ */

VpBarrier* vp_barrier_create(int64_t count);
void vp_barrier_destroy(VpBarrier* barrier);
int vp_barrier_wait(VpBarrier* barrier);
int64_t vp_barrier_get_count(VpBarrier* barrier);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_BARRIER_H */
