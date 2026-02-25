/**
 * Viper Semaphore Header
 * Phase 4: Synchronization Primitives
 */

#ifndef VIPER_SEMAPHORE_H
#define VIPER_SEMAPHORE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Semaphore Type                                */
/* ============================================ */

typedef struct VpSemaphore {
    int64_t count;
    void* mutex;
    void* cond;
} VpSemaphore;

/* ============================================ */
/* Semaphore Functions                           */
/* ============================================ */

VpSemaphore* vp_sem_create(int64_t count);
void vp_sem_destroy(VpSemaphore* sem);
void vp_sem_wait(VpSemaphore* sem);
bool vp_sem_try_wait(VpSemaphore* sem);
void vp_sem_post(VpSemaphore* sem);
int64_t vp_sem_get_value(VpSemaphore* sem);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_SEMAPHORE_H */
