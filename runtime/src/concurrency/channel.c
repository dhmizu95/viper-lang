/**
 * Viper Channel Implementation - Lock-Free Ring Buffer
 *
 * Channels provide typed communication between concurrent tasks.
 * Uses lock-free ring buffer for high-performance operations.
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>
#include <stdatomic.h>
#include <sched.h>
#include "channel.h"
#include "../fiber.h"

/* ============================================ */
/* Channel Structure                            */
/* ============================================ */

struct ViperChannel {
    /* Ring buffer */
    _Atomic int64_t* buffer;    /* Lock-free buffer */
    size_t capacity;            /* Buffer capacity (power of 2) */
    
    /* Lock-free indices */
    _Atomic size_t head;        /* Read position (consumer) */
    _Atomic size_t tail;        /* Write position (producer) */
    
    /* For blocking operations */
    pthread_mutex_t mutex;
    pthread_cond_t not_full;
    pthread_cond_t not_empty;
    _Atomic int64_t waiting_senders;
    _Atomic int64_t waiting_receivers;
    
    /* State */
    _Atomic bool closed;
    _Atomic int64_t ref_count;
};

/* ============================================ */
/* Helper Functions                            */
/* ============================================ */

static size_t next_power_of_2(size_t n) {
    size_t power = 1;
    while (power < n) {
        power *= 2;
    }
    return power;
}

static void spin_wait(void) {
    #if defined(__x86_64__) || defined(_M_X64) || defined(_M_AMD64)
        __builtin_ia32_pause();
    #elif defined(__aarch64__)
        __asm__ volatile("yield" ::: "memory");
    #else
        sched_yield();
    #endif
}

/* ============================================ */
/* Core Channel Functions                       */
/* ============================================ */

ViperChannel* vp_channel_create(size_t capacity) {
    ViperChannel* chan = calloc(1, sizeof(ViperChannel));
    if (!chan) {
        return NULL;
    }
    
    /* Capacity must be power of 2 for efficient modulo */
    chan->capacity = next_power_of_2(capacity > 0 ? capacity : 16);
    
    /* Allocate aligned buffer for better cache performance */
    chan->buffer = calloc(chan->capacity, sizeof(_Atomic int64_t));
    if (!chan->buffer) {
        free(chan);
        return NULL;
    }
    
    atomic_store(&chan->head, 0);
    atomic_store(&chan->tail, 0);
    atomic_store(&chan->waiting_senders, 0);
    atomic_store(&chan->waiting_receivers, 0);
    atomic_store(&chan->closed, false);
    atomic_store(&chan->ref_count, 1);
    
    pthread_mutex_init(&chan->mutex, NULL);
    pthread_cond_init(&chan->not_full, NULL);
    pthread_cond_init(&chan->not_empty, NULL);
    
    return chan;
}

void vp_channel_destroy(ViperChannel* chan) {
    if (!chan) return;
    
    /* Decrement reference count */
    if (atomic_fetch_sub(&chan->ref_count, 1) > 1) {
        return;
    }
    
    free(chan->buffer);
    pthread_mutex_destroy(&chan->mutex);
    pthread_cond_destroy(&chan->not_full);
    pthread_cond_destroy(&chan->not_empty);
    free(chan);
}

void vp_channel_retain(ViperChannel* chan) {
    if (!chan) return;
    atomic_fetch_add(&chan->ref_count, 1);
}

void vp_channel_release(ViperChannel* chan) {
    if (!chan) return;
    vp_channel_destroy(chan);
}

/* ============================================ */
/* Lock-Free Send (for buffered channels)      */
/* ============================================ */

bool vp_channel_send(ViperChannel* chan, int64_t value) {
    if (!chan || atomic_load(&chan->closed)) {
        return false;
    }
    
    size_t capacity_mask = chan->capacity - 1;
    int spin_count = 0;
    const int MAX_SPINS = 100;
    
    while (1) {
        size_t tail = atomic_load_explicit(&chan->tail, memory_order_relaxed);
        size_t head = atomic_load_explicit(&chan->head, memory_order_acquire);
        size_t next_tail = (tail + 1) & capacity_mask;
        
        /* Check if buffer is full */
        if (next_tail == head) {
            /* Buffer full - spin briefly then block */
            if (spin_count++ < MAX_SPINS) {
                spin_wait();
                continue;
            }
            
            /* Block on mutex */
            atomic_fetch_add(&chan->waiting_senders, 1);
            pthread_mutex_lock(&chan->mutex);
            while (((atomic_load(&chan->tail) + 1) & capacity_mask) == atomic_load(&chan->head)) {
                pthread_cond_wait(&chan->not_full, &chan->mutex);
            }
            pthread_mutex_unlock(&chan->mutex);
            atomic_fetch_sub(&chan->waiting_senders, 1);
            spin_count = 0;
            continue;
        }
        
        /* Try to claim the slot */
        if (atomic_compare_exchange_weak_explicit(
                &chan->tail, &tail, next_tail,
                memory_order_acq_rel, memory_order_relaxed)) {
            /* Successfully claimed slot - write value */
            atomic_store_explicit(&chan->buffer[tail], value, memory_order_release);
            
            /* Wake up receivers if needed */
            if (atomic_load(&chan->waiting_receivers) > 0) {
                pthread_cond_signal(&chan->not_empty);
            }
            return true;
        }
        
        /* CAS failed - another thread claimed the slot, retry */
    }
}

