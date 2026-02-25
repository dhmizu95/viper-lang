/**
 * Viper Atomic Types Header
 * Phase 4: Synchronization Primitives
 */

#ifndef VIPER_ATOMIC_H
#define VIPER_ATOMIC_H

#include <stdint.h>
#include <stdbool.h>
#include <stdatomic.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================ */
/* Atomic Integer Type                           */
/* ============================================ */

typedef struct VpAtomicInt {
    _Atomic int64_t value;
} VpAtomicInt;

/* ============================================ */
/* Atomic Boolean Type                          */
/* ============================================ */

typedef struct VpAtomicBool {
    _Atomic bool value;
} VpAtomicBool;

/* ============================================ */
/* Atomic Integer Functions                     */
/* ============================================ */

VpAtomicInt* vp_atomic_int_create(int64_t initial);
void vp_atomic_int_destroy(VpAtomicInt* atomic);
int64_t vp_atomic_int_load(VpAtomicInt* atomic);
void vp_atomic_int_store(VpAtomicInt* atomic, int64_t value);
int64_t vp_atomic_int_add(VpAtomicInt* atomic, int64_t delta);
int64_t vp_atomic_int_sub(VpAtomicInt* atomic, int64_t delta);
int64_t vp_atomic_int_fetch_add(VpAtomicInt* atomic, int64_t delta);
int64_t vp_atomic_int_fetch_sub(VpAtomicInt* atomic, int64_t delta);
int64_t vp_atomic_int_cas(VpAtomicInt* atomic, int64_t expected, int64_t desired);
int64_t vp_atomic_int_swap(VpAtomicInt* atomic, int64_t desired);

/* ============================================ */
/* Atomic Boolean Functions                     */
/* ============================================ */

VpAtomicBool* vp_atomic_bool_create(bool initial);
void vp_atomic_bool_destroy(VpAtomicBool* atomic);
bool vp_atomic_bool_load(VpAtomicBool* atomic);
void vp_atomic_bool_store(VpAtomicBool* atomic, bool value);
bool vp_atomic_bool_cas(VpAtomicBool* atomic, bool expected, bool desired);
bool vp_atomic_bool_swap(VpAtomicBool* atomic, bool desired);

#ifdef __cplusplus
}
#endif

#endif /* VIPER_ATOMIC_H */
