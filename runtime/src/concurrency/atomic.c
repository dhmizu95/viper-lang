/**
 * Viper Atomic Types Implementation
 * Phase 4: Synchronization Primitives
 */

#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdatomic.h>
#include "atomic.h"

/* ============================================ */
/* Atomic Integer Implementation                */
/* ============================================ */

VpAtomicInt* vp_atomic_int_create(int64_t initial) {
    VpAtomicInt* atomic = (VpAtomicInt*)malloc(sizeof(VpAtomicInt));
    if (!atomic) {
        return NULL;
    }
    atomic_store(&atomic->value, initial);
    return atomic;
}

void vp_atomic_int_destroy(VpAtomicInt* atomic) {
    free(atomic);
}

int64_t vp_atomic_int_load(VpAtomicInt* atomic) {
    if (!atomic) return 0;
    return atomic_load(&atomic->value);
}

void vp_atomic_int_store(VpAtomicInt* atomic, int64_t value) {
    if (!atomic) return;
    atomic_store(&atomic->value, value);
}

int64_t vp_atomic_int_add(VpAtomicInt* atomic, int64_t delta) {
    if (!atomic) return 0;
    return atomic_fetch_add(&atomic->value, delta) + delta;
}

int64_t vp_atomic_int_sub(VpAtomicInt* atomic, int64_t delta) {
    if (!atomic) return 0;
    return atomic_fetch_sub(&atomic->value, delta) - delta;
}

int64_t vp_atomic_int_fetch_add(VpAtomicInt* atomic, int64_t delta) {
    if (!atomic) return 0;
    return atomic_fetch_add(&atomic->value, delta);
}

int64_t vp_atomic_int_fetch_sub(VpAtomicInt* atomic, int64_t delta) {
    if (!atomic) return 0;
    return atomic_fetch_sub(&atomic->value, delta);
}

int64_t vp_atomic_int_cas(VpAtomicInt* atomic, int64_t expected, int64_t desired) {
    if (!atomic) return 0;
    atomic_compare_exchange_strong(&atomic->value, &expected, desired);
    return expected;
}

int64_t vp_atomic_int_swap(VpAtomicInt* atomic, int64_t desired) {
    if (!atomic) return 0;
    return atomic_exchange(&atomic->value, desired);
}

/* ============================================ */
/* Atomic Boolean Implementation               */
/* ============================================ */

VpAtomicBool* vp_atomic_bool_create(bool initial) {
    VpAtomicBool* atomic = (VpAtomicBool*)malloc(sizeof(VpAtomicBool));
    if (!atomic) {
        return NULL;
    }
    atomic_store(&atomic->value, initial);
    return atomic;
}

void vp_atomic_bool_destroy(VpAtomicBool* atomic) {
    free(atomic);
}

bool vp_atomic_bool_load(VpAtomicBool* atomic) {
    if (!atomic) return false;
    return atomic_load(&atomic->value);
}

void vp_atomic_bool_store(VpAtomicBool* atomic, bool value) {
    if (!atomic) return;
    atomic_store(&atomic->value, value);
}

bool vp_atomic_bool_cas(VpAtomicBool* atomic, bool expected, bool desired) {
    if (!atomic) return false;
    atomic_compare_exchange_strong(&atomic->value, &expected, desired);
    return expected;
}

bool vp_atomic_bool_swap(VpAtomicBool* atomic, bool desired) {
    if (!atomic) return false;
    return atomic_exchange(&atomic->value, desired);
}