/* ============================================ */
/* Lock-Free Receive (for buffered channels)   */
/* ============================================ */

int64_t vp_channel_recv(ViperChannel* chan) {
    if (!chan) {
        return 0;
    }
    
    size_t capacity_mask = chan->capacity - 1;
    int spin_count = 0;
    const int MAX_SPINS = 100;
    
    while (1) {
        size_t head = atomic_load_explicit(&chan->head, memory_order_relaxed);
        size_t tail = atomic_load_explicit(&chan->tail, memory_order_acquire);
        
        /* Check if buffer is empty */
        if (head == tail) {
            /* Buffer empty - check if closed */
            if (atomic_load(&chan->closed)) {
                return 0;
            }
            
            /* Spin briefly then block */
            if (spin_count++ < MAX_SPINS) {
                spin_wait();
                continue;
            }
            
            /* Block on mutex */
            atomic_fetch_add(&chan->waiting_receivers, 1);
            pthread_mutex_lock(&chan->mutex);
            while (atomic_load(&chan->head) == atomic_load(&chan->tail) && 
                   !atomic_load(&chan->closed)) {
                pthread_cond_wait(&chan->not_empty, &chan->mutex);
            }
            pthread_mutex_unlock(&chan->mutex);
            atomic_fetch_sub(&chan->waiting_receivers, 1);
            spin_count = 0;
            continue;
        }
        
        /* Read value */
        int64_t value = atomic_load_explicit(&chan->buffer[head], memory_order_acquire);
        
        /* Try to advance head */
        size_t next_head = (head + 1) & capacity_mask;
        if (atomic_compare_exchange_weak_explicit(
                &chan->head, &head, next_head,
                memory_order_acq_rel, memory_order_relaxed)) {
            /* Successfully consumed - wake up senders if needed */
            if (atomic_load(&chan->waiting_senders) > 0) {
                pthread_cond_signal(&chan->not_full);
            }
            return value;
        }
        
        /* CAS failed - another thread consumed, retry */
    }
}

bool vp_channel_try_send(ViperChannel* chan, void* value) {
    if (!chan || atomic_load(&chan->closed)) {
        return false;
    }
    
    size_t capacity_mask = chan->capacity - 1;
    size_t tail = atomic_load(&chan->tail);
    size_t head = atomic_load(&chan->head);
    
    if (((tail + 1) & capacity_mask) == head) {
        return false;  /* Buffer full */
    }
    
    size_t next_tail = (tail + 1) & capacity_mask;
    if (atomic_compare_exchange_strong(&chan->tail, &tail, next_tail)) {
        atomic_store(&chan->buffer[tail], (int64_t)(intptr_t)value);
        return true;
    }
    
    return false;
}

bool vp_channel_try_recv(ViperChannel* chan, void** out_value) {
    if (!chan) {
        return false;
    }
    
    size_t capacity_mask = chan->capacity - 1;
    size_t head = atomic_load(&chan->head);
    size_t tail = atomic_load(&chan->tail);
    
    if (head == tail) {
        return atomic_load(&chan->closed);  /* true if closed */
    }
    
    int64_t value = atomic_load(&chan->buffer[head]);
    size_t next_head = (head + 1) & capacity_mask;
    
    if (atomic_compare_exchange_strong(&chan->head, &head, next_head)) {
        *out_value = (void*)(intptr_t)value;
        return true;
    }
    
    return false;
}

void vp_channel_close(ViperChannel* chan) {
    if (!chan) return;
    
    atomic_store(&chan->closed, true);
    
    /* Wake up all waiters */
    pthread_mutex_lock(&chan->mutex);
    pthread_cond_broadcast(&chan->not_full);
    pthread_cond_broadcast(&chan->not_empty);
    pthread_mutex_unlock(&chan->mutex);
}

bool vp_channel_is_closed(ViperChannel* chan) {
    if (!chan) return true;
    return atomic_load(&chan->closed);
}

size_t vp_channel_len(ViperChannel* chan) {
    if (!chan) return 0;
    
    size_t tail = atomic_load(&chan->tail);
    size_t head = atomic_load(&chan->head);
    
    return (tail - head + chan->capacity) & (chan->capacity - 1);
}
