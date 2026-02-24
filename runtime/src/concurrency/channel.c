/**
 * Viper Channel Implementation
 * 
 * Channels provide typed communication between concurrent tasks.
 * They support both buffered and unbuffered (synchronous) communication.
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <pthread.h>
#include <stdatomic.h>
#include "channel.h"

/* ============================================ */
/* Channel Structure                            */
/* ============================================ */

struct ViperChannel {
    void** buffer;           /* Circular buffer for messages */
    size_t capacity;         /* Buffer capacity */
    size_t head;             /* Read position */
    size_t tail;             /* Write position */
    size_t count;            /* Current number of messages */
    
    pthread_mutex_t mutex;
    pthread_cond_t not_full;  /* Signal when space available */
    pthread_cond_t not_empty; /* Signal when data available */
    
    bool closed;             /* Channel is closed */
    _Atomic int64_t ref_count; /* Reference counting */
};

/* ============================================ */
/* Core Channel Functions                       */
/* ============================================ */

ViperChannel* vp_channel_create(size_t capacity) {
    ViperChannel* chan = (ViperChannel*)malloc(sizeof(ViperChannel));
    if (!chan) {
        return NULL;
    }
    
    chan->capacity = capacity > 0 ? capacity : 1;
    chan->buffer = (void**)calloc(chan->capacity, sizeof(void*));
    if (!chan->buffer) {
        free(chan);
        return NULL;
    }
    
    chan->head = 0;
    chan->tail = 0;
    chan->count = 0;
    chan->closed = false;
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
    
    pthread_mutex_lock(&chan->mutex);
    free(chan->buffer);
    pthread_mutex_unlock(&chan->mutex);
    
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

bool vp_channel_send(ViperChannel* chan, int64_t value) {
    if (!chan || chan->closed) {
        return false;
    }
    
    pthread_mutex_lock(&chan->mutex);
    
    /* Wait until there's space (or channel is closed) */
    while (chan->count >= chan->capacity && !chan->closed) {
        pthread_cond_wait(&chan->not_full, &chan->mutex);
    }
    
    if (chan->closed) {
        pthread_mutex_unlock(&chan->mutex);
        return false;
    }
    
    /* Store value directly as int64_t */
    chan->buffer[chan->tail] = (void*)(intptr_t)value;
    chan->tail = (chan->tail + 1) % chan->capacity;
    chan->count++;
    
    /* Signal that data is available */
    pthread_cond_signal(&chan->not_empty);
    pthread_mutex_unlock(&chan->mutex);
    
    return true;
}

int64_t vp_channel_recv(ViperChannel* chan) {
    if (!chan) {
        return 0;
    }
    
    pthread_mutex_lock(&chan->mutex);
    
    /* Wait until there's data (or channel is closed and empty) */
    while (chan->count == 0 && !chan->closed) {
        pthread_cond_wait(&chan->not_empty, &chan->mutex);
    }
    
    if (chan->count == 0) {
        /* Channel is closed and empty */
        pthread_mutex_unlock(&chan->mutex);
        return 0;
    }
    
    /* Get value from buffer */
    int64_t value = (int64_t)(intptr_t)chan->buffer[chan->head];
    chan->buffer[chan->head] = NULL;
    chan->head = (chan->head + 1) % chan->capacity;
    chan->count--;
    
    /* Signal that space is available */
    pthread_cond_signal(&chan->not_full);
    pthread_mutex_unlock(&chan->mutex);
    
    return value;
}

bool vp_channel_try_send(ViperChannel* chan, void* value) {
    if (!chan || chan->closed) {
        return false;
    }
    
    pthread_mutex_lock(&chan->mutex);
    
    if (chan->count >= chan->capacity) {
        pthread_mutex_unlock(&chan->mutex);
        return false;
    }
    
    chan->buffer[chan->tail] = value;
    chan->tail = (chan->tail + 1) % chan->capacity;
    chan->count++;
    
    pthread_cond_signal(&chan->not_empty);
    pthread_mutex_unlock(&chan->mutex);
    
    return true;
}

bool vp_channel_try_recv(ViperChannel* chan, void** out_value) {
    if (!chan) {
        return false;
    }
    
    pthread_mutex_lock(&chan->mutex);
    
    if (chan->count == 0) {
        pthread_mutex_unlock(&chan->mutex);
        return chan->closed;  /* Return true if closed (no more data coming) */
    }
    
    *out_value = chan->buffer[chan->head];
    chan->buffer[chan->head] = NULL;
    chan->head = (chan->head + 1) % chan->capacity;
    chan->count--;
    
    pthread_cond_signal(&chan->not_full);
    pthread_mutex_unlock(&chan->mutex);
    
    return true;
}

void vp_channel_close(ViperChannel* chan) {
    if (!chan) return;
    
    pthread_mutex_lock(&chan->mutex);
    chan->closed = true;
    
    /* Wake up all waiters */
    pthread_cond_broadcast(&chan->not_full);
    pthread_cond_broadcast(&chan->not_empty);
    pthread_mutex_unlock(&chan->mutex);
}

bool vp_channel_is_closed(ViperChannel* chan) {
    if (!chan) return true;
    
    pthread_mutex_lock(&chan->mutex);
    bool closed = chan->closed;
    pthread_mutex_unlock(&chan->mutex);
    
    return closed;
}

size_t vp_channel_len(ViperChannel* chan) {
    if (!chan) return 0;
    
    pthread_mutex_lock(&chan->mutex);
    size_t len = chan->count;
    pthread_mutex_unlock(&chan->mutex);
    
    return len;
}
