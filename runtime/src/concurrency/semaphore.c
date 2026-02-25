/**
 * Viper Semaphore Implementation
 * Phase 4: Synchronization Primitives
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>
#include "semaphore.h"

VpSemaphore* vp_sem_create(int64_t count) {
    VpSemaphore* sem = (VpSemaphore*)malloc(sizeof(VpSemaphore));
    if (!sem) {
        return NULL;
    }

    sem->count = count;

    pthread_mutex_t* mutex = (pthread_mutex_t*)malloc(sizeof(pthread_mutex_t));
    pthread_mutex_init(mutex, NULL);
    sem->mutex = mutex;

    pthread_cond_t* cond = (pthread_cond_t*)malloc(sizeof(pthread_cond_t));
    pthread_cond_init(cond, NULL);
    sem->cond = cond;

    return sem;
}

void vp_sem_destroy(VpSemaphore* sem) {
    if (!sem) return;

    pthread_mutex_t* mutex = (pthread_mutex_t*)sem->mutex;
    pthread_cond_t* cond = (pthread_cond_t*)sem->cond;

    pthread_mutex_destroy(mutex);
    pthread_cond_destroy(cond);

    free(mutex);
    free(cond);
    free(sem);
}

void vp_sem_wait(VpSemaphore* sem) {
    if (!sem) return;

    pthread_mutex_t* mutex = (pthread_mutex_t*)sem->mutex;
    pthread_cond_t* cond = (pthread_cond_t*)sem->cond;

    pthread_mutex_lock(mutex);

    while (sem->count <= 0) {
        pthread_cond_wait(cond, mutex);
    }

    sem->count--;

    pthread_mutex_unlock(mutex);
}

bool vp_sem_try_wait(VpSemaphore* sem) {
    if (!sem) return false;

    pthread_mutex_t* mutex = (pthread_mutex_t*)sem->mutex;
    pthread_cond_t* cond = (pthread_cond_t*)sem->cond;

    pthread_mutex_lock(mutex);

    if (sem->count > 0) {
        sem->count--;
        pthread_mutex_unlock(mutex);
        return true;
    }

    pthread_mutex_unlock(mutex);
    return false;
}

void vp_sem_post(VpSemaphore* sem) {
    if (!sem) return;

    pthread_mutex_t* mutex = (pthread_mutex_t*)sem->mutex;
    pthread_cond_t* cond = (pthread_cond_t*)sem->cond;

    pthread_mutex_lock(mutex);

    sem->count++;

    pthread_cond_signal(cond);
    pthread_mutex_unlock(mutex);
}

int64_t vp_sem_get_value(VpSemaphore* sem) {
    if (!sem) return 0;

    pthread_mutex_t* mutex = (pthread_mutex_t*)sem->mutex;
    pthread_mutex_lock(mutex);

    int64_t count = sem->count;

    pthread_mutex_unlock(mutex);

    return count;
}
