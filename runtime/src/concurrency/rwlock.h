/**
 * Viper Read-Write Lock Header
 * Phase 4: Synchronization Primitives
 */

#ifndef VIPER_RWLOCK_H
#define VIPER_RWLOCK_H

#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Read-Write Lock Type                         */
/* ============================================ */

typedef struct VpRwLock {
    pthread_rwlock_t lock;
} VpRwLock;

/* ============================================ */
/* Read-Write Lock Functions                    */
/* ============================================ */

VpRwLock* vp_rwlock_create(void);
void vp_rwlock_destroy(VpRwLock* rwlock);
void vp_rwlock_read_lock(VpRwLock* rwlock);
void vp_rwlock_read_unlock(VpRwLock* rwlock);
void vp_rwlock_write_lock(VpRwLock* rwlock);
void vp_rwlock_write_unlock(VpRwLock* rwlock);
bool vp_rwlock_try_read_lock(VpRwLock* rwlock);
bool vp_rwlock_try_write_lock(VpRwLock* rwlock);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_RWLOCK_H */
