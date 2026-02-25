/**
 * Viper Read-Write Lock Implementation
 * Phase 4: Synchronization Primitives
 */

#include <stdlib.h>
#include <pthread.h>
#include "rwlock.h"

VpRwLock* vp_rwlock_create(void) {
    VpRwLock* rwlock = (VpRwLock*)malloc(sizeof(VpRwLock));
    if (!rwlock) {
        return NULL;
    }

    pthread_rwlock_init(&rwlock->lock, NULL);
    return rwlock;
}

void vp_rwlock_destroy(VpRwLock* rwlock) {
    if (!rwlock) return;
    pthread_rwlock_destroy(&rwlock->lock);
    free(rwlock);
}

void vp_rwlock_read_lock(VpRwLock* rwlock) {
    if (!rwlock) return;
    pthread_rwlock_rdlock(&rwlock->lock);
}

void vp_rwlock_read_unlock(VpRwLock* rwlock) {
    if (!rwlock) return;
    pthread_rwlock_unlock(&rwlock->lock);
}

void vp_rwlock_write_lock(VpRwLock* rwlock) {
    if (!rwlock) return;
    pthread_rwlock_wrlock(&rwlock->lock);
}

void vp_rwlock_write_unlock(VpRwLock* rwlock) {
    if (!rwlock) return;
    pthread_rwlock_unlock(&rwlock->lock);
}

bool vp_rwlock_try_read_lock(VpRwLock* rwlock) {
    if (!rwlock) return false;
    return pthread_rwlock_tryrdlock(&rwlock->lock) == 0;
}

bool vp_rwlock_try_write_lock(VpRwLock* rwlock) {
    if (!rwlock) return false;
    return pthread_rwlock_trywrlock(&rwlock->lock) == 0;
}
