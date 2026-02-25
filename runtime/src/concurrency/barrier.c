/**
 * Viper Barrier Implementation
 * Phase 4: Synchronization Primitives
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>
#include "barrier.h"

VpBarrier* vp_barrier_create(int64_t count) {
    if (count <= 0) {
        return NULL;
    }

    VpBarrier* barrier = (VpBarrier*)malloc(sizeof(VpBarrier));
    if (!barrier) {
        return NULL;
    }

    barrier->count = count;
    barrier->waiting = 0;

    pthread_mutex_t* mutex = (pthread_mutex_t*)malloc(sizeof(pthread_mutex_t));
    pthread_mutex_init(mutex, NULL);
    barrier->mutex = mutex;

    pthread_cond_t* cond = (pthread_cond_t*)malloc(sizeof(pthread_cond_t));
    pthread_cond_init(cond, NULL);
    barrier->cond = cond;

    return barrier;
}

void vp_barrier_destroy(VpBarrier* barrier) {
    if (!barrier) return;

    pthread_mutex_t* mutex = (pthread_mutex_t*)barrier->mutex;
    pthread_cond_t* cond = (pthread_cond_t*)barrier->cond;

    pthread_mutex_destroy(mutex);
    pthread_cond_destroy(cond);

    free(mutex);
    free(cond);
    free(barrier);
}

int vp_barrier_wait(VpBarrier* barrier) {
    if (!barrier) return -1;

    pthread_mutex_t* mutex = (pthread_mutex_t*)barrier->mutex;
    pthread_cond_t* cond = (pthread_cond_t*)barrier->cond;

    pthread_mutex_lock(mutex);

    barrier->waiting++;

    if (barrier->waiting >= barrier->count) {
        /* Signal all waiting threads to proceed */
        pthread_cond_broadcast(cond);
        pthread_mutex_unlock(mutex);
        return 0;  /* Thread that woke everyone */
    }

    /* Wait for more threads */
    pthread_cond_wait(cond, mutex);
    pthread_mutex_unlock(mutex);

    return 1;  /* Regular waiting thread */
}

int64_t vp_barrier_get_count(VpBarrier* barrier) {
    if (!barrier) return 0;
    return barrier->count;
}
