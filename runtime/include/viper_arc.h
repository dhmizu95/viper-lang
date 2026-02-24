/**
 * Viper ARC (Automatic Reference Counting) Header
 */

#ifndef VIPER_ARC_H
#define VIPER_ARC_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdatomic.h>

/* ============================================ */
/* ARC Header Structure                         */
/* ============================================ */

/* 
 * All heap-allocated objects have this header prepended.
 * The header contains the reference count.
 * 
 * Memory layout:
 *   [ViperHeader][Object Data...]
 *    ^           ^
 *    |           |
 *  returned    user gets
 *  from        this pointer
 *  malloc      
 */

typedef struct {
    _Atomic int64_t ref_count;
    void (*destructor)(void*);
} ViperHeader;

/* ============================================ */
/* ARC Core Functions                           */
/* ============================================ */

/**
 * Allocate memory with ARC header
 * @param size Size of the object (excluding header)
 * @return Pointer to the object (after header)
 */
void* vp_arc_alloc(size_t size);

/**
 * Free memory with ARC header
 * @param ptr Pointer to the object (after header)
 */
void vp_arc_free(void* ptr);

/**
 * Increment reference count
 * @param ptr Pointer to the object
 */
void vp_arc_retain(void* ptr);

/**
 * Decrement reference count and free if zero
 * @param ptr Pointer to the object
 */
void vp_arc_release(void* ptr);

/**
 * Get current reference count
 * @param ptr Pointer to the object
 * @return Current reference count
 */
int64_t vp_arc_ref_count(void* ptr);

/**
 * Set destructor callback for when object is freed
 * @param ptr Pointer to the object
 * @param destructor Function to call before freeing
 */
void vp_arc_set_destructor(void* ptr, void (*destructor)(void*));

/* ============================================ */
/* Helper Macros                                */
/* ============================================ */

/* Get header from object pointer */
#define VP_GET_HEADER(ptr) ((ViperHeader*)(ptr) - 1)

/* Get object pointer from header */
#define VP_GET_OBJECT(header) ((void*)((ViperHeader*)(header) + 1))

/* Atomic increment */
#define VP_ATOMIC_INC(ptr) atomic_fetch_add((ptr), 1)

/* Atomic decrement */
#define VP_ATOMIC_DEC(ptr) atomic_fetch_sub((ptr), 1)

#endif /* VIPER_ARC_H */
