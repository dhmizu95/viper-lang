/**
 * Viper WaitGroup Implementation
 * 
 * WaitGroup provides a way to wait for multiple concurrent operations
 * to complete. It uses atomic operations for thread safety.
 */

#include <stdlib.h>
#include <stdint.h>
#include <pthread.h>
#include <stdatomic.h>
#include "wait_group.h"

/* ============================================ */
/* WaitGroup Structure                          */
/* ============================================ */

struct ViperWaitGroup {
    _Atomic int64_t counter;
    pthread_mutex_t mutex;
    pthread_cond_t cond;
};

/* ============================================ */
/* Core WaitGroup Functions                     */
/* ============================================ */

ViperWaitGroup* vp_waitgroup_create_impl(void) {
    ViperWaitGroup* wg = (ViperWaitGroup*)malloc(sizeof(ViperWaitGroup));
    if (!wg) {
        return NULL;
    }
    
    atomic_store(&wg->counter, 0);
    pthread_mutex_init(&wg->mutex, NULL);
    pthread_cond_init(&wg->cond, NULL);
    
    return wg;
}

void vp_waitgroup_destroy_impl(ViperWaitGroup* wg) {
    if (!wg) return;
    
    pthread_mutex_destroy(&wg->mutex);
    pthread_cond_destroy(&wg->cond);
    free(wg);
}

void vp_waitgroup_add_impl(ViperWaitGroup* wg, int64_t delta) {
    if (!wg) return;
    
    pthread_mutex_lock(&wg->mutex);
    atomic_fetch_add(&wg->counter, delta);
    pthread_cond_broadcast(&wg->cond);
    pthread_mutex_unlock(&wg->mutex);
}

void vp_waitgroup_done_impl(ViperWaitGroup* wg) {
    if (!wg) return;
    
    pthread_mutex_lock(&wg->mutex);
    int64_t old_value = atomic_fetch_sub(&wg->counter, 1);
    
    if (old_value <= 1) {
        /* Counter would go negative or to zero - broadcast to wake all waiters */
        pthread_cond_broadcast(&wg->cond);
    }
    
    pthread_mutex_unlock(&wg->mutex);
}

void vp_waitgroup_wait_impl(ViperWaitGroup* wg) {
    if (!wg) return;
    
    pthread_mutex_lock(&wg->mutex);
    
    while (atomic_load(&wg->counter) > 0) {
        pthread_cond_wait(&wg->cond, &wg->mutex);
    }
    
    pthread_mutex_unlock(&wg->mutex);
}

int64_t vp_waitgroup_count(ViperWaitGroup* wg) {
    if (!wg) return 0;
    return atomic_load(&wg->counter);
}
